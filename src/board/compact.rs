//! The compaction of the journal (ADR 0021): the lines that still matter, and a snapshot of the
//! channels and of the leases and ends of the posts.

use super::{Event, HISTORY_MS, Line, Msg, State, inbox_of};
use std::collections::HashSet;

/// A delivered message is kept this long after it was sent.
pub const MSG_RETAIN_MS: u64 = 24 * 3600 * 1000;

/// The lines of a compacted journal with the same state as `lines` at the time `now`, without the
/// messages that are delivered and older than MSG_RETAIN_MS, and without the posts that left the
/// history (BOARD-1, BOARD-15).
pub(super) fn keep(lines: Vec<Line>, now: u64) -> Vec<Line> {
    let st = State::fold(lines.clone());
    let msgs = kept_msgs(&st, now);
    let since = now.saturating_sub(HISTORY_MS);
    let posts: HashSet<String> = st.posts.iter().filter(|p| p.ended_at.is_none_or(|t| t >= since)).map(|p| p.post.id.clone()).collect();
    let mut out: Vec<Line> = lines.into_iter().filter(|l| needed(&l.ev, &msgs, &posts)).collect();
    let replay = State::fold(out.clone());
    out.extend(channel_snapshot(&st, &replay, now));
    out.extend(post_snapshot(&st, &posts));
    out
}

/// The messages that stay: new ones, a message to an inbox that its session did not take yet, an
/// initiation in flight, and the responses to those.
fn kept_msgs(st: &State, now: u64) -> HashSet<String> {
    let undelivered = |m: &Msg| st.target(m).and_then(|t| inbox_of(&t)).is_some_and(|n| st.claimed(&m.id, n).is_none()) || m.to.is_none() && st.choice(&m.id).is_none();
    let mut keep: HashSet<String> = st
        .msgs
        .iter()
        .filter(|(at, m)| now.saturating_sub(*at) < MSG_RETAIN_MS || undelivered(m) || (m.kind.initiation() && st.in_flight(m)))
        .map(|(_, m)| m.id.clone())
        .collect();
    let answers: Vec<String> = st.msgs.iter().map(|(_, m)| m).filter(|m| m.in_reply_to.as_ref().is_some_and(|r| keep.contains(r))).map(|m| m.id.clone()).collect();
    keep.extend(answers);
    keep
}

/// A line stays when its message or its post stays. The snapshot replaces the lines of the
/// channels, of the leases and of the ends of the posts.
fn needed(ev: &Event, msgs: &HashSet<String>, posts: &HashSet<String>) -> bool {
    match ev {
        Event::Msg(m) => msgs.contains(&m.id),
        Event::Claim { msg, .. } | Event::Landed { msg, .. } | Event::Choice { msg, .. } => msgs.contains(msg),
        Event::Post(p) => posts.contains(&p.id),
        _ => false,
    }
}

/// The open channels with their members, and a close for each channel that the kept messages would
/// open again.
fn channel_snapshot(st: &State, replay: &State, now: u64) -> Vec<Line> {
    let open: Vec<Line> = st.channels().map(|((repo, channel), m)| Event::Channel { repo: repo.clone(), channel: channel.clone(), members: m.iter().copied().collect() }).map(|ev| Line { at: now, ev }).collect();
    let closed = replay.channels().filter(|(k, _)| st.members(&k.0, &k.1).is_none()).map(|((repo, channel), _)| Line { at: now, ev: Event::Close { repo: repo.clone(), channel: channel.clone() } });
    open.into_iter().chain(closed).collect()
}

/// For each kept post: its expiry or its renewal with their times, or its end.
fn post_snapshot(st: &State, posts: &HashSet<String>) -> Vec<Line> {
    let mut out = Vec::new();
    for p in st.posts.iter().filter(|p| posts.contains(&p.post.id)) {
        let id = p.post.id.clone();
        if let (Some(end), Some(at)) = (&p.end, p.ended_at) {
            out.push(Line { at, ev: Event::Ended { post: id, end: end.clone() } });
        } else if let Some((at, _)) = p.expired.filter(|_| p.expired()) {
            out.push(Line { at, ev: Event::Expired { post: id } });
        } else if let Some((at, _)) = p.renewed {
            out.push(Line { at, ev: Event::Renewed { post: id } });
        }
    }
    out
}

#[cfg(test)]
#[path = "compact_tests.rs"]
mod tests;
