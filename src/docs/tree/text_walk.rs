//! The text view and the worktree view walk the files on a thread of their own: they open each file
//! to see that it is text, and a large repository takes seconds on a cold disk. A switch of the view
//! and the scan every few seconds do not wait for it (ADR 0009, FILES-79).
use super::*;
use std::sync::mpsc::{Receiver, TryRecvError};

/// The text files and the folders of a walk, relative to the repo root.
type Found = (Vec<PathBuf>, Vec<PathBuf>);
/// The view and the worktree that a walk is for.
type For = (View, Option<PathBuf>);

/// The walk of text files that runs now.
#[derive(Default)]
pub(super) struct TextWalk {
    /// The view and the worktree that the walk is for, and the answer to come.
    pending: Option<(For, Receiver<Found>)>,
    /// The view and the worktree whose files the tree shows.
    shown: Option<For>,
}

impl Tree {
    /// Start the walk of the text files of the repo, or of the worktree in the worktree view. The
    /// tree keeps the rows of the same view until the answer comes.
    pub(super) fn walk_text(&mut self, root: &Path) {
        let key = (self.view, self.worktree.clone().filter(|_| self.view == View::Worktree));
        if self.walk.pending.as_ref().is_some_and(|(k, _)| *k == key) {
            return;
        }
        if self.walk.shown.as_ref() != Some(&key) {
            (self.files, self.dirs) = (Vec::new(), Vec::new());
        }
        self.drop_changes();
        self.deleted.clear();
        self.root = root.to_path_buf();
        let (tx, rx) = std::sync::mpsc::channel();
        let (root, wt) = (root.to_path_buf(), key.1.clone());
        std::thread::spawn(move || tx.send(walk(&root, wt.as_deref())));
        self.walk.pending = Some((key, rx));
    }

    /// Take the files of the walk, when they came. True when the rows changed.
    pub(super) fn poll_text(&mut self) -> bool {
        let Some((key, rx)) = &self.walk.pending else { return false };
        let found = match rx.try_recv() {
            Ok(f) => f,
            Err(TryRecvError::Empty) => return false,
            Err(TryRecvError::Disconnected) => (Vec::new(), Vec::new()),
        };
        let key = key.clone();
        self.walk.pending = None;
        let now = (self.view, self.worktree.clone().filter(|_| self.view == View::Worktree));
        if key != now {
            return false;
        }
        (self.files, self.dirs) = found;
        self.walk.shown = Some(key);
        self.scan_notes();
        self.scanned = Some(Instant::now());
        true
    }

    /// Scan, and wait for the walk of the text files: for tests and for a caller off the UI thread.
    pub fn scan_wait(&mut self, root: &Path) {
        self.scan(root);
        while self.walking() {
            std::thread::sleep(std::time::Duration::from_millis(2));
            self.poll_text();
        }
    }

    /// The walk of the text files runs now.
    pub fn walking(&self) -> bool {
        self.walk.pending.is_some()
    }
}

/// The text files and the folders under the root, or under the worktree `wt` in it; the paths
/// relative to the root.
fn walk(root: &Path, wt: Option<&Path>) -> Found {
    let dir = wt.map_or_else(|| root.to_path_buf(), |w| root.join(w));
    let walk = ignore::WalkBuilder::new(&dir).hidden(true).git_ignore(true).git_global(true).git_exclude(true).require_git(false).parents(wt.is_none()).build();
    let (mut files, mut dirs) = (Vec::new(), Vec::new());
    for e in walk.flatten() {
        let Ok(rel) = e.path().strip_prefix(root) else { continue };
        if rel.as_os_str().is_empty() {
            continue;
        }
        match e.file_type() {
            Some(t) if t.is_dir() && wt.is_none() => dirs.push(rel.to_path_buf()),
            Some(t) if t.is_file() && is_text(e.path()) => files.push(rel.to_path_buf()),
            _ => {}
        }
    }
    files.sort();
    (files, dirs)
}
