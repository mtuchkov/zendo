//! The notes read again on a worker, and the look at the disk each second: after a save, a change
//! outside zendo, a tool call or a turn of the agent (IO-1, IO-6, ADR 0024).
use super::*;
use crate::docs::structure::{self, Structure};
use std::time::Duration;

/// The structure that a worker read again, and the number of things doctor does not know.
pub(crate) type StructureRead = (Option<Result<Structure, String>>, Option<usize>);

/// What a worker read again.
pub(crate) struct Loaded {
    /// The notes and the files opened from the tree, newest first; None when the folder cannot be
    /// read.
    notes: Option<Vec<Note>>,
    /// The files opened from the tree that the worker looked at, and those of them that exist.
    asked: Vec<PathBuf>,
    alive: Vec<PathBuf>,
    /// The open document that the worker looked at, and whether it exists.
    current: Option<PathBuf>,
    current_exists: bool,
    /// The file of the editor and its text, when the caller asked for it (UNDO-6, UNDO-13).
    open: Option<(PathBuf, String)>,
}

/// What a look at the disk saw: `project-structure.toml`, and the notes folder.
pub(crate) struct Look {
    structure_mtime: Option<SystemTime>,
    /// The structure, read again when it changed.
    structure: Option<StructureRead>,
    notes: (usize, Option<SystemTime>),
    current_exists: bool,
}

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
        self.reloading.start(move || load(&store, extra, current, open));
    }

    /// The worker read the notes again: they are the notes now. The text of the file of the editor,
    /// when it was asked for.
    pub(crate) fn poll_reload(&mut self) -> Option<Option<(PathBuf, String)>> {
        let l = self.reloading.poll()?;
        if let Some(n) = l.notes {
            // The open document stays, also while it has no version in the session (FILES-40).
            let cur = self.current.clone();
            self.extra.retain(|p| l.alive.contains(p) || cur.as_ref() == Some(p) || !l.asked.contains(p));
            self.notes = n;
            self.fill_notes_view();
        }
        if self.current == l.current && !l.current_exists && self.current.as_ref().is_some_and(|p| !self.extra.contains(p)) {
            self.current = self.notes.first().map(|n| n.path.clone());
        }
        if let Some(open) = self.reload_again.take() {
            self.reload_reading(open);
        }
        Some(l.open)
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
        self.looking.start(move || look(&root, &dir, current.as_deref(), seen));
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

fn load(store: &Store, extra: Vec<PathBuf>, current: Option<PathBuf>, open: Option<PathBuf>) -> Loaded {
    let alive: Vec<PathBuf> = extra.iter().filter(|p| p.exists()).cloned().collect();
    let notes = store.load_all().ok().map(|mut n| {
        let more: Vec<Note> = alive.iter().filter(|p| !n.iter().any(|x| &x.path == *p)).filter_map(|p| notes::load(p).ok()).collect();
        n.extend(more);
        n.sort_by_key(|b| std::cmp::Reverse(b.modified));
        n
    });
    let current_exists = current.as_ref().is_some_and(|p| p.exists());
    let open = open.and_then(|p| std::fs::read_to_string(&p).ok().map(|b| (p, b)));
    Loaded { notes, asked: extra, alive, current, current_exists, open }
}

fn look(root: &Path, dir: &Path, current: Option<&Path>, seen: Option<SystemTime>) -> Look {
    let mtime = |p: &Path| std::fs::metadata(p).and_then(|m| m.modified()).ok();
    let structure_mtime = mtime(&root.join(structure::FILE));
    let structure = (structure_mtime != seen).then(|| {
        let issues = match crate::setup::doctor::structure_problem(root) {
            Some(crate::setup::doctor::Problem::Issues(list)) => Some(list.len()),
            _ => None,
        };
        (Structure::load(root), issues)
    });
    let md: Vec<Option<SystemTime>> = std::fs::read_dir(dir)
        .map(|d| d.flatten().filter(|e| e.path().extension().is_some_and(|x| x == "md")).map(|e| e.metadata().and_then(|m| m.modified()).ok()).collect())
        .unwrap_or_default();
    let open = current.and_then(mtime);
    let notes = (md.len(), md.iter().copied().chain([open]).max().flatten());
    Look { structure_mtime, structure, notes, current_exists: current.is_some_and(Path::exists) }
}
