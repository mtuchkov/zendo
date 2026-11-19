//! The disk work that the workers of the app do for the documents: the file actions of the tree,
//! the folders for a completion, the notes read again, and the look at the disk each second. The UI
//! thread never calls these; the app runs them on `zendo::worker` (IO-1, ADR 0024).
use super::notes::{self, Note, Store};
use super::structure::{self, Structure};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// The structure read again, and the number of things that doctor does not know.
pub type StructureRead = (Option<Result<Structure, String>>, Option<usize>);

/// How a rename or a move ended.
pub enum Relocated {
    Done,
    /// A file has the name of the target.
    Exists,
    Failed(String),
}

/// Rename or move `old` to `target`; a folder that is not there yet is made.
pub fn relocate(old: &Path, target: &Path) -> Relocated {
    if target.exists() {
        return Relocated::Exists;
    }
    match target.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|()| std::fs::rename(old, target)) {
        Ok(()) => Relocated::Done,
        Err(e) => Relocated::Failed(e.to_string()),
    }
}

/// Remove the file `path`.
pub fn delete(path: &Path) -> std::io::Result<()> {
    std::fs::remove_file(path)
}

/// Make the file `path` with `body` and the text of `template`, when it is not there: `first` makes
/// the text. `Ok(false)`: a file has that name.
pub fn make_file(path: &Path, template: Option<&Path>, first: impl FnOnce(Option<String>) -> String) -> std::io::Result<bool> {
    if path.exists() {
        return Ok(false);
    }
    let body = first(template.and_then(|t| std::fs::read_to_string(t).ok()));
    path.parent().map_or(Ok(()), std::fs::create_dir_all)?;
    crate::atomic::write(path, body).map(|()| true)
}

/// Make the folder `path`.
pub fn make_folder(path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}

/// The folders in `dir` that are not hidden, by name.
pub fn folders_in(dir: &PathBuf) -> Vec<PathBuf> {
    let Ok(read) = std::fs::read_dir(dir) else { return Vec::new() };
    read.flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| PathBuf::from(e.file_name()))
        .filter(|n| !n.to_string_lossy().starts_with('.'))
        .collect()
}

/// The notes read again, with the files opened outside the notes folder that still exist, newest
/// first; None when the folder cannot be read.
pub struct Reloaded {
    pub notes: Option<Vec<Note>>,
    pub alive: Vec<PathBuf>,
    pub current_exists: bool,
    /// The file of the editor and its text, when it was asked for.
    pub open: Option<(PathBuf, String)>,
}

/// Read the notes again, the files in `extra` that exist, whether `current` exists, and the text
/// of `open`.
pub fn reload(store: &Store, extra: &[PathBuf], current: Option<&Path>, open: Option<PathBuf>) -> Reloaded {
    let alive: Vec<PathBuf> = extra.iter().filter(|p| p.exists()).cloned().collect();
    let notes = store.load_all().ok().map(|mut n| {
        let more: Vec<Note> = alive.iter().filter(|p| !n.iter().any(|x| &x.path == *p)).filter_map(|p| notes::load(p).ok()).collect();
        n.extend(more);
        n.sort_by_key(|b| std::cmp::Reverse(b.modified));
        n
    });
    let current_exists = current.is_some_and(Path::exists);
    let open = open.and_then(|p| std::fs::read_to_string(&p).ok().map(|b| (p, b)));
    Reloaded { notes, alive, current_exists, open }
}

/// What a look at the disk saw: `project-structure.toml`, and the notes folder.
pub struct Looked {
    pub structure_mtime: Option<SystemTime>,
    /// The structure, read again when its time is not `seen`.
    pub structure: Option<StructureRead>,
    /// The number of notes and the newest change of the notes and of the open document.
    pub notes: (usize, Option<SystemTime>),
    pub current_exists: bool,
}

/// Look at the structure of `root`, the notes folder `dir` and the open document `current`.
pub fn look(root: &Path, dir: &Path, current: Option<&Path>, seen: Option<SystemTime>) -> Looked {
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
    Looked { structure_mtime, structure, notes, current_exists: current.is_some_and(Path::exists) }
}
