//! The file tree of the files: the app reads the tree, and changes it only through these methods.
use super::Files;
use crate::docs::structure::Structure;
use crate::docs::tree::{Tree, View, worktree};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

impl Files {
    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    pub fn structure(&self) -> Option<&Structure> {
        self.tree.structure.as_ref()
    }

    /// Fill the tree from the repository `root`. `notes` is a notes folder outside it (FILES-25),
    /// `worktree` the worktree of the active agent session (FILES-32).
    pub fn scan(&mut self, root: &Path, notes: Option<PathBuf>, worktree: Option<PathBuf>) {
        self.tree.notes = notes;
        self.tree.worktree = worktree;
        self.tree.scan(root);
    }

    /// Outside a repository: fill the tree with the files under `under` only.
    pub fn scan_in(&mut self, root: &Path, under: &Path) {
        self.tree.scan_in(root, under);
    }

    /// The tree is older than its scan interval, or was never filled.
    pub fn tree_stale(&self) -> bool {
        self.tree.stale()
    }

    /// Take the answer of git for the worktree changed view, when it came (ADR 0009). True when the
    /// rows changed. The first rows of the view select the open file, as a scan does.
    pub fn poll_tree(&mut self) -> bool {
        let first = self.tree.files.is_empty();
        let changed = self.tree.poll();
        if changed && first {
            self.reveal_current();
        }
        changed
    }

    /// The next tick fills the tree again.
    pub fn scan_again(&mut self) {
        self.tree.scanned = None;
    }

    pub fn set_view(&mut self, view: View) {
        self.tree.view = view;
    }

    /// The next view of the tree, from its first row. Gives the name that the state keeps.
    pub fn next_view(&mut self) -> &'static str {
        self.tree.next_view();
        (self.tree.sel, self.tree.scroll) = (0, 0);
        self.tree.view.name()
    }

    /// The files that changed since the last visit that counted: they get a mark in the tree.
    pub fn mark_changed(&mut self, changed: BTreeSet<PathBuf>) {
        self.tree.changed = changed;
    }

    /// The selection goes to row `i` of the rows on screen.
    pub fn select_row(&mut self, i: usize) {
        self.tree.sel = i;
    }

    /// The wheel: the tree scrolls by `n` rows. The next draw keeps the scroll in the rows.
    pub fn scroll_tree(&mut self, up: bool, n: usize) {
        let t = &mut self.tree;
        t.scroll = if up { t.scroll.saturating_sub(n) } else { t.scroll + n };
    }

    /// The typing of a file name filter is on.
    pub fn filtering(&self) -> bool {
        self.tree.find.is_some()
    }

    pub fn close_filter(&mut self) {
        self.tree.find = None;
    }

    /// The tree opens the folders above `rel`, and selects its row.
    pub fn reveal(&mut self, rel: &Path) {
        self.tree.reveal(rel);
    }

    /// The tree shows the row of the open document.
    pub fn reveal_current(&mut self) {
        if let Some(rel) = self.current_in_tree() {
            self.tree.reveal(&rel);
        }
    }

    /// The row of the open document in the tree. Outside the worktree views, a file of a worktree
    /// has the row of its place in the repository.
    pub fn current_in_tree(&self) -> Option<PathBuf> {
        let rel = self.store.rel(self.current.as_ref()?);
        let text = rel.to_string_lossy();
        match worktree::split(&text) {
            Some((_, inner)) if !self.tree.view.is_worktree() => Some(PathBuf::from(inner)),
            _ => Some(rel.to_path_buf()),
        }
    }

    /// While the tree shows: when the open document changed since the tree showed it last, the
    /// tree opens its folders and selects it (FILES-58). Not before the first scan, nor while the
    /// name filter is on.
    pub fn follow_current(&mut self) {
        if self.tree.scanned.is_none() || self.tree.find.is_some() || self.followed == self.current {
            return;
        }
        self.followed = self.current.clone();
        self.reveal_current();
    }

    /// The next `follow_current` shows the open document, also when it did not change: the pane
    /// opens, the session changes, a file opens (FILES-58).
    pub fn follow_again(&mut self) {
        self.followed = None;
    }

    /// `[+]` in the head of the tree (FILES-57).
    pub fn expand_all(&mut self) {
        self.tree.expand_all();
    }

    /// `[-]` in the head of the tree (FILES-57).
    pub fn collapse_all(&mut self) {
        self.tree.collapse_all();
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
