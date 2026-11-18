//! The file tree of the files: the app reads the tree, and changes it only through these methods.
use super::Files;
use crate::docs::structure::Structure;
use crate::docs::notes::Note;
use crate::docs::tree::{Kind, Row, Tree, View, worktree};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// How long the hint of the view strip shows after `v` (FILES-74).
const VIEW_HINT: std::time::Duration = std::time::Duration::from_millis(2500);

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
        self.fill_notes_view();
    }

    /// Scan, and wait for the walk of the text files (FILES-79): for tests.
    pub fn scan_wait(&mut self, root: &Path, notes: Option<PathBuf>, worktree: Option<PathBuf>) {
        self.scan(root, notes, worktree);
        while self.tree.walking() {
            std::thread::sleep(std::time::Duration::from_millis(2));
            self.poll_tree();
        }
    }

    /// The rows of the Notes view: the notes of the notes folder by age, under the heads Today,
    /// This week and Older (FILES-69).
    pub(super) fn fill_notes_view(&mut self) {
        let today = chrono::Local::now().date_naive();
        let head = |n: &Note| match (today - n.modified.date_naive()).num_days() {
            0 => "TODAY",
            1..=6 => "THIS WEEK",
            _ => "OLDER",
        };
        let mut rows = Vec::new();
        let mut last = "";
        for n in self.notes.iter().filter(|n| n.path.starts_with(&self.store.dir)) {
            if head(n) != last {
                last = head(n);
                rows.push(Row { rel: PathBuf::new(), name: last.to_string(), depth: 0, kind: Kind::Head, section: false });
            }
            let rel = n.path.strip_prefix(&self.store.root).unwrap_or(&n.path).to_path_buf();
            rows.push(Row { rel, name: n.title.clone(), depth: 0, kind: Kind::File, section: false });
        }
        self.tree.notes_view = rows;
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

    /// `v`: the next view of the tree, from its first row, and the hint of the strip for 2.5 s
    /// (FILES-4, FILES-74). Gives the name that the state keeps.
    pub fn next_view(&mut self) -> &'static str {
        self.tree.next_view();
        self.tree.hint_until = Some(std::time::Instant::now() + VIEW_HINT);
        (self.tree.sel, self.tree.scroll) = (0, 0);
        self.tree.view.name()
    }

    /// A click on a letter of the strip or a name of the hint (FILES-73).
    pub fn pick_view(&mut self, view: View) -> &'static str {
        self.tree.view = view;
        (self.tree.sel, self.tree.scroll) = (0, 0);
        self.first_selectable();
        view.name()
    }

    /// The hint of the strip shows now (FILES-74).
    pub fn view_hint(&self) -> bool {
        self.tree.hint_until.is_some_and(|t| std::time::Instant::now() < t)
    }

    /// After a change of the view: the selection is on a row that can have it.
    pub fn first_view_row(&mut self) {
        self.first_selectable();
    }

    /// A head of the Notes view is no place for the selection.
    pub(super) fn first_selectable(&mut self) {
        let rows = self.tree.rows();
        if rows.get(self.tree.sel).is_some_and(|r| r.kind == Kind::Head) {
            self.tree.sel = rows.iter().position(|r| r.kind != Kind::Head).unwrap_or(0);
        }
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
