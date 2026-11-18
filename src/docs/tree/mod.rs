//! The file tree: markdown files of the repo, `.gitignore` respected.
//! Views: the project structure (the main materials of `project-structure.toml`),
//! files under a `docs` folder (`**/docs/**`), or every text file and every folder;
//! and the worktree of the active agent session: every text file, or the files that differ from `main`.
//! Every folder starts closed (FILES-56).

use crate::docs::structure::Structure;
use globset::{Glob, GlobMatcher};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

pub const DOCS_GLOB: &str = "**/docs/**";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Dir { open: bool },
    File,
    /// A head of the Notes view: Today, This week, Older (FILES-69).
    Head,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    /// The main materials of `project-structure.toml`, in its order.
    Structure,
    /// Files under a `docs` folder.
    Docs,
    /// Every text file: markdown, code, config; and every folder, also an empty one (FILES-17).
    Text,
    /// Every text file of the worktree of the active agent session (FILES-26).
    Worktree,
    /// The files of that worktree that differ from the merge base with `main` (FILES-27).
    WorktreeChanged,
    /// The notes by age (FILES-69).
    Notes,
}

impl View {
    pub fn name(self) -> &'static str {
        match self {
            View::Structure => "structure",
            View::Docs => "docs",
            View::Text => "text",
            View::Worktree => "worktree",
            View::WorktreeChanged => "worktree changed",
            View::Notes => "notes",
        }
    }

    /// The letter of the view in the strip of the tree (FILES-70).
    pub fn letter(self) -> &'static str {
        match self {
            View::Structure => "S",
            View::Docs => "D",
            View::Text => "T",
            View::Worktree => "W",
            View::WorktreeChanged => "Δ",
            View::Notes => "N",
        }
    }

    pub fn from_name(s: &str) -> Option<View> {
        match s {
            "structure" => Some(View::Structure),
            "docs" => Some(View::Docs),
            "text" => Some(View::Text),
            "worktree" => Some(View::Worktree),
            "worktree changed" => Some(View::WorktreeChanged),
            "notes" => Some(View::Notes),
            _ => None,
        }
    }
    /// A view of the worktree of the active agent session.
    pub fn is_worktree(self) -> bool {
        matches!(self, View::Worktree | View::WorktreeChanged)
    }
}

#[derive(Clone, Debug)]
pub struct Row {
    /// Repo-relative path of the file or folder.
    pub rel: PathBuf,
    pub name: String,
    pub depth: usize,
    pub kind: Kind,
    /// A main material from the structure file.
    pub section: bool,
}

pub struct Tree {
    pub view: View,
    /// The project structure, when the repo has one.
    pub structure: Option<Structure>,
    pub files: Vec<PathBuf>,
    /// The repo root of the last scan.
    pub root: PathBuf,
    /// The folders and main materials that are open; each one starts closed (FILES-56).
    pub opened: BTreeSet<PathBuf>,
    pub sel: usize,
    pub scroll: usize,
    pub scanned: Option<Instant>,
    /// Files changed since your last visit that counted (Δ).
    pub changed: BTreeSet<PathBuf>,
    /// The folders of the repository, in the text view.
    pub dirs: Vec<PathBuf>,
    /// The rows of the Notes view: the heads and the notes by age (FILES-69).
    pub notes_view: Vec<Row>,
    /// Until when the hint of the view strip shows (FILES-74).
    pub hint_until: Option<Instant>,
    /// The file name filter (`` ` `` in the tree): while it is on, the tree is a flat list of matches.
    pub find: Option<String>,
    /// `[.*]` of the filter: the text is a regular expression for the path (FILES-67).
    pub find_regex: bool,
    /// The notes folder in the notes repo, while the notes repo is on (FILES-25).
    pub notes: Option<PathBuf>,
    /// The markdown files of the notes folder, relative to it.
    pub notes_files: Vec<PathBuf>,
    /// The worktree of the active agent session, from the repo root, when it exists.
    pub worktree: Option<PathBuf>,
    /// The deleted files in the worktree changed view (FILES-35).
    pub deleted: BTreeSet<PathBuf>,
    /// Git for the worktree changed view (ADR 0009).
    git: changes::Changes,
    docs: GlobMatcher,
}

