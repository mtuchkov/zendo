//! Local state in `.zendo/db.json`: theme, Claude thread ids, and per-file work:
//! engaged visits, the text you last worked on (the base for delta), and recaps.

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default, Serialize, Deserialize)]
pub struct State {
    #[serde(default)]
    pub theme: String,
    /// Thread ids per agent command, then per note.
    #[serde(default)]
    pub threads: BTreeMap<String, BTreeMap<String, String>>,
    #[serde(default = "yes")]
    pub focus_dim: bool,
    #[serde(default = "yes")]
    pub show_recap: bool,
    /// Layout preset: "standard" or "wide". Empty means wide, the default.
    #[serde(default)]
    pub layout: String,
    /// Pane widths in columns, set by dragging: "tree", "chat", "delta".
    #[serde(default)]
    pub widths: BTreeMap<String, u16>,
    /// The file tree pane is open, and shows every .md (true) or only `**/docs/**` (false).
    #[serde(default)]
    pub tree: bool,
    #[serde(default)]
    pub tree_all: bool,
    /// Tree view: "structure", "docs" or "all". Empty: structure when the repo has one, else docs.
    #[serde(default)]
    pub tree_view: String,
    /// The agent session of the chat: 0 to SESSIONS - 1 (F6 to F10).
    #[serde(default)]
    pub session: usize,
    /// The width of the text column, set with `alt-=` and `alt--`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<u16>,
    /// The width of the code column, set with `alt-=` and `alt--` on a code file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_column: Option<u16>,
    /// Per file, keyed by the repo-relative path.
    #[serde(default)]
    pub files: BTreeMap<String, FileState>,
    /// The files that the user opened in the note pane, the most recent first (FIND-2, FIND-4).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recent: Vec<Opened>,
    /// The last document that the user opened in each agent session, and the place in it
    /// (FILES-50, FILES-52, FILES-54).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub session_docs: BTreeMap<usize, LastDoc>,
    /// The name of each Claude session that has one, keyed by its session ID (NAMES-3).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub names: BTreeMap<String, String>,
}

/// A file that the user opened in the note pane, and when.
#[derive(Clone, Serialize, Deserialize)]
pub struct Opened {
    pub path: String,
    pub at: DateTime<Local>,
}

/// The last document of an agent session: its path without the worktree, and where the user left
/// it in read mode, in rendered rows.
#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LastDoc {
    pub path: String,
    #[serde(default)]
    pub scroll: usize,
    #[serde(default)]
    pub row: usize,
    #[serde(default)]
    pub col: usize,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct FileState {
    /// Visits that counted as work: an edit, or many scrolls and clicks.
    #[serde(default)]
    pub visits: u32,
    #[serde(default)]
    pub edits: u32,
    #[serde(default)]
    pub interactions: u32,
    #[serde(default)]
    pub worked_at: Option<DateTime<Local>>,
    /// The text at the end of the last visit that counted. Delta starts here.
    #[serde(default)]
    pub seen: Option<Snapshot>,
    #[serde(default)]
    pub recap: Option<Recap>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub hash: String,
    pub text: String,
    pub at: DateTime<Local>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Recap {
    pub text: String,
    /// The note text the recap describes.
    pub of_hash: String,
    pub of_text: String,
    pub at: DateTime<Local>,
    /// Written while Claude was idle (true) or on request (false).
    pub auto: bool,
}

/// FNV-1a, stable across Rust versions (unlike `DefaultHasher`).
pub fn hash(s: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{h:016x}")
}

fn yes() -> bool {
    true
}

pub fn dir(root: &Path) -> PathBuf {
    root.join(".zendo")
}

/// The files of `.zendo` that git does not see: the state, the loglets (UNDO-4) and the log.
const IGNORED: &str = "db.json\ndb.json.tmp\ndb.log.json\ndb.log.json.tmp\nstate.json\nagent.log\n";

/// `.zendo/.gitignore` gets the files of IGNORED that it does not have; its other lines stay.
pub(crate) fn ignore(d: &Path) {
    let path = d.join(".gitignore");
    let has = fs::read_to_string(&path).unwrap_or_default();
    let missing: String = IGNORED.lines().filter(|f| !has.lines().any(|l| l == *f)).map(|f| format!("{f}\n")).collect();
    if !missing.is_empty() {
        let sep = if has.is_empty() || has.ends_with('\n') { "" } else { "\n" };
        let _ = fs::write(&path, format!("{has}{sep}{missing}"));
    }
}

impl State {
    pub fn wide(&self) -> bool {
        self.layout != "standard"
    }

    pub fn session(&self, agent: &str, note: &str) -> Option<String> {
        self.threads.get(agent).and_then(|m| m.get(note)).cloned()
    }
    pub fn set_session(&mut self, agent: &str, note: &str, id: String) {
        self.threads.entry(agent.to_string()).or_default().insert(note.to_string(), id);
    }
    pub fn take_session(&mut self, agent: &str, note: &str) -> Option<String> {
        self.threads.get_mut(agent).and_then(|m| m.remove(note))
    }

    pub fn file(&mut self, rel: &str) -> &mut FileState {
        self.files.entry(rel.to_string()).or_default()
    }

    pub fn load(root: &Path) -> Self {
        let d = dir(root);
        // db.json replaced state.json in 0.3; read the old file once.
        fs::read_to_string(d.join("db.json"))
            .or_else(|_| fs::read_to_string(d.join("state.json")))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(State { focus_dim: true, show_recap: true, ..Default::default() })
    }

    pub fn save(&self, root: &Path) {
        let d = dir(root);
        if fs::create_dir_all(&d).is_err() {
            return;
        }
        ignore(&d);
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let tmp = d.join("db.json.tmp");
            if fs::write(&tmp, s).is_ok() {
                let _ = fs::rename(&tmp, d.join("db.json"));
            }
        }
    }
}

pub const DEFAULT_PROMPT: &str = "\
You are helping me think through architecture notes in this repository.
I am working with the specifications and want a sharp second opinion.
Keep answers short and concrete. Ask one question when something is unclear.";

pub fn prompt(root: &Path) -> String {
    fs::read_to_string(dir(root).join("prompt.md")).unwrap_or_else(|_| DEFAULT_PROMPT.to_string())
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
