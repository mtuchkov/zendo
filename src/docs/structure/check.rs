//! Checks of the structure file: what zendo does not understand, and quick fixes.
use super::*;

// ----- checks and quick fixes --------------------------------------------------

/// The format, for people and for Claude. Embedded from `resources/`.
pub const FORMAT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/project-structure.md"));
/// Agent instructions for `zendo doctor --fix`. Embedded from `resources/`.
pub const FIX_PROMPT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/fix-project-structure.md"));

const TOP_KEYS: [&str; 3] = ["meta", "project", "entry"];
const PROJECT_KEYS: [&str; 3] = ["name", "about", "claude"];
const ENTRY_KEYS: [&str; 7] = ["path", "label", "about", "claude", "main", "words", "template"];

/// Other names people use for the entry keys.
const ALIASES: [(&str, &str); 14] = [
    ("description", "about"),
    ("desc", "about"),
    ("purpose", "about"),
    ("instructions", "claude"),
    ("agent", "claude"),
    ("ai", "claude"),
    ("llm", "claude"),
    ("title", "label"),
    ("name", "label"),
    ("dir", "path"),
    ("folder", "path"),
    ("file", "path"),
    ("primary", "main"),
    ("pinned", "main"),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issue {
    /// Where: `entry 3 (docs/specs)`, `[project]`, …
    pub at: String,
    pub what: String,
}

/// What `zendo doctor` knows about the structure file.
pub struct Report {
    /// The parsed file, or the parse error.
    pub parsed: Result<Structure, String>,
    /// Problems in a file that parses.
    pub issues: Vec<Issue>,
}

impl Report {
    pub fn ok(&self) -> bool {
        self.parsed.is_ok() && self.issues.is_empty()
    }
}

fn clean_path(p: &str) -> String {
    let mut s = p.trim().to_string();
    while let Some(rest) = s.strip_prefix("./") {
        s = rest.to_string();
    }
    s.trim_start_matches('/').to_string()
}

/// Check the text of a structure file against the format and the repository at `root`.
pub fn check(text: &str, root: &Path) -> Report {
    let parsed: Result<Structure, String> = toml::from_str(text).map_err(|e| e.to_string());
    let mut issues = Vec::new();
    let Ok(st) = &parsed else { return Report { parsed, issues } };
    let raw: toml::Table = toml::from_str(text).unwrap_or_default();
    for k in raw.keys().filter(|k| !TOP_KEYS.contains(&k.as_str())) {
        issues.push(Issue { at: "top level".into(), what: format!("unknown key `{k}`") });
    }
    if let Some(p) = raw.get("project").and_then(|v| v.as_table()) {
        for k in p.keys().filter(|k| !PROJECT_KEYS.contains(&k.as_str())) {
            issues.push(Issue { at: "[project]".into(), what: format!("unknown key `{k}`") });
        }
    }
    let raw_entries = raw.get("entry").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let mut seen = std::collections::BTreeSet::new();
    for (i, e) in st.entries.iter().enumerate() {
        let at = format!("entry {} ({})", i + 1, if e.path().is_empty() { "no path".to_string() } else { e.paths_label() });
        if let Some(t) = raw_entries.get(i).and_then(|v| v.as_table()) {
            for k in t.keys().filter(|k| !ENTRY_KEYS.contains(&k.as_str())) {
                let hint = ALIASES.iter().find(|(a, _)| a == k).map(|(_, to)| format!("; did you mean `{to}`?")).unwrap_or_default();
                issues.push(Issue { at: at.clone(), what: format!("unknown key `{k}`{hint}") });
            }
        }
        if e.paths.iter().all(|p| p.trim().is_empty()) {
            issues.push(Issue { at, what: "empty `path`".into() });
            continue;
        }
        if let Some(t) = e.template.as_deref().filter(|t| !root.join(t).is_file()) {
            issues.push(Issue { at: at.clone(), what: format!("the template {t} is not there") });
        }
        if e.main && e.is_glob() && !e.is_group() {
            issues.push(Issue { at: at.clone(), what: "a main entry with one path cannot have `*`: zendo does not show it; a list of paths can".into() });
        }
        issues.extend(e.paths.iter().flat_map(|p| path_issues(p, e.main, root, &at, &mut seen)));
    }
    issues.extend(label_issues(st));
    Report { parsed, issues }
}

/// The problems of one location of an entry.
fn path_issues(p: &str, main: bool, root: &Path, at: &str, seen: &mut std::collections::BTreeSet<String>) -> Vec<Issue> {
    let mut out = Vec::new();
    let mut put = |what: &str| out.push(Issue { at: at.to_string(), what: what.to_string() });
    if clean_path(p) != p.trim() || p.split('/').any(|s| s == "..") {
        put("path must be relative to the repo root, without `/`, `./` or `..`");
    }
    if !seen.insert(p.trim_end_matches('/').to_string()) {
        put("the same path is in another entry");
    }
    if main && !is_glob(p) && !root.join(clean_path(p)).exists() {
        put("a main entry names a path that does not exist");
    }
    out
}

/// Labels written in the file that another entry has too. Case does not matter (STRUCT-14).
fn label_issues(st: &Structure) -> impl Iterator<Item = Issue> + '_ {
    let mut seen = std::collections::BTreeSet::new();
    st.entries.iter().enumerate().filter_map(move |(i, e)| {
        let label = e.label.as_deref()?.trim();
        (!seen.insert(label.to_lowercase())).then(|| Issue {
            at: format!("entry {} ({})", i + 1, e.paths_label()),
            what: format!("the label \"{label}\" is in another entry"),
        })
    })
}

