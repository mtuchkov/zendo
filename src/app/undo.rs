//! Undo and redo in every input (UNDO-1 to UNDO-15, ADR 0014). After each key, paste and mouse
//! event, the editors record their changes in their loglets. The loglet of a document goes to
//! `.zendo/db.log.json` with each save, and comes back when the document opens in write mode. The
//! undo of a change of another writer asks first.
use super::*;

/// The question before the undo of an external change (UNDO-12).
pub const EXTERNAL_QUESTION: &str = "The next undo takes back a change of the file from outside zendo: the agent, git or another editor. Undo it? n keeps it and undoes your change before it; esc does nothing.";

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

    /// `undo` in write mode. Before a change of another writer, the question opens and nothing
    /// changes yet (UNDO-12). `Some(edited)`, or `None` when the question is open.
    pub(super) fn write_undo(&mut self) -> Option<bool> {
        let e = self.writer.editor_mut();
        if e.external_next() {
            self.writer.set_asks_external(true);
            return None;
        }
        Some(e.undo())
    }

    /// A key while the question is open: `y` undoes the external change, `n` keeps it and undoes
    /// the change of the user before it, `esc` and every other key do nothing (UNDO-12, UNDO-14).
    pub(super) fn key_external_question(&mut self, k: KeyEvent) {
        let answer = match k.code {
            KeyCode::Char('y' | 'Y') => Some(true),
            KeyCode::Char('n' | 'N') => Some(false),
            _ => None,
        };
        self.answer_external(answer);
    }

    /// A click while the question is open: on one of its answers.
    pub(super) fn click_external_question(&mut self, x: u16, y: u16) {
        let Some((yes, no)) = self.frame.external_answers() else { return };
        let hit = |r: Rect| r.contains(ratatui::layout::Position::new(x, y));
        if hit(yes) || hit(no) {
            self.answer_external(Some(hit(yes)));
        }
    }

    fn answer_external(&mut self, undo_it: Option<bool>) {
        self.writer.set_asks_external(false);
        let edited = match undo_it {
            Some(true) => self.writer.editor_mut().undo(),
            Some(false) => {
                self.writer.editor_mut().keep_external();
                self.write_undo().unwrap_or(false)
            }
            None => false,
        };
        if edited {
            self.writer.touch();
        }
    }

    /// Write mode on a document: its stored loglet goes on. A change of the file since the last
    /// save is one external operation (UNDO-4, UNDO-6, UNDO-15).
    pub(crate) fn resume_log(&mut self) {
        let Some(path) = self.writer.path().cloned() else { return };
        let Some(d) = self.logs.get(&self.rel(&path)) else { return };
        let disk = if d.disk.is_empty() { d.text.clone() } else { d.disk.clone() };
        let (log, text) = (d.log.clone(), d.text.clone());
        self.writer.editor_mut().resume(log, &text, &disk);
    }

    /// The document is on disk in `path`: its loglet goes to `.zendo/db.log.json` (UNDO-4).
    pub(crate) fn keep_log(&mut self, path: &Path) {
        let (log, text, disk) = self.writer.editor_mut().history();
        let rel = self.rel(path);
        self.logs.put(&rel, log, text, disk);
        self.logs.save(&self.cfg.root);
    }
}
