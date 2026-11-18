use super::*;
use chrono::TimeDelta;

fn at(s: i64) -> DateTime<Local> {
    DateTime::from_timestamp(1_800_000_000 + s, 0).unwrap().with_timezone(&Local)
}

fn texts(h: &History, session: usize) -> Vec<String> {
    h.list(session).into_iter().map(|m| m.text).collect()
}

// HISTORY-1, HISTORY-4: a thread sees the common history as it was when the thread started, and
// its own messages, in the order of time; what another session merges later does not show.
#[test]
fn a_thread_sees_the_common_history_of_its_start_and_its_own_messages() {
    let mut h = History::default();
    h.push(0, "one", Vec::new(), at(1));
    h.push(1, "two", Vec::new(), at(2));
    h.push(0, "three", Vec::new(), at(3));
    assert_eq!(texts(&h, 0), ["one", "three"]);
    assert_eq!(texts(&h, 1), ["two"]);
    h.merge(1, at(4));
    assert_eq!(texts(&h, 0), ["one", "three"], "the thread of session 0 started before the merge");
    assert_eq!(texts(&h, 1), ["two"], "the new thread starts with the common history");
    assert_eq!(texts(&h, 2), ["two"], "a session without a thread sees the common history");
    h.merge(0, at(5));
    assert_eq!(texts(&h, 0), ["one", "two", "three"], "a new thread: the common history of now, by time");
    assert_eq!(texts(&h, 1), ["two"], "the thread of session 1 started before the second merge");
    h.push(2, "four", Vec::new(), at(6));
    h.merge(1, at(7));
    assert_eq!(texts(&h, 2), ["one", "two", "three", "four"], "the thread of session 2 started at its first message");
}

// HISTORY-2: a new thread moves the history of its session into the common history, in the order
// of time, and the session has no messages of its own after it.
#[test]
fn a_new_thread_merges_its_session_into_the_common_history_by_time() {
    let mut h = History::default();
    h.push(0, "a", Vec::new(), at(1));
    h.push(1, "b", Vec::new(), at(2));
    h.push(0, "c", Vec::new(), at(3));
    h.push(1, "d", Vec::new(), at(4));
    h.merge(1, at(5));
    h.merge(0, at(6));
    let common: Vec<&str> = h.common.iter().map(|m| m.text.as_str()).collect();
    assert_eq!(common, ["a", "b", "c", "d"]);
    assert!(h.sessions.values().all(|t| t.sent.is_empty()));
}

// HISTORY-3: the common history keeps the last 1000 messages; so does the history of a session.
#[test]
fn the_history_keeps_the_last_1000_messages() {
    let mut h = History::default();
    for i in 0..1005 {
        h.push(2, &format!("m{i}"), Vec::new(), at(0) + TimeDelta::seconds(i));
    }
    assert_eq!(texts(&h, 2).len(), MAX);
    h.merge(2, at(2000));
    assert_eq!(h.common.len(), MAX);
    assert_eq!(h.common.first().map(|m| m.text.as_str()), Some("m5"));
    assert_eq!(h.common.last().map(|m| m.text.as_str()), Some("m1004"));
}

// HISTORY-7: an empty text goes nowhere, and the same text as the last message of the session goes
// in once.
#[test]
fn an_empty_or_repeated_message_does_not_go_in() {
    let mut h = History::default();
    h.push(0, "  ", Vec::new(), at(1));
    h.push(0, "same", Vec::new(), at(2));
    h.push(0, "same", Vec::new(), at(3));
    h.push(0, "other", Vec::new(), at(4));
    h.push(0, "same", Vec::new(), at(5));
    assert_eq!(texts(&h, 0), ["same", "other", "same"]);
}

// HISTORY-3: the history goes into db.json and back; a db.json without it loads an empty one.
#[test]
fn the_history_goes_to_json_and_back() {
    let mut h = History::default();
    h.push(0, "kept", Vec::new(), at(1));
    h.merge(0, at(2));
    h.push(3, "mine", Vec::new(), at(3));
    let json = serde_json::to_string(&h).unwrap();
    let back: History = serde_json::from_str(&json).unwrap();
    assert_eq!(texts(&back, 3), ["kept", "mine"]);
    let old: History = serde_json::from_str("{}").unwrap();
    assert!(old.list(0).is_empty() && old.is_empty());
    let no_chips: Sent = serde_json::from_str(r#"{"text":"t","at":"2026-11-18T03:00:00-08:00"}"#).unwrap();
    assert!(no_chips.chips.is_empty(), "a message from 0.98.0 has no chips");
}

fn chip(label: &str, text: &str) -> SavedChip {
    SavedChip { label: label.into(), text: text.into() }
}

// HISTORY-7: a message keeps its text chips, with their labels and their text, also in db.json;
// the same text with other chips is another message.
#[test]
fn a_message_keeps_its_text_chips() {
    let mut h = History::default();
    h.push(0, "why ⟦fn main · 3 lines⟧", vec![chip("⟦fn main · 3 lines⟧", "fn main() {}")], at(1));
    h.push(0, "why ⟦fn main · 3 lines⟧", vec![chip("⟦fn main · 3 lines⟧", "fn other() {}")], at(2));
    h.push(0, "⟦only · 1 line⟧", vec![chip("⟦only · 1 line⟧", "only")], at(3));
    let back: History = serde_json::from_str(&serde_json::to_string(&h).unwrap()).unwrap();
    let list = back.list(0);
    assert_eq!(list.len(), 3, "the other chip text is another message; a message of only a chip goes in");
    assert_eq!(list[0].chips, [chip("⟦fn main · 3 lines⟧", "fn main() {}")]);
    assert_eq!(list[2].text, "⟦only · 1 line⟧");
}
