//! Delete, rename and move a file, from the file tree or the notes list. The files do the work on
//! disk; the thread and the session of the file follow through an `Effect`.
use super::{Effect, FilePrompt, Files};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
            'r' => FilePrompt::Rename(rel, name),
            _ => FilePrompt::Move(rel, dir),
        });
        None
    }

    /// The keys while a question is open: `y` or not for a delete; typing, `tab`, `enter` and `esc`
    /// for a name or a folder. `root` is the repository.
    pub fn key_prompt(&mut self, k: KeyEvent, root: &Path) -> Option<Effect> {
        let mut prompt = self.prompt.take()?;
        let Some(text) = prompt.field_mut() else {
            return match (&prompt, k.code) {
                (FilePrompt::Delete(rel), KeyCode::Char('y' | 'Y')) => self.delete(root, rel),
                _ => None,
            };
        };
        match k.code {
            KeyCode::Esc => return None,
            KeyCode::Enter => return self.finish(prompt, root),
            KeyCode::Tab => {
                if let Some(t) = complete_folder(root, text) {
                    *text = t;
                }
            }
            KeyCode::Backspace => {
                text.pop();
            }
            KeyCode::Char(c) if !k.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => text.push(c),
            _ => {}
        }
        self.prompt = Some(prompt);
        None
    }

    fn finish(&mut self, prompt: FilePrompt, root: &Path) -> Option<Effect> {
        match prompt {
            FilePrompt::Delete(rel) => self.delete(root, &rel),
            FilePrompt::Rename(rel, name) => self.rename(root, &rel, &name),
            FilePrompt::Move(rel, folder) => self.move_to(root, &rel, &folder),
            FilePrompt::NewFile(path) => self.make_file(root, &path),
            FilePrompt::NewFolder(path) => self.make_folder(root, &path),
        }
    }

    fn delete(&mut self, root: &Path, rel: &Path) -> Option<Effect> {
        let path = root.join(rel);
        if let Err(e) = std::fs::remove_file(&path) {
            return Effect::flash(format!("delete failed: {e}"));
        }
        let key = self.rel(&path);
        if self.current.as_ref() == Some(&path) {
            self.current = None;
        }
        self.reload();
        Some(Effect::Deleted(key))
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
        let dir = root.join(folder);
        if let Err(e) = std::fs::create_dir_all(&dir) {
            return Effect::flash(format!("move failed: {e}"));
        }
        let target = dir.join(old.file_name().unwrap_or_default());
        self.relocate(&old, &target, "moved")
    }

    /// Rename or move a file on disk; `done` is the word for the message.
    fn relocate(&mut self, old: &Path, target: &Path, done: &'static str) -> Option<Effect> {
        if target == old {
            return None;
        }
        if target.exists() {
            return Effect::flash("a file with that name exists");
        }
        match self.move_file(old, target) {
            Ok(()) => Some(Effect::Moved { old: old.to_path_buf(), target: target.to_path_buf(), done }),
            Err(e) => Effect::flash(format!("{done} failed: {e}")),
        }
    }

    /// Move a file on disk; the open document follows it.
    pub fn move_file(&mut self, old: &Path, target: &Path) -> std::io::Result<()> {
        std::fs::rename(old, target)?;
        if self.current.as_deref() == Some(old) {
            self.current = Some(target.to_path_buf());
        }
        self.reload();
        Ok(())
    }
}

/// The best folder for the last part of a typed path: `docs/zen` → `docs/zendo/`.
pub fn complete_folder(root: &Path, text: &str) -> Option<String> {
    let (parent, part) = text.rsplit_once('/').unwrap_or(("", text));
    if part.is_empty() {
        return None;
    }
    let names: Vec<PathBuf> = std::fs::read_dir(root.join(parent))
        .ok()?
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| PathBuf::from(e.file_name()))
        .filter(|n| !n.to_string_lossy().starts_with('.'))
        .collect();
    let best = crate::app::at::rank(&names, part).into_iter().next()?;
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
