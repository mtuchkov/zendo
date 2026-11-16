//! The selection in read mode, and what it gives the chat.
use super::*;

impl App {
    /// The selection in read mode, start before end, both ends in it: ((row, col), (row, col)).
    pub fn read_selection(&self) -> Option<((usize, usize), (usize, usize))> {
        self.reader.selection()
    }

    /// The selected rows; without a selection, the paragraph under the cursor.
    pub fn selection_rows(&self) -> (usize, usize) {
        self.reader.selection_rows(self.frame.read_text())
    }

    pub(crate) fn selection_text(&self) -> String {
        self.reader.selection_text(self.frame.read_text())
    }

    /// The selection, or the paragraph under the cursor, quoted with its file and source lines.
    pub(crate) fn selection_context(&self) -> Option<String> {
        let text = self.selection_text();
        if text.is_empty() {
            return None;
        }
        let (a, b) = self.selection_rows();
        Some(self.read_context(&text, a, b))
    }

    /// `text` from rows `a` to `b` of read mode, quoted with its file and source lines.
    fn read_context(&self, text: &str, a: usize, b: usize) -> String {
        // Source lines of the selection, 1-based, for Claude to find them in the file.
        // Lines that share a row (a paragraph, or a skipped title and blank line): take the first
        // line with text that is not the title.
        let src: Vec<String> = self.note_text().unwrap_or_default().lines().map(str::to_string).collect();
        let text_at = |i: usize| src.get(i).is_some_and(|l| !(l.trim().is_empty() || (i == 0 && l.starts_with("# "))));
        let lo = self.frame.read_rows().iter().rposition(|r| *r <= a).unwrap_or(0);
        let row = self.frame.read_rows().get(lo).copied().unwrap_or(0);
        let lo = (0..self.frame.read_rows().len()).find(|i| self.frame.read_rows()[*i] == row && text_at(*i)).unwrap_or(lo);
        let hi = self.frame.read_rows().iter().rposition(|r| *r <= b).unwrap_or(lo);
        let rel = self.files.current().map(|p| self.rel(p)).unwrap_or_default();
        let quoted: String = text.lines().map(|l| format!("> {l}\n")).collect();
        format!("Selected text from {rel} (about lines {}–{}):\n{quoted}", lo + 1, hi + 1)
    }

    /// Alt and a drag of the mouse: the text goes to the chat input as a chip, with where it comes
    /// from, and the focus goes to the input (PANES-44).
    pub fn mouse_to_chat(&mut self, text: String) {
        let Some(s) = self.pointer.take() else { return };
        if text.trim().is_empty() {
            return self.flash("nothing to add here");
        }
        let quoted: String = text.lines().map(|l| format!("> {l}\n")).collect();
        let rel = |a: &Self, p: Option<&std::path::PathBuf>| p.map(|p| a.rel(p)).unwrap_or_default();
        let ctx = match s.region {
            Region::Note if self.mode == Mode::Read => {
                let row = |y: u16| self.reader.scroll() + y.saturating_sub(s.rect.y) as usize;
                self.read_context(&text, row(s.anchor.1.min(s.head.1)), row(s.anchor.1.max(s.head.1)))
            }
            Region::Note => format!("Selected text from {}:\n{quoted}", rel(self, self.writer.path())),
            Region::Delta => format!("Selected text from the delta:\n{quoted}"),
            Region::Panel if self.cli.pane.is_some() => format!("Text from the terminal:\n{quoted}"),
            Region::Panel => format!("Text from this chat:\n{quoted}"),
        };
        let label = self.composer.add_chip(chips::selection_label(&text), Some(ctx), None, true);
        self.open_panel();
        if self.threads.shown() {
            self.focus = Focus::Claude;
            self.threads.transcript_mut().focus(false);
        }
        self.flash(format!("{} is in the chat", label.trim_matches([chips::OPEN, chips::CLOSE])));
    }

    /// `a`: ask Claude about the selection, or the paragraph under the cursor.
    pub fn ask_about_selection(&mut self) {
        let Some(ctx) = self.selection_context() else {
            self.flash("nothing to ask about here");
            return;
        };
        self.composer.set_context(ctx);
        self.composer.set_default("Look at the selected text: what is unclear, missing or wrong?".into(), "the selection");
        self.reader.unselect();
        self.open_panel();
    }

    /// `+`: add the selection to the chat input as a `[selection]` chip; the focus stays here.
    pub fn add_to_chat(&mut self) {
        let Some(ctx) = self.selection_context() else {
            self.flash("nothing to add here");
            return;
        };
        let label = self.composer.add_chip(chips::selection_label(&self.selection_text()), Some(ctx), None, true);
        self.reader.unselect();
        let back = self.focus;
        self.open_panel();
        if self.threads.shown() {
            self.focus = back;
            self.flash(format!("{label} is in the chat"));
        }
    }

    pub fn copy_selection(&mut self) {
        let text = self.selection_text();
        if !text.is_empty() {
            self.pointer.copy(text);
            self.reader.unselect();
        }
    }

    /// Write mode: the selection, or the line of the cursor, goes to the chat as a chip.
    pub(crate) fn add_line_to_chat(&mut self) {
        let e = self.writer.editor();
        let ((lo, _), (hi, _)) = e.selection().unwrap_or(((e.row, 0), (e.row, 0)));
        let text = e.selected_text().unwrap_or_else(|| e.lines[e.row].clone());
        if text.trim().is_empty() {
            return self.flash("nothing to add here");
        }
        let rel = self.writer.path().map(|p| self.rel(p)).unwrap_or_default();
        let quoted: String = text.lines().map(|l| format!("> {l}\n")).collect();
        let ctx = format!("Selected text from {rel} (about lines {}–{}):\n{quoted}", lo + 1, hi + 1);
        let label = self.composer.add_chip(chips::selection_label(&text), Some(ctx), None, true);
        let back = self.focus;
        self.open_panel();
        if self.threads.shown() {
            self.focus = back;
            self.flash(format!("{} is in the chat", label.trim_matches([chips::OPEN, chips::CLOSE])));
        }
    }

    /// While text is selected in the note in write mode or in the chat input, the keys of
    /// `[selection]` come first: `c` copies and `x` cuts (PANES-37). True when the key did that.
    pub(crate) fn selection_key(&mut self, pane: crate::keys::Pane, k: &KeyEvent) -> bool {
        use crate::keys::Pane;
        let Some(act) = self.keys.local(Pane::Selected, k) else { return false };
        let e = match pane {
            Pane::Write => self.writer.editor_mut(),
            Pane::Chat => self.composer.input_mut(),
            _ => return false,
        };
        let Some(text) = e.selected_text().filter(|_| e.selection().is_some()) else { return false };
        match act {
            "cut" => {
                e.delete_selection();
            }
            _ => e.clear_selection(),
        }
        self.pointer.copy(text);
        if pane == Pane::Write && act == "cut" {
            self.writer.touch();
        }
        self.did(Pane::Selected, act);
        true
    }
}
