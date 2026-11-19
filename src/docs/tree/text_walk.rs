//! Each view of the tree walks its files on a thread of its own: a large repository takes seconds on
//! a cold disk, and the text views open each file to see that it is text. A switch of the view and
//! the scan every few seconds do not wait for it (ADR 0009, ADR 0024, FILES-79, IO-1).
use super::*;
use std::sync::mpsc::{Receiver, TryRecvError};

/// What a walk found: the files and the folders, relative to the repo root, and the notes of the
/// notes folder, relative to it.
type Found = (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>);
/// The view, the worktree or folder, and the folder under the root that a walk is for.
type For = (View, Option<PathBuf>, PathBuf);

/// The walk that runs now.
#[derive(Default)]
pub(super) struct TextWalk {
    /// What the walk is for, and the answer to come.
    pending: Option<(For, Receiver<Found>)>,
    /// What the files of the tree are for.
    shown: Option<For>,
}

/// What a walk needs, owned, for its thread.
struct Job {
    root: PathBuf,
    under: PathBuf,
    view: View,
    wt: Option<PathBuf>,
    notes: Option<PathBuf>,
    docs: GlobMatcher,
}

impl Tree {
    /// The folder of the worktree view: the worktree of the session, or the main checkout for
    /// agent session 0 (FILES-81).
    fn walk_wt(&self) -> Option<PathBuf> {
        match self.view {
            View::Worktree => self.changes_dir(),
            _ => None,
        }
    }

    /// Start the walk of the files of the view under `under`. The tree keeps the rows of the same
    /// view until the answer comes (IO-6).
    pub(super) fn walk_view(&mut self, root: &Path, under: &Path) {
        let key = (self.view, self.walk_wt(), under.to_path_buf());
        if self.walk.pending.as_ref().is_some_and(|(k, _)| *k == key) {
            return;
        }
        if self.walk.shown.as_ref() != Some(&key) {
            (self.files, self.dirs) = (Vec::new(), Vec::new());
        }
        self.drop_changes();
        self.deleted.clear();
        self.root = root.to_path_buf();
        let job = Job { root: root.to_path_buf(), under: under.to_path_buf(), view: self.view, wt: key.1.clone(), notes: self.notes.clone(), docs: self.docs.clone() };
        let (tx, rx) = std::sync::mpsc::channel();
        let _ = std::thread::Builder::new().name("zendo-tree-walk".into()).spawn(move || {
            if tx.send(run(&job)).is_ok() {
                crate::worker::wake().notify_one();
            }
        });
        self.walk.pending = Some((key, rx));
    }

    /// Take the files of the walk, when they came. True when the rows changed.
    pub(super) fn poll_text(&mut self) -> bool {
        let Some((key, rx)) = &self.walk.pending else { return false };
        let found = match rx.try_recv() {
            Ok(f) => f,
            Err(TryRecvError::Empty) => return false,
            Err(TryRecvError::Disconnected) => (Vec::new(), Vec::new(), Vec::new()),
        };
        let key = key.clone();
        self.walk.pending = None;
        if (key.0, key.1.as_ref()) != (self.view, self.walk_wt().as_ref()) {
            return false;
        }
        (self.files, self.dirs, self.notes_files) = found;
        self.walk.shown = Some(key);
        self.scanned = Some(Instant::now());
        true
    }

    /// Scan, and wait for the walk: for tests and for a caller off the UI thread.
    pub fn scan_wait(&mut self, root: &Path) {
        self.scan(root);
        self.walk_wait();
    }

    /// Wait for the walk that runs: for tests and for a caller off the UI thread.
    pub fn walk_wait(&mut self) {
        while self.walking() {
            std::thread::sleep(std::time::Duration::from_millis(2));
            self.poll_text();
        }
    }

    /// The walk runs now.
    pub fn walking(&self) -> bool {
        self.walk.pending.is_some()
    }
}

/// The walk of a view, on its thread.
fn run(j: &Job) -> Found {
    let (files, dirs) = match j.view {
        View::Text => walk(&j.root, None),
        View::Worktree => j.wt.as_deref().map(|wt| walk(&j.root, Some(wt))).unwrap_or_default(),
        _ => (markdown(j), Vec::new()),
    };
    (files, dirs, j.notes.as_deref().map(notes_in).unwrap_or_default())
}

/// The markdown files of the docs, structure and notes views under `under`; outside a repo the walk
/// lists every `.md` of the notes folder.
fn markdown(j: &Job) -> Vec<PathBuf> {
    let all = j.under != j.root;
    let walk = ignore::WalkBuilder::new(&j.under).hidden(true).git_ignore(true).git_global(true).git_exclude(true).require_git(false).parents(true).build();
    let mut files: Vec<PathBuf> = walk
        .flatten()
        .filter(|e| e.file_type().is_some_and(|t| t.is_file()) && e.path().extension().is_some_and(|x| x == "md"))
        .filter_map(|e| e.path().strip_prefix(&j.root).ok().map(Path::to_path_buf))
        .filter(|rel| all || j.view != View::Docs || j.docs.is_match(rel))
        .collect();
    files.sort();
    files
}

/// The markdown files of the notes folder, relative to it; hidden files are skipped.
fn notes_in(dir: &Path) -> Vec<PathBuf> {
    let walk = ignore::WalkBuilder::new(dir).hidden(true).require_git(false).build();
    let mut files: Vec<PathBuf> = walk
        .flatten()
        .filter(|e| e.file_type().is_some_and(|t| t.is_file()) && e.path().extension().is_some_and(|x| x == "md"))
        .filter_map(|e| e.path().strip_prefix(dir).ok().map(Path::to_path_buf))
        .collect();
    files.sort();
    files
}

/// The text files and the folders under the root, or under the worktree `wt` in it; the paths
/// relative to the root.
fn walk(root: &Path, wt: Option<&Path>) -> (Vec<PathBuf>, Vec<PathBuf>) {
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
