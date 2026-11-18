//! The history of the messages of the user (HISTORY-1 to HISTORY-7): one for each zendo session,
//! and one common history. Both live in `.zendo/db.json`.

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The most messages that a history keeps (HISTORY-3).
pub const MAX: usize = 1000;

/// A message of the user, and when the user sent it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sent {
    pub text: String,
    pub at: DateTime<Local>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct History {
    /// The messages of the threads that ended with `ctrl-n`, the oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub common: Vec<Sent>,
    /// The messages of the current thread of each zendo session, the oldest first.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sessions: BTreeMap<usize, Vec<Sent>>,
}

impl History {
    pub fn is_empty(&self) -> bool {
        self.common.is_empty() && self.sessions.values().all(Vec::is_empty)
    }

    /// A message that the user sent in `session` (HISTORY-1). No text, or the text of the last
    /// message of the session, adds nothing (HISTORY-7).
    pub fn push(&mut self, session: usize, text: &str, at: DateTime<Local>) {
        let text = text.trim();
        let own = self.sessions.get(&session).and_then(|v| v.last());
        if text.is_empty() || own.or(self.common.last()).is_some_and(|m| m.text == text) {
            return;
        }
        let v = self.sessions.entry(session).or_default();
        v.push(Sent { text: text.to_string(), at });
        keep_last(v);
    }

    /// `ctrl-n`: the history of `session` goes into the common history, in the order of time
    /// (HISTORY-2).
    pub fn merge(&mut self, session: usize) {
        let Some(own) = self.sessions.remove(&session) else { return };
        self.common.extend(own);
        self.common.sort_by_key(|m| m.at);
        keep_last(&mut self.common);
    }

    /// The texts that `session` goes through with the arrows: the common history and its own, in
    /// the order of time, the oldest first (HISTORY-4).
    pub fn list(&self, session: usize) -> Vec<String> {
        let mut all: Vec<&Sent> = self.common.iter().chain(self.sessions.get(&session).into_iter().flatten()).collect();
        all.sort_by_key(|m| m.at);
        let from = all.len().saturating_sub(MAX);
        all[from..].iter().map(|m| m.text.clone()).collect()
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
