//! The files: which documents there are and which one is open, the file tree, the find panel, and
//! the questions about a file (ADR 0008). The files do not see the app: a change in the chat, the
//! sessions or the focus goes back as an `Effect`. The file actions are in `files_ops.rs` and
//! `files_new.rs`, the keys of the tree in `tree_keys.rs`.
use super::version::Peek;
use super::{Effect, FilePrompt, Find, WorktreeUi};
use crate::docs::notes::{self, Note, Store};
use crate::docs::structure::Structure;
use crate::docs::tree::Tree;
use crate::setup::sync::Syncer;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime};

#[derive(smart_default::SmartDefault)]
pub struct Files {
    store: Store,
    notes: Vec<Note>,
    current: Option<PathBuf>,
    /// Files opened from the tree outside the notes folder.
    extra: Vec<PathBuf>,
    pub(super) tree: Tree,
    /// `alt-/`: the find panel and the recent files (FIND-1).
    pub(super) find: Find,
    /// `delete`, `f2` or `m` on a file: the question in the status bar.
    prompt: Option<FilePrompt>,
    /// The worktree views: the popup before a file action, the recap timer.
    pub(super) worktree: WorktreeUi,
    /// Why `project-structure.toml` does not parse, if it does not.
    structure_error: Option<String>,
    structure_seen: Option<SystemTime>,
    /// Notes folder fingerprint (count and newest change), checked every second.
    disk: (usize, Option<SystemTime>),
    /// When the disk was checked last; none: the next tick checks it.
    #[default(Some(Instant::now()))]
    disk_checked: Option<Instant>,
    /// `m` or `u` held in read mode: the version of `main` or of the disk (FILES-45, FILES-47).
    pub(super) peek: Peek,
    /// The git branch of the repository at the start.
    branch: Option<String>,
    /// Commits and pushes the notes repo in the background.
    syncer: Option<Syncer>,
}

impl Files {
    /// The notes of the store; the newest one is the open document.
    pub fn start(store: Store) -> anyhow::Result<Self> {
        let notes = store.load_all()?;
        let current = notes.first().map(|n| n.path.clone());
        Ok(Self { store, notes, current, ..Self::default() })
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    /// The open document.
    pub fn current(&self) -> Option<&PathBuf> {
        self.current.as_ref()
    }

    pub fn current_path(&self) -> Option<&Path> {
        self.current.as_deref()
    }

    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    pub fn tree_mut(&mut self) -> &mut Tree {
        &mut self.tree
    }

    pub fn find(&self) -> &Find {
        &self.find
    }

    pub fn find_mut(&mut self) -> &mut Find {
        &mut self.find
    }

    pub fn prompt(&self) -> Option<&FilePrompt> {
        self.prompt.as_ref()
    }

    /// Open a question about a file.
    pub fn ask(&mut self, prompt: FilePrompt) {
        self.prompt = Some(prompt);
    }

    pub fn worktree(&self) -> &WorktreeUi {
        &self.worktree
    }

    pub fn worktree_mut(&mut self) -> &mut WorktreeUi {
        &mut self.worktree
    }

    pub fn peek(&self) -> &Peek {
        &self.peek
    }

    pub fn peek_mut(&mut self) -> &mut Peek {
        &mut self.peek
    }

    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    pub fn set_branch(&mut self, branch: Option<String>) {
        self.branch = branch;
    }

    pub fn set_syncer(&mut self, syncer: Option<Syncer>) {
        self.syncer = syncer;
    }

    /// The syncer, for its last run at the quit.
    pub fn take_syncer(&mut self) -> Option<Syncer> {
        self.syncer.take()
    }

    /// What the notes sync wants the user to know, if anything.
    pub fn sync_label(&self) -> Option<String> {
        self.syncer.as_ref().and_then(|s| s.state().label())
    }

    pub fn structure(&self) -> Option<&Structure> {
        self.tree.structure.as_ref()
    }

    pub fn structure_error(&self) -> Option<&str> {
        self.structure_error.as_deref()
    }

    /// The path of a file from the root of the store, as text.
    pub fn rel(&self, p: &Path) -> String {
        self.store.rel(p).display().to_string()
    }

    /// The document on the screen: the version that a held key shows, else the open one.
    pub fn note(&self) -> Option<&Note> {
        if self.peek.on.is_some() {
            return self.peek.note.as_ref();
        }
        let p = self.current.as_ref()?;
        self.notes.iter().find(|n| &n.path == p)
    }

    pub fn note_index(&self) -> Option<usize> {
        let p = self.current.as_ref()?;
        self.notes.iter().position(|n| &n.path == p)
    }

    /// Read the notes again, and the files opened outside the notes folder that still exist.
    pub fn reload(&mut self) {
        if let Ok(mut n) = self.store.load_all() {
            // The open document stays, also while it has no version in the session (FILES-40).
            let cur = self.current.clone();
            self.extra.retain(|p| p.exists() || cur.as_ref() == Some(p));
            let more: Vec<Note> = self.extra.iter().filter(|p| !n.iter().any(|x| &x.path == *p)).filter_map(|p| notes::load(p).ok()).collect();
            n.extend(more);
            n.sort_by_key(|b| std::cmp::Reverse(b.modified));
            self.notes = n;
        }
        if self.current.as_ref().is_some_and(|p| !p.exists() && !self.extra.contains(p)) {
            self.current = self.notes.first().map(|n| n.path.clone());
        }
    }

    /// Note `i` of the list is the open document.
    pub fn show_note(&mut self, i: usize) {
        self.current = Some(self.notes[i].path.clone());
    }

    /// The file is the open document; a file outside the notes is read.
    pub fn open(&mut self, path: PathBuf) {
        self.current = Some(path.clone());
        if !self.notes.iter().any(|n| n.path == path) && !self.extra.contains(&path) {
            self.extra.push(path);
            self.reload();
        }
    }

    /// The editor saved the file: it is the open document. A note in a worktree is not in the
    /// notes of the store (FILES-49).
    pub fn saved(&mut self, path: PathBuf) {
        if !path.starts_with(&self.store.dir) && !self.extra.contains(&path) {
            self.extra.push(path.clone());
        }
        self.current = Some(path);
        self.reload();
    }

    /// The float and the screen behind it each have an open document, or none.
    pub(crate) fn swap_current(&mut self, other: &mut Option<PathBuf>) {
        std::mem::swap(&mut self.current, other);
    }

    /// The tree shows the row of the open document.
    pub fn reveal_current(&mut self) {
        if let Some(p) = self.current.clone() {
            let rel = self.store.rel(&p).to_path_buf();
            self.tree.reveal(&rel);
        }
    }

    /// The draw of the tree: the selection stays in the `rows` rows, and in the view of `h` rows.
    pub fn fit_tree(&mut self, rows: usize, h: usize) {
        let t = &mut self.tree;
        t.sel = t.sel.min(rows.saturating_sub(1));
        if t.sel < t.scroll {
            t.scroll = t.sel;
        } else if t.sel >= t.scroll + h {
            t.scroll = t.sel + 1 - h;
        }
        t.scroll = t.scroll.min(rows.saturating_sub(h));
    }
}

#[path = "files_disk.rs"]
mod disk;
#[path = "files_new.rs"]
mod new;
#[path = "files_ops.rs"]
pub(crate) mod ops;
#[path = "tree_keys.rs"]
pub(crate) mod tree_keys;

#[cfg(test)]
#[path = "files_state_tests.rs"]
mod tests;
