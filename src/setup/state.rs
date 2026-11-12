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
    /// The width of the text column, set with `alt-=` and `alt--`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<u16>,
    /// Per file, keyed by the repo-relative path.
    #[serde(default)]
    pub files: BTreeMap<String, FileState>,
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
        let ignore = d.join(".gitignore");
        let want = "db.json\ndb.json.tmp\nstate.json\nagent.log\n";
        if fs::read_to_string(&ignore).map(|s| !s.contains("db.json")).unwrap_or(true) {
            let _ = fs::write(&ignore, want);
        }
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
I write short notes in the morning and want a sharp second opinion.
Keep answers short and concrete. Ask one question when something is unclear.
When you propose a change, edit the note file directly: I review each edit before it is written.
Write requirements in EARS style (WHEN / WHILE / IF ... THEN / WHERE ... the system shall ...).";

pub fn prompt(root: &Path) -> String {
    fs::read_to_string(dir(root).join("prompt.md")).unwrap_or_else(|_| DEFAULT_PROMPT.to_string())
}
