//! The worktree changed view reads git on a thread of its own: a switch of the agent session and the
//! scan every few seconds do not wait for git (ADR 0009). An error of git shows in the view.
use super::worktree::{Change, changes};
use super::*;
use std::sync::mpsc::{Receiver, TryRecvError};

type Answer = Result<Vec<Change>, String>;

/// The state of git for the worktree changed view.
#[derive(Default)]
pub(super) struct Changes {
    /// The worktree that git reads now, and the answer to come.
    pending: Option<(PathBuf, Receiver<Answer>)>,
    /// The worktree whose changes or error the view shows.
    shown: Option<PathBuf>,
    /// The error of git for that worktree.
    error: Option<String>,
    /// The file to select when the answer comes: a reveal while git reads.
    reveal: Option<PathBuf>,
}

impl Tree {
    /// Start git on a thread for the changes of the worktree `wt`, from the repo root. The view keeps
    /// the rows of the same worktree until the answer comes.
    pub(super) fn scan_changes(&mut self, root: &Path, wt: PathBuf) {
        if self.git.shown.as_ref() != Some(&wt) {
            self.files.clear();
            self.deleted.clear();
            self.git = Changes::default();
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let dir = root.join(&wt);
        std::thread::spawn(move || tx.send(changes(&dir)));
        self.git.pending = Some((wt, rx));
    }

    /// Forget git: the view is another one now, or the session has no worktree.
    pub(super) fn drop_changes(&mut self) {
        self.git = Changes::default();
    }

    /// Take the answer of git, when it came. True when the rows changed.
    pub fn poll(&mut self) -> bool {
        if self.poll_text() {
            return true;
        }
        let Some((wt, rx)) = &self.git.pending else { return false };
        let answer = match rx.try_recv() {
            Ok(a) => a,
            Err(TryRecvError::Empty) => return false,
            Err(TryRecvError::Disconnected) => Err("git stopped without an answer".into()),
        };
        let wt = wt.clone();
        self.git.pending = None;
        if self.view != View::WorktreeChanged || self.worktree.as_ref() != Some(&wt) {
            return false;
        }
        self.files.clear();
        self.deleted.clear();
        self.git.error = answer.as_ref().err().cloned();
        for c in answer.unwrap_or_default() {
            let rel = wt.join(&c.rel);
            if c.deleted {
                self.deleted.insert(rel.clone());
            }
            self.files.push(rel);
        }
        self.git.shown = Some(wt);
        if let Some(rel) = self.git.reveal.take() {
            self.reveal(&rel);
        }
        true
    }

    /// While git reads, the answer selects `rel` too: the rows of now may not hold it yet.
    pub(super) fn reveal_later(&mut self, rel: &Path) {
        if self.scanning() {
            self.git.reveal = Some(rel.to_path_buf());
        }
    }

    /// Git reads the changes of the worktree now.
    pub fn scanning(&self) -> bool {
        self.git.pending.is_some()
    }

    /// The error of git in the worktree changed view.
    pub fn error(&self) -> Option<&str> {
        self.git.error.as_deref()
    }
}
