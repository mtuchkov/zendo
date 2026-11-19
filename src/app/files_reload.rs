//! The notes read again on a worker, and the look at the disk each second: after a save, a change
//! outside zendo, a tool call or a turn of the agent (IO-1, IO-6, ADR 0024).
use super::*;
use crate::docs::disk::{self, Looked, Reloaded};
use std::time::Duration;

pub(crate) use crate::docs::disk::StructureRead;

/// What a worker read again: the files opened from the tree and the open document that it looked
/// at, and what it found.
pub(crate) struct Loaded {
    asked: Vec<PathBuf>,
    current: Option<PathBuf>,
    found: Reloaded,
}

/// A look at the disk, from a worker.
pub(crate) type Look = Looked;

impl Files {
    /// Read the notes again, and the files opened outside the notes folder that still exist; on a
    /// worker (IO-1).
    pub fn reload(&mut self) {
        self.reload_reading(None);
    }

    /// Read the notes again, and the text of `open`, the file of the editor.
    pub(crate) fn reload_reading(&mut self, open: Option<PathBuf>) {
        if self.reloading.running() {
            self.reload_again = Some(self.reload_again.take().flatten().or(open));
            return;
        }
        let (store, extra, current) = (self.store.clone(), self.extra.clone(), self.current.clone());
        self.reloading.start(move || {
            let found = disk::reload(&store, &extra, current.as_deref(), open);
            Loaded { asked: extra, current, found }
        });
    }

    /// The worker read the notes again: they are the notes now. The text of the file of the editor,
    /// when it was asked for.
    pub(crate) fn poll_reload(&mut self) -> Option<Option<(PathBuf, String)>> {
        let Loaded { asked, current, found } = self.reloading.poll()?;
        if let Some(n) = found.notes {
            // The open document stays, also while it has no version in the session (FILES-40).
            let cur = self.current.clone();
            self.extra.retain(|p| found.alive.contains(p) || cur.as_ref() == Some(p) || !asked.contains(p));
            self.notes = n;
            self.fill_notes_view();
        }
        if self.current == current && !found.current_exists && self.current.as_ref().is_some_and(|p| !self.extra.contains(p)) {
            self.current = self.notes.first().map(|n| n.path.clone());
        }
        if let Some(open) = self.reload_again.take() {
            self.reload_reading(open);
        }
        Some(found.open)
    }

    /// A worker reads the notes now.
    pub fn reloading(&self) -> bool {
        self.reloading.running()
    }

    /// How long the reload waits for the worker.
    pub fn reload_waited(&self) -> Option<std::time::Duration> {
        self.reloading.waited()
    }

    /// At most once a second: start a look at the disk on a worker.
    pub(crate) fn look_at_disk(&mut self, root: &Path) {
        if self.looking.running() || self.disk_checked.is_some_and(|t| t.elapsed() < Duration::from_secs(1)) {
            return;
        }
        self.disk_checked = Some(Instant::now());
        let (root, dir, current, seen) = (root.to_path_buf(), self.store.dir.clone(), self.current.clone(), self.structure_seen);
        self.looking.start(move || disk::look(&root, &dir, current.as_deref(), seen));
    }

    /// The look at the disk came: the structure that changed, and whether the notes folder or the
    /// open document changed since the last look, but not at the first look.
    pub(crate) fn poll_look(&mut self) -> Option<(Option<StructureRead>, bool)> {
        let l = self.looking.poll()?;
        if l.structure.is_some() {
            self.structure_seen = l.structure_mtime;
        }
        // The open document came into being in the checkout of the session (FILES-44).
        if l.current_exists && self.peek.on.is_none() && self.note().is_none() {
            self.reload();
        }
        let first = self.disk.1.is_none() && self.disk.0 == 0;
        let changed = l.notes != self.disk && !first;
        self.disk = l.notes;
        Some((l.structure, changed))
    }

    /// A look or a reload runs now: for tests.
    pub fn disk_busy(&self) -> bool {
        self.reloading.running() || self.looking.running()
    }
}
