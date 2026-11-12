//! The file tree: markdown files of the repo, `.gitignore` respected.
//! Views: the project structure (the main materials of `project-structure.toml`),
//! files under a `docs` folder (`**/docs/**`), or every `.md` file.

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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    /// The main materials of `project-structure.toml`, in its order.
    Structure,
    /// Files under a `docs` folder.
    Docs,
    /// Every `.md` file.
    All,
    /// Every folder, also an empty one or one without `.md`, and every `.md` file.
    Folders,
}

impl View {
    pub fn name(self) -> &'static str {
        match self {
            View::Structure => "structure",
            View::Docs => "docs",
            View::All => "all",
            View::Folders => "folders",
        }
    }
    pub fn from_name(s: &str) -> Option<View> {
        match s {
            "structure" => Some(View::Structure),
            "docs" => Some(View::Docs),
            "all" => Some(View::All),
            "folders" => Some(View::Folders),
            _ => None,
        }
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
    pub closed: BTreeSet<PathBuf>,
    /// Main materials that are open (they start closed).
    pub opened: BTreeSet<PathBuf>,
    pub sel: usize,
    pub scroll: usize,
    pub scanned: Option<Instant>,
    /// Files changed since your last visit that counted (Δ).
    pub changed: BTreeSet<PathBuf>,
    /// The folders of the repository, in the folders view.
    pub dirs: Vec<PathBuf>,
    /// The file name filter (`/` in the tree): while it is on, the tree is a flat list of matches.
    pub find: Option<String>,
    docs: GlobMatcher,
}

impl Default for Tree {
    fn default() -> Self {
        Self {
            view: View::Docs,
            structure: None,
            files: Vec::new(),
            root: PathBuf::new(),
            closed: BTreeSet::new(),
            opened: BTreeSet::new(),
            sel: 0,
            scroll: 0,
            scanned: None,
            changed: BTreeSet::new(),
            dirs: Vec::new(),
            find: None,
            docs: Glob::new(DOCS_GLOB).expect("valid glob").compile_matcher(),
        }
    }
}

#[derive(Default)]
struct Node {
    dirs: BTreeMap<String, Node>,
    files: Vec<String>,
}

mod scan;

impl Tree {
    pub fn filter_label(&self) -> &'static str {
        match self.view {
            View::Structure => "structure",
            View::Docs => DOCS_GLOB,
            View::All => "*.md",
            View::Folders => "folders",
        }
    }

    /// The structure view, when there is a structure file with main materials.
    pub fn structured(&self) -> Option<&Structure> {
        self.structure.as_ref().filter(|s| self.view == View::Structure && s.main().next().is_some())
    }

    /// `v`: structure (if there is one) → docs → all → folders → …
    pub fn next_view(&mut self) {
        let has = self.structure.as_ref().is_some_and(|s| s.main().next().is_some());
        self.view = match self.view {
            View::Structure => View::Docs,
            View::Docs => View::All,
            View::All => View::Folders,
            View::Folders if has => View::Structure,
            View::Folders => View::Docs,
        };
    }

    /// Visible rows: folders first, then files; folder chains with one child are joined (`docs/notes`).
    /// In the structure view, the main materials come first, in the order of the file.
    pub fn rows(&self) -> Vec<Row> {
        if let Some(q) = self.find.as_deref().filter(|q| !q.trim().is_empty()) {
            return self.found(q);
        }
        if let Some(st) = self.structured() {
            return self.section_rows(st);
        }
        self.tree_rows(&self.files, &self.dirs, Path::new(""), 0)
    }

    fn section_rows(&self, st: &Structure) -> Vec<Row> {
        let mut out = Vec::new();
        for e in st.main() {
            let rel = PathBuf::from(e.path.trim_end_matches('/'));
            let inside: Vec<PathBuf> = self.files.iter().filter(|f| f.starts_with(&rel) && **f != rel).cloned().collect();
            // A main material that is a file, of any kind (e.g. a .toml).
            if self.files.contains(&rel) || self.root.join(&rel).is_file() || rel.extension().is_some_and(|x| x == "md") {
                out.push(Row { rel, name: e.label(), depth: 0, kind: Kind::File, section: true });
                continue;
            }
            let open = self.opened.contains(&rel);
            out.push(Row { rel: rel.clone(), name: e.label(), depth: 0, kind: Kind::Dir { open }, section: true });
            if open {
                let stripped: Vec<PathBuf> = inside.iter().filter_map(|f| f.strip_prefix(&rel).ok().map(Path::to_path_buf)).collect();
                out.extend(self.tree_rows(&stripped, &[], &rel, 1));
            }
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
            let open = !self.closed.contains(&rel);
            out.push(Row { rel: rel.clone(), name, depth, kind: Kind::Dir { open }, section: false });
            if open {
                self.flatten(child, rel, depth + 1, out);
            }
        }
        for f in &node.files {
            out.push(Row { rel: at.join(f), name: f.clone(), depth, kind: Kind::File, section: false });
        }
    }

    /// Open every folder on the way to `rel` and select it.
    pub fn reveal(&mut self, rel: &Path) {
        // In the structure view, open the most specific main material that holds the file.
        let section = self.structured().and_then(|st| {
            st.main().map(|e| PathBuf::from(e.path.trim_end_matches('/'))).filter(|p| rel.starts_with(p) && p != rel).max_by_key(|p| p.components().count())
        });
        if let Some(p) = section {
            self.opened.insert(p);
        }
        let mut p = rel.parent();
        while let Some(d) = p.filter(|d| !d.as_os_str().is_empty()) {
            self.closed.remove(d);
            p = d.parent();
        }
        if let Some(i) = self.rows().iter().position(|r| r.rel == rel) {
            self.sel = i;
        }
    }

    pub fn toggle(&mut self, rel: &Path) {
        let section = self.structured().is_some_and(|st| st.main().any(|e| Path::new(e.path.trim_end_matches('/')) == rel));
        if section {
            if !self.opened.remove(rel) {
                self.opened.insert(rel.to_path_buf());
            }
            return;
        }
        if !self.closed.remove(rel) {
            self.closed.insert(rel.to_path_buf());
        }
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
