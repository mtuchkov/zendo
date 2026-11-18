//! The history of the messages of the user (HISTORY-1 to HISTORY-7): one for each zendo session,
//! and one common history. Both live in `.zendo/db.json`.

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The most messages that a history keeps (HISTORY-3).
pub const MAX: usize = 1000;

/// A message of the user, and when the user sent it. The text has the labels of its text chips.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sent {
    pub text: String,
    pub at: DateTime<Local>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chips: Vec<SavedChip>,
    /// When the message went into the common history (HISTORY-2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub merged: Option<DateTime<Local>>,
}

/// A text chip of a message: its label in the text, and the text that it brings (HISTORY-7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SavedChip {
    pub label: String,
    pub text: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct History {
    /// The messages of the threads that ended with `ctrl-n`, the oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub common: Vec<Sent>,
    /// The current thread of each zendo session that has one.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sessions: BTreeMap<usize, Thread>,
}

/// The history of the current thread of a zendo session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Thread {
    /// The start of the thread: it sees the common history as it was then (HISTORY-4).
    pub start: DateTime<Local>,
    /// Its messages, the oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sent: Vec<Sent>,
}

impl History {
    pub fn is_empty(&self) -> bool {
        self.common.is_empty() && self.sessions.is_empty()
    }

    /// A message that the user sent in `session`, with its text chips (HISTORY-1). The first
    /// message of a session without a thread starts one. No text, or the text and the chips of
    /// the last message of the session, adds nothing (HISTORY-7).
    pub fn push(&mut self, session: usize, text: &str, chips: Vec<SavedChip>, at: DateTime<Local>) {
        let text = text.trim();
        if text.is_empty() || self.list(session).last().is_some_and(|m| m.text == text && m.chips == chips) {
            return;
        }
        let t = self.sessions.entry(session).or_insert_with(|| Thread { start: at, sent: Vec::new() });
        t.sent.push(Sent { text: text.to_string(), at, chips, merged: None });
        keep_last(&mut t.sent);
    }

    /// A new thread in `session` at `now` (`ctrl-n`, `/clear`): its messages go into the common
    /// history, in the order of time, and the new thread sees the common history of now
    /// (HISTORY-2, HISTORY-4).
    pub fn merge(&mut self, session: usize, now: DateTime<Local>) {
        let old = self.sessions.insert(session, Thread { start: now, sent: Vec::new() });
        let sent = old.map(|t| t.sent).unwrap_or_default();
        self.common.extend(sent.into_iter().map(|m| Sent { merged: Some(now), ..m }));
        self.common.sort_by_key(|m| m.at);
        keep_last(&mut self.common);
    }

    /// The messages that `session` goes through with the arrows: the common history as it was at
    /// the start of its thread, and its own messages, in the order of time, the oldest first
    /// (HISTORY-4). A session without a thread sees the whole common history.
    pub fn list(&self, session: usize) -> Vec<Sent> {
        let thread = self.sessions.get(&session);
        let start = thread.map(|t| t.start);
        let seen = |m: &&Sent| m.merged.zip(start).is_none_or(|(merged, start)| merged <= start);
        let own = thread.into_iter().flat_map(|t| &t.sent);
        let mut all: Vec<&Sent> = self.common.iter().filter(seen).chain(own).collect();
        all.sort_by_key(|m| m.at);
        let from = all.len().saturating_sub(MAX);
        all[from..].iter().map(|m| (*m).clone()).collect()
    }
}

/// Only the last MAX messages stay (HISTORY-3).
fn keep_last(v: &mut Vec<Sent>) {
    let over = v.len().saturating_sub(MAX);
    v.drain(..over);
}

#[cfg(test)]
#[path = "history_tests.rs"]
mod tests;
