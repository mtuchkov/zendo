use super::*;
use chrono::TimeDelta;

fn at(s: i64) -> DateTime<Local> {
    DateTime::from_timestamp(1_800_000_000 + s, 0).unwrap().with_timezone(&Local)
}

fn texts(h: &History, session: usize) -> Vec<String> {
    h.list(session)
}

// HISTORY-1, HISTORY-4: a session sees the common history and its own messages, in the order of
// time; another session does not see them before a new thread.
#[test]
fn a_session_sees_the_common_history_and_its_own_messages_by_time() {
    let mut h = History::default();
    h.push(0, "one", at(1));
    h.push(1, "two", at(2));
    h.push(0, "three", at(3));
    assert_eq!(texts(&h, 0), ["one", "three"]);
    assert_eq!(texts(&h, 1), ["two"]);
    h.merge(1);
    assert_eq!(texts(&h, 0), ["one", "two", "three"], "the common history and its own, by time");
    assert_eq!(texts(&h, 1), ["two"], "the new thread starts with the common history");
}

// HISTORY-2: a new thread moves the history of its session into the common history, in the order
// of time, and the session has no messages of its own after it.
#[test]
fn a_new_thread_merges_its_session_into_the_common_history_by_time() {
    let mut h = History::default();
    h.push(0, "a", at(1));
    h.push(1, "b", at(2));
    h.push(0, "c", at(3));
    h.push(1, "d", at(4));
    h.merge(1);
    h.merge(0);
    let common: Vec<&str> = h.common.iter().map(|m| m.text.as_str()).collect();
    assert_eq!(common, ["a", "b", "c", "d"]);
    assert!(h.sessions.values().all(Vec::is_empty) || h.sessions.is_empty());
}

// HISTORY-3: the common history keeps the last 1000 messages; so does the history of a session.
#[test]
fn the_history_keeps_the_last_1000_messages() {
    let mut h = History::default();
    for i in 0..1005 {
        h.push(2, &format!("m{i}"), at(0) + TimeDelta::seconds(i));
    }
    assert_eq!(h.list(2).len(), MAX);
    h.merge(2);
    assert_eq!(h.common.len(), MAX);
    assert_eq!(h.common.first().map(|m| m.text.as_str()), Some("m5"));
    assert_eq!(h.common.last().map(|m| m.text.as_str()), Some("m1004"));
}

// HISTORY-7: an empty text goes nowhere, and the same text as the last message of the session goes
// in once.
#[test]
fn an_empty_or_repeated_message_does_not_go_in() {
    let mut h = History::default();
    h.push(0, "  ", at(1));
    h.push(0, "same", at(2));
    h.push(0, "same", at(3));
    h.push(0, "other", at(4));
    h.push(0, "same", at(5));
    assert_eq!(texts(&h, 0), ["same", "other", "same"]);
}

// HISTORY-3: the history goes into db.json and back; a db.json without it loads an empty one.
#[test]
fn the_history_goes_to_json_and_back() {
    let mut h = History::default();
    h.push(0, "kept", at(1));
    h.merge(0);
    h.push(3, "mine", at(2));
    let json = serde_json::to_string(&h).unwrap();
    let back: History = serde_json::from_str(&json).unwrap();
    assert_eq!(back.list(3), ["kept", "mine"]);
    let old: History = serde_json::from_str("{}").unwrap();
    assert!(old.list(0).is_empty() && old.is_empty());
}
