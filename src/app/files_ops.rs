//! Delete, rename and move a file, from the file tree or the notes list. The thread of the file
//! goes with it.
use super::*;

impl App {
    /// The selected row of the tree, when it is a file.
    pub(crate) fn tree_file(&self) -> Option<PathBuf> {
        let rows = self.tree.rows();
        rows.get(self.tree.sel).filter(|r| r.kind == crate::docs::tree::Kind::File).map(|r| r.rel.clone())
    }

    /// The selected note of the notes list, from the repository root.
    pub(crate) fn notes_file(&self) -> Option<PathBuf> {
        let n = self.notes.get(self.notes_sel)?;
        Some(n.path.strip_prefix(&self.cfg.root).unwrap_or(&n.path).to_path_buf())
    }

    /// Ask about a file: `d` delete, `r` rename, `m` move.
    pub(crate) fn ask_file(&mut self, rel: Option<PathBuf>, what: char) {
        let Some(rel) = rel else { return self.flash("select a file first") };
        let name = rel.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let dir = rel.parent().map(|p| p.display().to_string()).unwrap_or_default();
        self.file_prompt = Some(match what {
            'd' => FilePrompt::Delete(rel),
            'r' => FilePrompt::Rename(rel, name),
            _ => FilePrompt::Move(rel, dir),
        });
    }

    /// The keys while a question is open: `y` or not for a delete; typing, `tab`, `enter` and `esc`
    /// for a name or a folder.
    pub(crate) fn key_file_prompt(&mut self, k: KeyEvent) {
        let Some(mut prompt) = self.file_prompt.take() else { return };
        let Some(text) = prompt.field_mut() else {
            if let (FilePrompt::Delete(rel), KeyCode::Char('y' | 'Y')) = (&prompt, k.code) {
                self.delete_file(&rel.clone());
            }
            return;
        };
        match k.code {
            KeyCode::Esc => return,
            KeyCode::Enter => return self.finish_prompt(prompt),
            KeyCode::Tab => {
                if let Some(t) = self.complete_folder(text) {
                    *text = t;
                }
            }
            KeyCode::Backspace => {
                text.pop();
            }
            KeyCode::Char(c) if !k.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) => text.push(c),
            _ => {}
        }
        self.file_prompt = Some(prompt);
    }

    fn finish_prompt(&mut self, prompt: FilePrompt) {
        match prompt {
            FilePrompt::Delete(rel) => self.delete_file(&rel),
            FilePrompt::Rename(rel, name) => self.rename_file(&rel, &name),
            FilePrompt::Move(rel, folder) => self.move_to(&rel, &folder),
            FilePrompt::NewFile(path) => self.make_file(&path),
            FilePrompt::NewFolder(path) => self.make_folder(&path),
        }
    }

    fn delete_file(&mut self, rel: &Path) {
        let path = self.cfg.root.join(rel);
        if let Err(e) = std::fs::remove_file(&path) {
            return self.flash(format!("delete failed: {e}"));
        }
        let key = self.rel(&path);
        self.state.take_session(&self.cfg.agent, &key);
        self.state.save(&self.cfg.root);
        self.threads.remove(&key);
        if self.current.as_ref() == Some(&path) {
            self.current = None;
        }
        self.reload();
        self.scan_tree();
        self.notes_sel = self.notes_sel.min(self.notes.len().saturating_sub(1));
        self.flash(format!("deleted {key}"));
    }

    fn rename_file(&mut self, rel: &Path, name: &str) {
        let old = self.cfg.root.join(rel);
        match rename_target(&old, name) {
            Ok(target) => self.relocate(&old, &target, "renamed"),
            Err(e) => self.flash(e),
        }
    }

    /// Move a file to a folder of the repository; a folder that is not there yet is made.
    fn move_to(&mut self, rel: &Path, folder: &str) {
        let old = self.cfg.root.join(rel);
        let folder = folder.trim().trim_matches('/');
        if folder.split('/').any(|p| p == "..") {
            return self.flash("give a folder inside the repository");
        }
        let dir = self.cfg.root.join(folder);
        if let Err(e) = std::fs::create_dir_all(&dir) {
            return self.flash(format!("move failed: {e}"));
        }
        let target = dir.join(old.file_name().unwrap_or_default());
        self.relocate(&old, &target, "moved");
    }

    /// Rename or move a file, then show it in the tree and the notes list.
    pub(crate) fn relocate(&mut self, old: &Path, target: &Path, done: &str) {
        if target == old {
            return;
        }
        if target.exists() {
            return self.flash("a file with that name exists");
        }
        if let Err(e) = self.move_file(old, target) {
            return self.flash(format!("{done} failed: {e}"));
        }
        self.scan_tree();
        // Show the file under its new name, also when a name filter found it.
        self.tree.find = None;
        let new_rel = target.strip_prefix(&self.cfg.root).unwrap_or(target).to_path_buf();
        self.tree.reveal(&new_rel);
        self.notes_sel = self.notes.iter().position(|n| n.path == target).unwrap_or(self.notes_sel);
        self.flash(format!("{done} to {}", self.rel(target)));
    }

    /// Move a file, and its thread and session with it.
    pub(crate) fn move_file(&mut self, old: &Path, target: &Path) -> std::io::Result<()> {
        std::fs::rename(old, target)?;
        let (ok, nk) = (self.rel(old), self.rel(target));
        if let Some(id) = self.state.take_session(&self.cfg.agent, &ok) {
            self.state.set_session(&self.cfg.agent, &nk, id);
            self.state.save(&self.cfg.root);
        }
        if let Some(t) = self.threads.remove(&ok) {
            self.threads.insert(nk, t);
        }
        if self.current.as_deref() == Some(old) {
            self.current = Some(target.to_path_buf());
        }
        self.reload();
        Ok(())
    }
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
