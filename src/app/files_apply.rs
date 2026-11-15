//! The app side of the file actions: what an `Effect` of the files changes in the chat, the
//! sessions, the tree and the focus (ADR 0008).
use super::*;

impl App {
    /// Do what a sub-state asked for.
    pub(crate) fn apply(&mut self, effect: Option<Effect>) {
        match effect {
            None => {}
            Some(Effect::Flash(text)) => self.flash(text),
            Some(Effect::Deleted(key)) => self.file_deleted(key),
            Some(Effect::Moved { old, target, done }) => self.file_moved(&old, &target, done),
            Some(Effect::MadeFile(path)) => self.file_made(path),
            Some(Effect::MadeFolder(path)) => {
                self.scan_tree();
                let rel = path.strip_prefix(&self.cfg.root).unwrap_or(&path).to_path_buf();
                self.files.tree.reveal(&rel);
                self.flash(format!("made {}/ · the folders view shows it", rel.display()));
            }
        }
    }

    /// The selected note of the notes list, from the repository root.
    pub(crate) fn notes_file(&self) -> Option<PathBuf> {
        let n = self.files.notes().get(self.lists.note())?;
        Some(n.path.strip_prefix(&self.cfg.root).unwrap_or(&n.path).to_path_buf())
    }

    /// Ask about a file: `d` delete, `r` rename, `m` move.
    pub(crate) fn ask_file(&mut self, rel: Option<PathBuf>, what: char) {
        let said = self.files.ask_file(rel, what);
        self.apply(said);
    }

    /// The keys while a question about a file is open.
    pub(crate) fn key_file_prompt(&mut self, k: KeyEvent) {
        let done = self.files.key_prompt(k, &self.cfg.root);
        self.apply(done);
    }

    /// The best folder for the last part of a typed path: `docs/zen` → `docs/zendo/`.
    pub fn complete_folder(&self, text: &str) -> Option<String> {
        files_state::ops::complete_folder(&self.cfg.root, text)
    }

    /// The thread and the session of a deleted file go with it.
    fn file_deleted(&mut self, key: String) {
        self.state.take_session(&self.cfg.agent, &key);
        self.state.save(&self.cfg.root);
        self.threads.remove(&key);
        self.scan_tree();
        self.lists.keep_note_in(self.files.notes().len());
        self.flash(format!("deleted {key}"));
    }

    /// A new file opens to write.
    fn file_made(&mut self, path: PathBuf) {
        self.scan_tree();
        let rel = path.strip_prefix(&self.cfg.root).unwrap_or(&path).to_path_buf();
        self.files.tree.reveal(&rel);
        self.open_file(path);
        self.focus = Focus::Note;
        self.start_write(false);
    }

    /// A renamed or moved file shows in the tree and the notes list under its new name.
    fn file_moved(&mut self, old: &Path, target: &Path, done: &str) {
        self.keys_moved(old, target);
        self.scan_tree();
        // Show the file under its new name, also when a name filter found it.
        self.files.tree.find = None;
        let new_rel = target.strip_prefix(&self.cfg.root).unwrap_or(target).to_path_buf();
        self.files.tree.reveal(&new_rel);
        if let Some(i) = self.files.notes().iter().position(|n| n.path == target) {
            self.lists.select_note(i);
        }
        self.flash(format!("{done} to {}", self.rel(target)));
    }

    /// The session, the place in the recent files and the thread of a file follow its new path.
    fn keys_moved(&mut self, old: &Path, target: &Path) {
        let (ok, nk) = (self.rel(old), self.rel(target));
        if let Some(id) = self.state.take_session(&self.cfg.agent, &ok) {
            self.state.set_session(&self.cfg.agent, &nk, id);
            self.state.save(&self.cfg.root);
        }
        self.rename_recent(&ok, &nk);
        self.threads.rename(&ok, nk);
    }

    /// Move a file, and its thread and session with it.
    pub(crate) fn move_file(&mut self, old: &Path, target: &Path) -> std::io::Result<()> {
        self.files.move_file(old, target)?;
        self.keys_moved(old, target);
        Ok(())
    }
}
