//! `project-structure.toml`: the project's folders and key files, each with a description for
//! people and instructions for Claude. Entries with `main = true` are the default file tree.

use globset::Glob;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;

pub const FILE: &str = "project-structure.toml";

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Structure {
    #[serde(default)]
    pub project: Project,
    /// Free-form facts about the file itself, for example a date or a note.
    #[serde(default)]
    pub meta: BTreeMap<String, toml::Value>,
    #[serde(default, rename = "entry")]
    pub entries: Vec<Entry>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Project {
    pub name: Option<String>,
    pub about: Option<String>,
    /// Rules for Claude that hold in the whole repository.
    pub claude: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Entry {
    /// A folder or a file, relative to the repo root. `*` and `**` are allowed.
    pub path: String,
    /// A short name for the file tree. Default: the last part of the path.
    pub label: Option<String>,
    /// What this is, for people.
    pub about: Option<String>,
    /// What Claude must do or not do here.
    pub claude: Option<String>,
    /// A main material: shown at the top level of the default file tree.
    #[serde(default)]
    pub main: bool,
    /// The word limit of the documents here, in place of 1,000. 0: no limit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub words: Option<usize>,
    /// The template of new documents in this folder, relative to the repo root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}

mod check;
pub use check::*;

impl Entry {
    pub fn label(&self) -> String {
        self.label.clone().unwrap_or_else(|| self.path.trim_end_matches('/').rsplit('/').next().unwrap_or(&self.path).to_string())
    }

    pub fn is_glob(&self) -> bool {
        self.path.contains('*') || self.path.contains('?') || self.path.contains('[')
    }

    /// The entry covers `rel`: the same path, a path inside it, or a glob match.
    pub fn covers(&self, rel: &Path) -> bool {
        let p = self.path.trim_end_matches('/');
        if self.is_glob() {
            let Ok(g) = Glob::new(p) else { return false };
            let m = g.compile_matcher();
            return rel.ancestors().any(|a| m.is_match(a));
        }
        rel == Path::new(p) || rel.starts_with(p)
    }

    fn depth(&self) -> usize {
        self.path.trim_end_matches('/').split('/').count()
    }
}

impl Structure {
    /// `None` when the repo has no structure file; `Err` with the reason when it does not parse.
    pub fn load(root: &Path) -> Option<Result<Structure, String>> {
        let text = std::fs::read_to_string(root.join(FILE)).ok()?;
        Some(toml::from_str(&text).map_err(|e| e.to_string()))
    }

    pub fn main(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(|e| e.main && !e.is_glob())
    }

    /// Entries that cover `rel`, from the most general to the most specific.
    pub fn covering(&self, rel: &Path) -> Vec<&Entry> {
        let mut v: Vec<&Entry> = self.entries.iter().filter(|e| e.covers(rel)).collect();
        v.sort_by_key(|e| e.depth());
        v
    }

    /// The word limit for `rel` from the most specific entry that sets one. None: the default.
    pub fn words_for(&self, rel: &Path) -> Option<usize> {
        self.covering(rel).into_iter().rev().find_map(|e| e.words)
    }

    /// The template of a new document at `rel`: from the most specific entry that names one.
    pub fn template_for(&self, rel: &Path) -> Option<&str> {
        self.covering(rel).into_iter().rev().find_map(|e| e.template.as_deref())
    }

    /// Every template, with the folder of the entry that names it.
    pub fn templates(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries.iter().filter_map(|e| e.template.as_deref().map(|t| (t, e.path.as_str())))
    }

    /// The description of the most specific entry for `rel`.
    pub fn about(&self, rel: &Path) -> Option<&str> {
        self.covering(rel).into_iter().rev().find_map(|e| e.about.as_deref())
    }

    /// The Claude rules for a file: project rules, then each covering entry.
    pub fn rules_for(&self, rel: &Path) -> String {
        let mut out = String::new();
        if let Some(c) = self.project.claude.as_deref().filter(|c| !c.trim().is_empty()) {
            out.push_str("Project rules:\n");
            out.push_str(c.trim());
            out.push('\n');
        }
        let here: Vec<&Entry> = self.covering(rel).into_iter().filter(|e| e.claude.as_deref().is_some_and(|c| !c.trim().is_empty())).collect();
        if !here.is_empty() {
            out.push_str(&format!("Rules for {} (from {FILE}):\n", rel.display()));
            for e in here {
                out.push_str(&format!("- {}: {}\n", e.path, e.claude.as_deref().unwrap_or("").trim().replace('\n', " ")));
            }
        }
        out
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
