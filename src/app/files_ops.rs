//! Delete, rename and move a file, from the file tree or the notes list. The files do the work on
//! disk; the thread and the session of the file follow through an `Effect`.
use super::{Effect, FilePrompt, Files};
use crate::app::IoJob;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use std::path::{Path, PathBuf};

impl Files {
    /// The selected row of the tree, when it is a file.
    pub fn tree_file(&self) -> Option<PathBuf> {
        let rows = self.tree.rows();
        rows.get(self.tree.sel).filter(|r| r.kind == crate::docs::tree::Kind::File).map(|r| r.rel.clone())
    }

    /// Ask about a file: `d` delete, `r` rename, `m` move.
    pub fn ask_file(&mut self, rel: Option<PathBuf>, what: char) -> Option<Effect> {
        let Some(rel) = rel else { return Effect::flash("select a file first") };
        let name = rel.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let dir = rel.parent().map(|p| p.display().to_string()).unwrap_or_default();
        self.prompt = Some(match what {
            'd' => FilePrompt::Delete(rel),
            'r' => FilePrompt::Rename(rel, name.into()),
            _ => FilePrompt::Move(rel, dir.into()),
        });
        None
    }

    /// The keys while a question is open: `y` or not for a delete; `tab`, `enter` and `esc` for a
    /// name or a folder. `root` is the repository. The app gives the other keys to the field.
    /// `act` is the action of the key in `[question]`; `esc` is fixed (PANES-48).
    pub fn key_prompt(&mut self, k: KeyEvent, act: Option<&str>, root: &Path) -> Option<Effect> {
        let mut prompt = self.prompt.take()?;
        let Some(text) = prompt.field_mut() else {
            return match (&prompt, act) {
                (FilePrompt::Delete(rel), Some("yes")) => self.delete(root, rel),
                _ => None,
            };
        };
        match (k.code, act) {
            (KeyCode::Esc, _) => return None,
            (_, Some("ok")) => return self.finish(prompt, root),
            // Before the worker read the folders, the tab waits for them (IO-6).
            (_, Some("complete")) => match self.complete_folder(root, text.text()) {
                Some(t) => text.set(&t),
                None => self.complete_waits = Some(text.text().to_string()),
            },
            _ => drop(text.key(None, &k)),
        }
        self.prompt = Some(prompt);
        None
    }

    fn finish(&mut self, prompt: FilePrompt, root: &Path) -> Option<Effect> {
        match prompt {
            FilePrompt::Delete(rel) => self.delete(root, &rel),
            FilePrompt::Rename(rel, name) => self.rename(root, &rel, name.text()),
            FilePrompt::Move(rel, folder) => self.move_to(root, &rel, folder.text()),
            FilePrompt::NewFile(path) => self.make_file(root, path.text()),
            FilePrompt::NewFolder(path) => self.make_folder(root, path.text()),
        }
    }

    /// The file goes on a thread; its answer forgets it (IO-1).
    fn delete(&mut self, root: &Path, rel: &Path) -> Option<Effect> {
        let path = root.join(rel);
        let key = self.rel(&path);
        Some(Effect::Io(IoJob::new(move || match std::fs::remove_file(&path) {
            Ok(()) => Some(Effect::Deleted(key)),
            Err(e) => Effect::flash(format!("delete failed: {e}")),
        })))
    }

    /// The file `path` is gone: it is no longer the open document, and the notes read again.
    pub fn gone(&mut self, path: &Path) {
        if self.current.as_deref() == Some(path) {
            self.current = None;
        }
        self.reload();
    }

    fn rename(&mut self, root: &Path, rel: &Path, name: &str) -> Option<Effect> {
        let old = root.join(rel);
        match rename_target(&old, name) {
            Ok(target) => self.relocate(&old, &target, "renamed"),
            Err(e) => Effect::flash(e),
        }
    }

