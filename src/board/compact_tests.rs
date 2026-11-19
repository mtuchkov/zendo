use super::super::tests::fresh;
use super::super::*;
use super::*;

const H: u64 = 3600 * 1000;
const NOW: u64 = 1_000 * 24 * H;

fn msg(id: &str, from: usize, to: &str, kind: Kind, reply: Option<&str>) -> Event {
    Event::Msg(Msg { id: id.into(), repo: "/r".into(), from, to: Some(to.into()), kind, status: None, in_reply_to: reply.map(Into::into), reply_to: None, text: id.into() })
}

fn line(hours_ago: u64, ev: Event) -> Line {
    Line { at: NOW - hours_ago * H, ev }
}

fn claim(id: &str, to: usize) -> Event {
    Event::Claim { msg: id.into(), to, how: How::Delivered }
}

fn post(id: &str) -> Event {
    Event::Post(Post { id: id.into(), repo: "/r".into(), session: 2, resource: id.into(), text: "x".into(), pid: None, start: None })
}

/// A journal with each kind of state: old messages that are done, an old message that waits, an
/// initiation in flight with one held behind it, an open and a closed channel, and posts that are
/// live, expired, in the history and past it.
fn journal() -> Vec<Line> {
    let mut reopen = msg("x1", 2, "x", Kind::Question, None);
    if let Event::Msg(m) = &mut reopen {
        m.to = Some("2/x".into());
    }
    vec![
        line(50, msg("done", 3, "session:2", Kind::Question, None)),
        line(50, claim("done", 2)),
        line(49, msg("done-a", 2, "session:3", Kind::Answer, Some("done"))),
        line(49, claim("done-a", 3)),
        line(48, msg("waits", 3, "session:4", Kind::Task, None)),
        line(47, msg("flight", 3, "session:1", Kind::Question, None)),
        line(47, claim("flight", 1)),
        line(30, msg("held", 3, "session:1", Kind::Question, None)),
        line(40, Event::Invite { repo: "/r".into(), channel: "3/oci".into(), session: 2 }),
        line(1, reopen),
        line(1, Event::ThreadEnd { repo: "/r".into(), session: 2 }),
        line(60, post("live")),
        line(60, post("expired")),
        line(59, Event::Expired { post: "expired".into() }),
        line(60, post("old")),
        line(9 * 24, Event::Release { post: "old".into() }),
        line(60, post("recent")),
        line(24, Event::Release { post: "recent".into() }),
    ]
}

fn oci() -> Line {
    let mut m = msg("oci", 3, "session:2", Kind::Question, None);
    if let Event::Msg(x) = &mut m {
        x.reply_to = Some("3/oci".into());
    }
    line(41, m)
}

/// What a session sees of a state.
fn view(st: &State) -> Vec<String> {
    let all = |_: usize| true;
    let mut v: Vec<String> = (0..5).map(|r| format!("{r}: {:?} {:?} {:?}", st.pending("/r", r, &all).iter().map(|m| &m.id).collect::<Vec<_>>(), st.taken("/r", r), st.channels_of("/r", r))).collect();
    v.extend(st.live_posts().map(|p| format!("live {} {}", p.post.id, p.expired())));
    v.extend(st.history().map(|p| format!("history {} {:?}", p.post.id, p.end)));
    v
}

// ADR 0021, BOARD-1, BOARD-15: a compaction drops the delivered old messages and the posts past
// the history, and keeps what each session sees.
#[test]
fn a_compaction_keeps_what_the_sessions_see() {
    let mut lines = journal();
    lines.splice(0..0, [oci(), line(41, claim("oci", 2)), line(41, Event::Landed { msg: "oci".into(), to: 2 })]);
    lines.push(Line { at: NOW, ev: Event::Landed { msg: "none".into(), to: 0 } });
    let before = State::fold(lines.clone());
    let kept = keep(lines, NOW);
    let after = State::fold(kept.clone());
    assert_eq!(view(&after), view(&before));
    assert!(view(&before).iter().any(|l| l.contains("3/oci")), "the open channel is in the view: {:?}", view(&before));
    for gone in ["done", "done-a", "oci"] {
        assert!(after.msg(gone).is_none(), "{gone} is delivered and old");
    }
    for stays in ["waits", "flight", "held"] {
        assert!(after.msg(stays).is_some(), "{stays} still matters");
    }
    assert!(after.post("old").is_none(), "past the history");
    assert!(after.channels_of("/r", 2).iter().all(|c| c != "2/x"), "a closed channel stays closed");
}

// ADR 0021: a journal over its limit compacts itself after a write, and no write of another
// writer is lost.
#[test]
fn writes_during_compactions_are_not_lost() {
    let c = fresh("compact-race", 3);
    let j = c.journal.clone().with_limit(4_000);
    std::thread::scope(|s| {
        for t in 0..4 {
            let j = j.clone();
            s.spawn(move || {
                for i in 0..40 {
                    let m = Msg { id: format!("m{t}-{i}"), repo: "/r".into(), from: t, to: Some("session:4".into()), kind: Kind::Task, status: None, in_reply_to: None, reply_to: None, text: "x".repeat(40) };
                    j.append(&[Event::Msg(m)]).unwrap();
                }
            });
        }
    });
    assert_eq!(j.state().msgs.len(), 160, "every message is there");
    let _ = std::fs::remove_dir_all(c.journal.dir());
}

// ADR 0021: a compaction makes a new file, and the stamp shows it.
#[test]
fn a_compaction_makes_a_new_file() {
    let c = fresh("compact-stamp", 3);
    c.journal.append(&[post("p")]).unwrap();
    let before = c.journal.stamp();
    c.journal.compact(now_ms()).unwrap();
    assert_ne!(c.journal.stamp().0, before.0);
    assert_eq!(c.journal.state().live_posts().count(), 1);
    let _ = std::fs::remove_dir_all(c.journal.dir());
}
