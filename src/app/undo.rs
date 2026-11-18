//! Undo and redo in every input (UNDO-1 to UNDO-6, ADR 0014). After each key, paste and mouse
//! event, the editors record their changes in their loglets. The loglet of a document goes to
//! `.zendo/db.log.json` with each save, and comes back when the document opens in write mode.
use super::*;

impl App {
    pub fn on_key(&mut self, k: KeyEvent) {
        self.key_in(k);
        self.commit_edits();
    }

    pub fn on_paste(&mut self, s: String) {
        self.paste_in(s);
        self.commit_edits();
    }

    pub fn on_mouse(&mut self, m: MouseEvent) {
        self.mouse_in(m);
        self.commit_edits();
    }

    /// The editors of write mode and of the chat input record what changed (UNDO-2). The text
    /// fields record their changes by themselves.
    pub(crate) fn commit_edits(&mut self) {
        self.writer.editor_mut().commit();
        self.composer.input_mut().commit();
    }

    /// Write mode on a document: its stored loglet goes on, moved over the changes on disk since
    /// the last save (UNDO-4, UNDO-6).
    pub(crate) fn resume_log(&mut self) {
        let Some(path) = self.writer.path().cloned() else { return };
        let Some(d) = self.logs.get(&self.rel(&path)) else { return };
        let (log, text) = (d.log.clone(), d.text.clone());
        self.writer.editor_mut().resume(log, &text);
    }

    /// The document is on disk in `path`: its loglet goes to `.zendo/db.log.json` (UNDO-4).
    pub(crate) fn keep_log(&mut self, path: &Path) {
        let (log, text) = self.writer.editor_mut().history();
        let rel = self.rel(path);
        self.logs.put(&rel, log, text);
        self.logs.save(&self.cfg.root);
    }
}