    /// Move a file to a folder of the repository; a folder that is not there yet is made.
    fn move_to(&mut self, root: &Path, rel: &Path, folder: &str) -> Option<Effect> {
        let old = root.join(rel);
        let folder = folder.trim().trim_matches('/');
        if folder.split('/').any(|p| p == "..") {
            return Effect::flash("give a folder inside the repository");
        }
        let target = root.join(folder).join(old.file_name().unwrap_or_default());
        self.relocate(&old, &target, "moved")
    }

    /// Rename or move a file on disk, on a thread; a folder that is not there yet is made; `done`
    /// is the word for the message (IO-1).
    fn relocate(&mut self, old: &Path, target: &Path, done: &'static str) -> Option<Effect> {
        if target == old {
            return None;
        }
        let (old, target) = (old.to_path_buf(), target.to_path_buf());
        Some(Effect::Io(IoJob::new(move || {
            if target.exists() {
                return Effect::flash("a file with that name exists");
            }
            match target.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|()| std::fs::rename(&old, &target)) {
                Ok(()) => Some(Effect::Moved { old, target, done }),
                Err(e) => Effect::flash(format!("{done} failed: {e}")),
            }
        })))
    }

    /// A file moved on disk: the open document follows it, and the notes read again.
    pub fn moved(&mut self, old: &Path, target: &Path) {
        if self.current.as_deref() == Some(old) {
            self.current = Some(target.to_path_buf());
        }
        self.reload();
    }

}

impl Files {
    /// A tab waits for the folders.
    pub fn completes(&self) -> bool {
        self.complete_waits.is_some()
    }

    /// A tab that waited for the folders: it completes the path now when the worker knows them and
    /// the path is as it was.
    pub fn complete_waiting(&mut self, root: &Path) {
        let Some(typed) = self.complete_waits.clone() else { return };
        let Some(t) = self.complete_folder(root, &typed) else { return };
        self.complete_waits = None;
        if let Some(p) = self.prompt.as_mut()
            && let Some(field) = p.field_mut()
            && field.text() == typed
        {
            field.set(&t);
        }
    }

    /// The best folder for the last part of a typed path: `docs/zen` → `docs/zendo/`. The folders
    /// come from a memo; none while the worker did not read them yet (IO-1).
    pub fn complete_folder(&self, root: &Path, text: &str) -> Option<String> {
        let parent = text.rsplit_once('/').map_or("", |(p, _)| p);
        complete_with(&self.folders.get(&root.join(parent))?, text)
    }
}

/// The folders in `dir` that are not hidden, by name; on a worker.
pub(super) fn folders_in(dir: &PathBuf) -> Vec<PathBuf> {
    let Ok(read) = std::fs::read_dir(dir) else { return Vec::new() };
    read.flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| PathBuf::from(e.file_name()))
        .filter(|n| !n.to_string_lossy().starts_with('.'))
        .collect()
}

/// The best folder of `names` for the last part of `text`.
fn complete_with(names: &[PathBuf], text: &str) -> Option<String> {
    let (parent, part) = text.rsplit_once('/').unwrap_or(("", text));
    if part.is_empty() {
        return None;
    }
    let best = crate::app::at::rank(names, part).into_iter().next()?;
    let best = best.display();
    Some(if parent.is_empty() { format!("{best}/") } else { format!("{parent}/{best}/") })
}

/// The new path of `old` for a typed name: in the same folder, with the old extension when the
/// name has none.
pub(crate) fn rename_target(old: &Path, name: &str) -> Result<PathBuf, String> {
    let name = name.trim();
    if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\']) {
        return Err("give a file name, without a folder".into());
    }
    let mut target = old.with_file_name(name);
    if target.extension().is_none()
        && let Some(ext) = old.extension()
    {
        target.set_extension(ext);
    }
    Ok(target)
}

#[cfg(test)]
#[path = "files_ops_tests.rs"]
mod tests;
