use super::super::tests::{beat, fresh, other};
use super::super::tools::{Ctx, call};
use super::*;
use serde_json::json;

fn watch(c: &Ctx, for_id: Option<String>, secs: u64) -> Watch {
    Watch { journal: c.journal.clone(), repo: c.repo.clone(), session: c.session, for_id, timeout: Some(Duration::from_secs(secs)), poll: Duration::from_millis(50) }
}

/// Run the watch on a thread, and the lines that it printed with its exit code.
fn run(w: Watch) -> std::thread::JoinHandle<(Vec<Value>, i32)> {
    std::thread::spawn(move || {
        let mut out = Vec::new();
        let code = w.run(&mut out).unwrap();
        (String::from_utf8(out).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect(), code)
    })
}

// BOARD-20, BOARD-21: the watch prints the waiting message first, then a new one, one JSON line
// each, and claims them as watched, so zendo does not deliver them again.
#[test]
fn a_watch_prints_a_message_during_the_turn() {
    let c3 = fresh("watch", 3);
    beat(&c3);
    call(&c3, "ask", &json!({ "to": "session:2", "text": "one" })).unwrap().unwrap();
    let h = run(watch(&other(&c3, 2), None, 1));
    std::thread::sleep(Duration::from_millis(200));
    call(&c3, "ask", &json!({ "to": "all", "text": "two" })).unwrap().unwrap();
    let (lines, _) = h.join().unwrap();
    let texts: Vec<_> = lines.iter().filter(|l| l["event"] == "message").map(|l| l["text"].clone()).collect();
    assert_eq!(texts, [json!("one"), json!("two")]);
    let st = c3.journal.state();
    assert!(st.msgs.iter().all(|(_, m)| st.claimed(&m.id, 2) == Some(How::Watched)), "zendo does not deliver them again");
    let _ = std::fs::remove_dir_all(c3.journal.dir());
}

// BOARD-22: when zendo ends, the last line of the watch names the reason, and the watch exits.
#[test]
fn a_watch_ends_with_a_reason() {
    let c = fresh("watch-end", 2);
    beat(&c);
    let h = run(watch(&c, None, 5));
    std::thread::sleep(Duration::from_millis(200));
    c.journal.unbeat(std::process::id());
    let (lines, code) = h.join().unwrap();
    assert_eq!(lines.last().unwrap(), &json!({ "event": "end", "reason": "zendo is not running for this repository" }));
    assert_eq!(code, 1);
    let _ = std::fs::remove_dir_all(c.journal.dir());
}

// BOARD-22: without a zendo of the repository, the watch ends at once and says why.
#[test]
fn a_watch_without_zendo_ends_at_once() {
    let c = fresh("watch-dead", 2);
    let (lines, code) = run(watch(&c, None, 5)).join().unwrap();
    assert_eq!(lines, [json!({ "event": "end", "reason": "zendo is not running for this repository" })]);
    assert_eq!(code, 1);
    let _ = std::fs::remove_dir_all(c.journal.dir());
}

// BOARD-23: `--for` prints the response and exits after the first answer.
#[test]
fn a_watch_for_a_response_ends_after_it() {
    let c3 = fresh("watch-for", 3);
    beat(&c3);
    let q = call(&c3, "ask", &json!({ "to": "session:2", "text": "which commit?" })).unwrap().unwrap()["id"].as_str().unwrap().to_string();
    let h = run(watch(&c3, Some(q.clone()), 5));
    c3.journal.claim(&q, 2, How::Delivered);
    std::thread::sleep(Duration::from_millis(150));
    call(&other(&c3, 2), "answer", &json!({ "in_reply_to": q, "text": "a1b2" })).unwrap().unwrap();
    let (lines, code) = h.join().unwrap();
    assert_eq!(code, 0);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["text"], "a1b2");
    let _ = std::fs::remove_dir_all(c3.journal.dir());
}

// BOARD-24: the watch names a channel of the session that closes.
#[test]
fn a_watch_names_a_closed_channel() {
    let c3 = fresh("watch-closed", 3);
    beat(&c3);
    let q = call(&c3, "ask", &json!({ "to": "session:2", "text": "q", "reply_to": "oci" })).unwrap().unwrap()["id"].as_str().unwrap().to_string();
    c3.journal.claim(&q, 2, How::Delivered);
    let h = run(watch(&other(&c3, 2), None, 1));
    std::thread::sleep(Duration::from_millis(150));
    c3.journal.append(&[super::super::Event::ThreadEnd { repo: "/r".into(), session: 3 }]).unwrap();
    let (lines, _) = h.join().unwrap();
    assert!(lines.contains(&json!({ "event": "closed", "channel": "3/oci" })), "{lines:?}");
    let _ = std::fs::remove_dir_all(c3.journal.dir());
}

// BOARD-27: two watches of one session both print a message; the session takes it once.
#[test]
fn two_watches_print_a_message_once_each() {
    let c3 = fresh("watch-two", 3);
    beat(&c3);
    let c2 = other(&c3, 2);
    let (a, b) = (run(watch(&c2, None, 1)), run(watch(&c2, None, 1)));
    std::thread::sleep(Duration::from_millis(150));
    call(&c3, "ask", &json!({ "to": "session:2", "text": "once" })).unwrap().unwrap();
    for h in [a, b] {
        let (lines, _) = h.join().unwrap();
        assert_eq!(lines.iter().filter(|l| l["text"] == "once").count(), 1, "{lines:?}");
    }
    let st = c3.journal.state();
    assert_eq!(st.taken_as("/r", 2, How::Watched).len(), 1);
    let _ = std::fs::remove_dir_all(c3.journal.dir());
}

// BOARD-20, BOARD-23: the arguments of `zendo board watch`.
#[test]
fn the_arguments_of_the_watch_parse() {
    let a = |s: &str| s.split_whitespace().map(String::from).collect::<Vec<_>>();
    assert_eq!(parse(&a("--session 2")), Some((2, None, None)));
    assert_eq!(parse(&a("--session 3 --for m1 --timeout 30")), Some((3, Some("m1".into()), Some(Duration::from_secs(30)))));
    assert_eq!(parse(&a("--for m1")), None, "the session is needed");
    assert_eq!(parse(&a("--session 2 --loud")), None);
}