impl Default for Tree {
    fn default() -> Self {
        Self {
            view: View::Docs,
            structure: None,
            files: Vec::new(),
            root: PathBuf::new(),
            opened: BTreeSet::new(),
            sel: 0,
            scroll: 0,
            scanned: None,
            changed: BTreeSet::new(),
            dirs: Vec::new(),
            find: None,
            find_regex: false,
            notes_view: Vec::new(),
            hint_until: None,
            notes: None,
            notes_files: Vec::new(),
            worktree: None,
            deleted: BTreeSet::new(),
            git: changes::Changes::default(),
            docs: Glob::new(DOCS_GLOB).expect("valid glob").compile_matcher(),
        }
    }
}

#[derive(Default)]
struct Node {
    dirs: BTreeMap<String, Node>,
    files: Vec<String>,
}

mod changes;
mod fold;
mod notes;
mod scan;
mod sections;
pub mod worktree;
pub use scan::is_text;
pub use sections::section_id;

impl Tree {
    pub fn filter_label(&self) -> &'static str {
        match self.view {
            View::Structure => "structure",
            View::Docs => DOCS_GLOB,
            View::Text => "text files",
            View::Worktree => "worktree",
            View::WorktreeChanged => "worktree Δ",
            View::Notes => "notes",
        }
    }

    /// The structure view, when there is a structure file with main materials.
    pub fn structured(&self) -> Option<&Structure> {
        self.structure.as_ref().filter(|s| self.view == View::Structure && s.main().next().is_some())
    }

    /// `v`: structure (if there is one) → docs → text → worktree → worktree changed → notes → …;
    /// the worktree views only while the active agent session has a worktree (FILES-4, FILES-31).
    pub fn next_view(&mut self) {
        let views = self.views();
        let at = views.iter().position(|v| *v == self.view).map_or(0, |i| i + 1);
        self.view = views[at % views.len()];
    }

    /// The views that the strip and the `v` cycle show now, in their order (FILES-4, FILES-72).
    pub fn views(&self) -> Vec<View> {
        let has = self.structure.as_ref().is_some_and(|s| s.main().next().is_some());
        let wt = self.worktree.is_some();
        [(View::Structure, has), (View::Docs, true), (View::Text, true), (View::Worktree, wt), (View::WorktreeChanged, wt), (View::Notes, true)]
            .into_iter()
            .filter_map(|(v, on)| on.then_some(v))
            .collect()
    }

    /// Visible rows: folders first, then files; folder chains with one child are joined (`docs/notes`).
    /// In the structure view, the main materials come first, in the order of the file.
    pub fn rows(&self) -> Vec<Row> {
        if let Some(q) = self.find.as_deref().filter(|q| !q.trim().is_empty()) {
            return self.found(q);
        }
        if self.view.is_worktree() {
            return self.worktree_rows();
        }
        if self.view == View::Notes {
            return self.notes_view.clone();
        }
        let mut out = self.notes_rows();
        match self.structured() {
            Some(st) => out.extend(self.section_rows(st)),
            None => out.extend(self.tree_rows(&self.files, &self.dirs, Path::new(""), 0)),
        }
        out
    }

    fn tree_rows(&self, files: &[PathBuf], dirs: &[PathBuf], at: &Path, depth: usize) -> Vec<Row> {
        let mut root = Node::default();
        for d in dirs {
            d.iter().fold(&mut root, |node, part| node.dirs.entry(part.to_string_lossy().into_owned()).or_default());
        }
        for f in files {
            let mut node = &mut root;
            let parts: Vec<String> = f.iter().map(|c| c.to_string_lossy().into_owned()).collect();
            let (name, dirs) = parts.split_last().expect("a file has a name");
            for d in dirs {
                node = node.dirs.entry(d.clone()).or_default();
            }
            node.files.push(name.clone());
        }
        let mut out = Vec::new();
        self.flatten(&root, at.to_path_buf(), depth, &mut out);
        out
    }

    fn flatten(&self, node: &Node, at: PathBuf, depth: usize, out: &mut Vec<Row>) {
        for (name, child) in &node.dirs {
            // Join folders that hold only one folder.
            let mut name = name.clone();
            let mut rel = at.join(&name);
            let mut child = child;
            while child.files.is_empty() && child.dirs.len() == 1 {
                let (n, c) = child.dirs.iter().next().expect("one child");
                name = format!("{name}/{n}");
                rel = rel.join(n);
                child = c;
            }
            let open = self.opened.contains(&rel);
            out.push(Row { rel: rel.clone(), name, depth, kind: Kind::Dir { open }, section: false });
            if open {
                self.flatten(child, rel, depth + 1, out);
            }
        }
        for f in &node.files {
            out.push(Row { rel: at.join(f), name: f.clone(), depth, kind: Kind::File, section: false });
        }
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
