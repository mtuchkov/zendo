//! The loglets of the documents in `.zendo/db.log.json` (UNDO-4, ADR 0014): for each document, by
//! its repository path, the operations of undo and redo, the head, and the text at the head. The
//! chat input and the text fields keep their loglets in memory (UNDO-5).
use crate::text::loglet::Loglet;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// The most documents that the file keeps; the documents that the user changed last stay.
pub const MAX_DOCS: usize = 64;

const FILE: &str = "db.log.json";

#[derive(Default, Serialize, Deserialize)]
pub struct Logs {
    #[serde(default)]
    docs: BTreeMap<String, DocLog>,
}

/// The loglet of one document.
#[derive(Clone, Serialize, Deserialize)]
pub struct DocLog {
    /// The text at the head of the loglet: a change on disk shows as a difference (UNDO-6).
    pub text: String,
    pub log: Loglet,
    /// The last save of the document.
    pub at: DateTime<Local>,
}

impl Logs {
    /// The loglets of the repository; none when the file is not there or is broken (UNDO-11).
    pub fn load(root: &Path) -> Self {
        fs::read_to_string(super::state::dir(root).join(FILE)).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }

    /// Write the file atomically: a temp file, then a rename.
    pub fn save(&self, root: &Path) {
        let d = super::state::dir(root);
        if fs::create_dir_all(&d).is_err() {
            return;
        }
        super::state::ignore(&d);
        if let Ok(s) = serde_json::to_string(self) {
            let tmp = d.join(format!("{FILE}.tmp"));
            if fs::write(&tmp, s).is_ok() {
                let _ = fs::rename(&tmp, d.join(FILE));
            }
        }
    }

    pub fn get(&self, rel: &str) -> Option<&DocLog> {
        self.docs.get(rel)
    }

    /// The loglet of the document `rel` and the text at its head. An empty loglet takes the
    /// document out; over MAX_DOCS, the document saved first goes.
    pub fn put(&mut self, rel: &str, log: Loglet, text: String) {
        if log.is_empty() {
            self.docs.remove(rel);
            return;
        }
        self.docs.insert(rel.to_string(), DocLog { text, log, at: Local::now() });
        while self.docs.len() > MAX_DOCS {
            let oldest = self.docs.iter().min_by_key(|(_, d)| d.at).map(|(k, _)| k.clone());
            oldest.map(|k| self.docs.remove(&k));
        }
    }

    pub fn len(&self) -> usize {
        self.docs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }
}

#[cfg(test)]
#[path = "logs_tests.rs"]
mod tests;