/// Fixes that need no judgement: other key names, `./` and `/` at the start of paths.
/// Keeps comments and layout. Returns the new text and what changed, or `None`.
pub fn quick_fix(text: &str) -> Option<(String, Vec<String>)> {
    let mut doc: toml_edit::DocumentMut = text.parse().ok()?;
    let mut changed = Vec::new();
    if let Some(arr) = doc.get_mut("entry").and_then(|v| v.as_array_of_tables_mut()) {
        for (i, t) in arr.iter_mut().enumerate() {
            for (from, to) in ALIASES {
                if t.contains_key(from) && !t.contains_key(to) {
                    // Rename the key; keep its comments and the value with its comments.
                    if let Some((old, item)) = t.remove_entry(from) {
                        let mut key = toml_edit::Key::new(to);
                        *key.leaf_decor_mut() = old.leaf_decor().clone();
                        t.insert_formatted(&key, item);
                        changed.push(format!("entry {}: `{from}` → `{to}`", i + 1));
                    }
                }
            }
            let path = t.get("path").and_then(|v| v.as_str()).map(str::to_string);
            if let Some(p) = path {
                let c = clean_path(&p);
                if c != p && !c.is_empty() {
                    set_keep_decor(t, "path", toml_edit::Value::from(c.clone()));
                    changed.push(format!("entry {}: path `{p}` → `{c}`", i + 1));
                }
            }
            // main = "true" (a string) → a boolean
            let m = t.get("main").and_then(|v| v.as_str()).map(|s| s.trim().to_lowercase());
            if let Some(m) = m.filter(|m| m == "true" || m == "false" || m == "yes" || m == "no") {
                set_keep_decor(t, "main", toml_edit::Value::from(m == "true" || m == "yes"));
                changed.push(format!("entry {}: main is a boolean", i + 1));
            }
        }
    }
    (!changed.is_empty()).then(|| (doc.to_string(), changed))
}

/// Replace a value and keep the comments around it.
fn set_keep_decor(t: &mut toml_edit::Table, key: &str, mut v: toml_edit::Value) {
    if let Some(old) = t.get(key).and_then(|i| i.as_value()) {
        *v.decor_mut() = old.decor().clone();
    }
    if let Some(slot) = t.get_mut(key) {
        *slot = toml_edit::Item::Value(v);
    }
}

/// The folders of the repo, two levels, for Claude. `.gitignore` respected.
pub fn folder_list(root: &Path) -> String {
    let walk = ignore::WalkBuilder::new(root).hidden(false).git_ignore(true).require_git(false).max_depth(Some(2)).build();
    let mut dirs: Vec<String> = walk
        .flatten()
        .filter(|e| e.file_type().is_some_and(|t| t.is_dir()) && e.depth() > 0)
        .filter_map(|e| e.path().strip_prefix(root).ok().map(|p| p.display().to_string()))
        .filter(|p| !p.starts_with(".git") || p == ".github")
        .collect();
    dirs.sort();
    dirs.iter().map(|d| format!("{d}/")).collect::<Vec<_>>().join("\n")
}

