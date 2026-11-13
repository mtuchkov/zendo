//! The selection in read mode, and what it gives the chat.
use super::*;

impl App {
    /// The selection in read mode, start before end, both ends in it: ((row, col), (row, col)).
    pub fn read_selection(&self) -> Option<((usize, usize), (usize, usize))> {
        let a = self.read_anchor?;
        let c = (self.read_cursor, self.read_col);
        Some(if a <= c { (a, c) } else { (c, a) })
    }

    /// The selection in read mode, when it starts in a cell of a table with box borders (PANES-34).
    pub fn read_table(&self) -> Option<crate::text::boxtable::Range> {
        crate::text::boxtable::in_table(&self.read_text, self.read_anchor?, (self.read_cursor, self.read_col))
    }

    /// The selected rows; without a selection, the paragraph under the cursor.
    pub fn selection_rows(&self) -> (usize, usize) {
        if let Some(((a, _), (b, _))) = self.read_selection() {
            return (a, b);
        }
        let blank = |i: usize| self.read_text.get(i).is_none_or(|l| l.trim().is_empty());
        let (mut a, mut b) = (self.read_cursor, self.read_cursor);
        while a > 0 && !blank(a - 1) {
            a -= 1;
        }
        while b + 1 < self.read_text.len() && !blank(b + 1) {
            b += 1;
        }
        (a, b)
    }

    pub(crate) fn selection_text(&self) -> String {
        if let Some(t) = self.read_table() {
            return t.text(&self.read_text).trim().to_string();
        }
        if let Some(((r1, c1), (r2, c2))) = self.read_selection() {
            let mut rows = Vec::new();
            for r in r1..=r2.min(self.read_text.len().saturating_sub(1)) {
                let chars: Vec<char> = self.read_text[r].trim_end().chars().collect();
                let from = if r == r1 { c1.min(chars.len()) } else { 0 };
                let to = if r == r2 { (c2 + 1).min(chars.len()) } else { chars.len() };
                rows.push(chars[from..to.max(from)].iter().collect::<String>());
            }
            return rows.join("\n").trim().to_string();
        }
        let (a, b) = self.selection_rows();
        let rows: Vec<&str> = self.read_text.get(a..=b.min(self.read_text.len().saturating_sub(1))).unwrap_or(&[]).iter().map(|s| s.trim_end()).collect();
        rows.join("\n").trim().to_string()
    }

    /// The selection, or the paragraph under the cursor, quoted with its file and source lines.
    pub(crate) fn selection_context(&self) -> Option<String> {
        let text = self.selection_text();
        if text.is_empty() {
            return None;
        }
        let (a, b) = self.selection_rows();
        // Source lines of the selection, 1-based, for Claude to find them in the file.
        // Lines that share a row (a paragraph, or a skipped title and blank line): take the first
        // line with text that is not the title.
        let src: Vec<String> = self.note_text().unwrap_or_default().lines().map(str::to_string).collect();
        let text_at = |i: usize| src.get(i).is_some_and(|l| !(l.trim().is_empty() || (i == 0 && l.starts_with("# "))));
        let lo = self.read_rows.iter().rposition(|r| *r <= a).unwrap_or(0);
        let row = self.read_rows.get(lo).copied().unwrap_or(0);
        let lo = (0..self.read_rows.len()).find(|i| self.read_rows[*i] == row && text_at(*i)).unwrap_or(lo);
        let hi = self.read_rows.iter().rposition(|r| *r <= b).unwrap_or(lo);
        let rel = self.current.as_ref().map(|p| self.rel(p)).unwrap_or_default();
        let quoted: String = text.lines().map(|l| format!("> {l}\n")).collect();
        Some(format!("Selected text from {rel} (about lines {}–{}):\n{quoted}", lo + 1, hi + 1))
    }

    /// `a`: ask Claude about the selection, or the paragraph under the cursor.
    pub fn ask_about_selection(&mut self) {
        let Some(ctx) = self.selection_context() else {
            self.flash("nothing to ask about here");
            return;
        };
        self.ask_context = Some(ctx);
        self.ask_default = Some("Look at the selected text: what is unclear, missing or wrong?".into());
        self.ask_about = "the selection";
        self.read_anchor = None;
        self.open_panel();
    }

    /// `+`: add the selection to the chat input as a `[selection]` chip; the focus stays here.
    pub fn add_to_chat(&mut self) {
        let Some(ctx) = self.selection_context() else {
            self.flash("nothing to add here");
            return;
        };
        let label = self.add_chip(chips::selection_label(&self.selection_text()), Some(ctx), None, true);
        self.read_anchor = None;
        let back = self.focus;
        self.open_panel();
        if self.panel {
            self.focus = back;
            self.flash(format!("{label} is in the chat"));
        }
    }

    pub fn copy_selection(&mut self) {
        let text = self.selection_text();
        if !text.is_empty() {
            self.clip = Some(text);
            self.read_anchor = None;
        }
    }

    /// Write mode: the selection, or the line of the cursor, goes to the chat as a chip.
    pub(crate) fn add_line_to_chat(&mut self) {
        let e = &self.editor;
        let ((lo, _), (hi, _)) = e.selection().unwrap_or(((e.row, 0), (e.row, 0)));
        let text = e.selected_text().unwrap_or_else(|| e.lines[e.row].clone());
        if text.trim().is_empty() {
            return self.flash("nothing to add here");
        }
        let rel = self.editor_path.as_ref().map(|p| self.rel(p)).unwrap_or_default();
        let quoted: String = text.lines().map(|l| format!("> {l}\n")).collect();
        let ctx = format!("Selected text from {rel} (about lines {}–{}):\n{quoted}", lo + 1, hi + 1);
        let label = self.add_chip(chips::selection_label(&text), Some(ctx), None, true);
        let back = self.focus;
        self.open_panel();
        if self.panel {
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
            Pane::Write => &mut self.editor,
            Pane::Chat => &mut self.input,
            _ => return false,
        };
        let Some(text) = e.selected_text().filter(|_| e.selection().is_some()) else { return false };
        match act {
            "cut" => {
                e.delete_selection();
            }
            _ => e.clear_selection(),
        }
        self.clip = Some(text);
        if pane == Pane::Write && act == "cut" {
            self.last_edit = Instant::now();
        }
        true
    }
}
