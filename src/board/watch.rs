//! `zendo board watch --session N`: the messages and posts of a session, one JSON line each, for
//! the Monitor tool of Claude Code (BOARD-20 to BOARD-24, BOARD-27).

use super::{How, Journal, Kind, Msg, State, now_ms, post_json};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::io::Write;
use std::time::{Duration, Instant};

pub struct Watch {
    pub journal: Journal,
    pub repo: String,
    pub session: usize,
    /// `--for <id>`: only the responses to that message (BOARD-23).
    pub for_id: Option<String>,
    pub timeout: Option<Duration>,
    pub poll: Duration,
}

/// The arguments after `zendo board watch`; None when they do not parse.
pub fn parse(args: &[String]) -> Option<(usize, Option<String>, Option<Duration>)> {
    let mut it = args.iter();
    let (mut session, mut for_id, mut timeout) = (None, None, None);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--session" => session = it.next()?.parse().ok(),
            "--for" => for_id = Some(it.next()?.clone()),
            "--timeout" => timeout = Some(Duration::from_secs(it.next()?.parse().ok()?)),
            _ => return None,
        }
    }
    Some((session?, for_id, timeout))
}

#[derive(Default)]
struct Seen {
    msgs: HashSet<String>,
    posts: HashSet<String>,
    channels: Vec<String>,
}

impl Watch {
    /// Print until the end, and the exit code: 0 after a response to `--for`, 1 otherwise.
    pub fn run(&self, out: &mut impl Write) -> std::io::Result<i32> {
        let start = Instant::now();
        let mut seen = Seen::default();
        let first = self.journal.state();
        seen.posts.extend(first.live_posts().map(|p| p.post.id.clone()));
        seen.channels = first.channels_of(&self.repo, self.session);
        seen.msgs.extend(first.msgs.iter().filter(|(_, m)| first.claimed(&m.id, self.session).is_some()).map(|(_, m)| m.id.clone()));
        loop {
            if !self.journal.alive(&self.repo) {
                return end(out, "zendo is not running for this repository", 1);
            }
            let st = self.journal.state();
            let done = match &self.for_id {
                Some(id) => self.responses(&st, id, &mut seen, out)?,
                None => self.messages(&st, &mut seen, out).map(|_| None)?,
            };
            if let Some(code) = done {
                return Ok(code);
            }
            if self.timeout.is_some_and(|t| start.elapsed() >= t) {
                return end(out, "timeout", 1);
            }
            std::thread::sleep(self.poll);
        }
    }

    /// The messages that wait for the session, the new posts and the closed channels.
    fn messages(&self, st: &State, seen: &mut Seen, out: &mut impl Write) -> std::io::Result<()> {
        let me = self.session;
        let mut claimed = false;
        for m in st.pending(&self.repo, me, &|n| n == me) {
            claimed |= self.journal.claim(&m.id, me, How::Watched);
        }
        let fresh;
        let st = if claimed {
            fresh = self.journal.state();
            &fresh
        } else {
            st
        };
        // Each watch prints each watched message once, also one that another watch claimed (BOARD-27).
        for m in st.taken_as(&self.repo, me, How::Watched).into_iter().filter(|m| seen.msgs.insert(m.id.clone())) {
            print(out, &message(m))?;
        }
        let now = now_ms();
        for p in st.live_posts().filter(|p| seen.posts.insert(p.post.id.clone())) {
            print(out, &json!({ "event": "post", "post": post_json(p, Some(now), now) }))?;
        }
        let channels = st.channels_of(&self.repo, me);
        for ch in seen.channels.iter().filter(|c| !channels.contains(c)) {
            print(out, &json!({ "event": "closed", "channel": ch }))?;
        }
        seen.channels = channels;
        Ok(())
    }

    /// The responses to `id`; the exit code after the first answer or the last status.
    fn responses(&self, st: &State, id: &str, seen: &mut Seen, out: &mut impl Write) -> std::io::Result<Option<i32>> {
        let Some(m) = st.msg(id).filter(|m| m.repo == self.repo) else { return end(out, "not found", 1).map(Some) };
        for r in st.responses(m).filter(|r| seen.msgs.insert(r.id.clone())) {
            self.journal.claim(&r.id, self.session, How::Watched);
            print(out, &message(r))?;
            if r.kind == Kind::Answer || r.status.is_some_and(|s| s.last()) {
                return Ok(Some(0));
            }
        }
        Ok(None)
    }
}

fn message(m: &Msg) -> Value {
    let mut v = serde_json::to_value(m).unwrap_or_default();
    v["event"] = json!("message");
    v
}

fn print(out: &mut impl Write, v: &Value) -> std::io::Result<()> {
    writeln!(out, "{v}")?;
    out.flush()
}

/// The last line, with the reason of the end (BOARD-22).
fn end(out: &mut impl Write, reason: &str, code: i32) -> std::io::Result<i32> {
    print(out, &json!({ "event": "end", "reason": reason }))?;
    Ok(code)
}

#[cfg(test)]
#[path = "watch_tests.rs"]
mod tests;
