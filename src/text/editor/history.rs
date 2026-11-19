//! Undo and redo: the loglet of the editor (UNDO-1 to UNDO-16, ADR 0014).
//!
//! The chat input and the text fields keep their loglets in memory: the app commits after each
//! key, paste and mouse event, and the difference since the head of the loglet is one operation.
//! A document records only what zendo saved: each save is one operation (UNDO-16), and the text
//! that is not saved yet stays out of the loglet. A change of the file by another writer is
//! appended as one external operation (UNDO-6).
use super::*;
use crate::text::loglet::Loglet;
use crate::text::ot::Op;

/// The loglet of an editor, the text at its head, and whether the last operation can grow.
#[derive(Clone, Debug, Default)]
pub struct History {
    log: Loglet,
    /// The text after the last done operation. For a document: the file as zendo last saved or
    /// read it.
    logged: String,
    /// The last operation is a typed char or one deleted char: the next one can join it (UNDO-7).
    open: bool,
    /// The cursor at the last commit: a move ends the group (UNDO-7).
    at: (usize, usize),
    /// The editor holds a document: only `flush` records (UNDO-16).
    document: bool,
}

impl History {
    pub(super) fn of(text: &str) -> Self {
        Self { logged: text.to_string(), ..Default::default() }
    }
}

/// The change is one typed char that is no new line, or one deleted char.
fn small(op: &Op) -> bool {
    matches!(op.single(), Some((_, i, d)) if i.chars().count() + d.chars().count() == 1 && i != "\n")
}

/// `next` goes on from `last`: the next char of a word, or the next `backspace` or `delete` of a
/// run. A word takes the whitespace after it; a char after that whitespace starts a new word.
fn joins(last: &Op, next: &Op) -> bool {
    let (Some((p, ins, del)), Some((q, i2, d2))) = (last.single(), next.single()) else { return false };
    let word = ins.contains(|c: char| !c.is_whitespace());
    let after_space = word && ins.ends_with(char::is_whitespace) && !i2.starts_with(char::is_whitespace);
    match (ins.is_empty(), i2.is_empty()) {
        (false, false) => q == p + ins.chars().count() && !after_space,
        (true, true) => !del.is_empty() && !d2.is_empty() && (q + 1 == p || q == p),
        _ => false,
    }
}

impl Editor {
    /// The editor holds a document whose file is `file`, empty for a new note: its loglet takes
    /// only the saved text (UNDO-16).
    pub fn as_document(&mut self, file: &str) {
        self.history = History { document: true, ..History::of(file) };
    }

    /// Record the change since the last commit as an operation of the loglet (UNDO-2, UNDO-7).
    /// A new operation trims the undone ones (UNDO-3). A document records only at `flush`.
    pub fn commit(&mut self) {
        let now = self.text();
        let h = &mut self.history;
        if h.document {
            return;
        }
        if now == h.logged {
            if h.at != (self.row, self.col) {
                h.open = false;
                h.at = (self.row, self.col);
            }
            return;
        }
        let op = Op::diff(&h.logged, &now);
        let start = h.log.last().filter(|last| h.open && small(&op) && joins(last, &op)).and_then(|last| last.invert().apply(&h.logged));
        h.open = small(&op);
        match start {
            Some(start) => h.log.replace_last(Op::diff(&start, &now)),
            None => h.log.record(op),
        }
        h.logged = now;
        h.at = (self.row, self.col);
    }

    /// The document is on disk as `saved`: the change since the last save is one operation
    /// (UNDO-16). A new operation trims the undone ones (UNDO-3).
    pub fn flush(&mut self, saved: &str) {
        let h = &mut self.history;
        h.log.record(Op::diff(&h.logged, saved));
        h.logged = saved.to_string();
    }

    /// `ctrl-z`: take back the last change (UNDO-1). False when there is none (UNDO-9). In a
    /// document the app saves first, so the undo starts from the saved text.
    pub fn undo(&mut self) -> bool {
        self.commit();
        let Some(op) = self.history.log.undo() else { return false };
        self.apply_logged(&op)
    }

    /// `ctrl-y`: do the last undone change again (UNDO-1). False when there is none (UNDO-9).
    pub fn redo(&mut self) -> bool {
        self.commit();
        let Some(op) = self.history.log.redo() else { return false };
        self.apply_logged(&op)
    }

    /// Put the text after `op` into the editor, the cursor at the end of the change (UNDO-8). An
    /// operation that does not fit the text: the loglet goes, the text stays (UNDO-11).
    fn apply_logged(&mut self, op: &Op) -> bool {
        let Some(text) = op.apply(&self.history.logged) else {
            self.history = History { document: self.history.document, ..History::of(&self.history.logged) };
            return false;
        };
        self.set_lines(&text);
        (self.row, self.col) = self.position(op.end());
        self.anchor = None;
        let h = &mut self.history;
        (h.logged, h.open) = (text, false);
        h.at = (self.row, self.col);
        self.touch();
        true
    }

    /// The text of the file at the last save or change from outside, as zendo saw it (UNDO-6).
    pub fn logged(&self) -> &str {
        &self.history.logged
    }

    /// The file of the document is `disk` now. When it differs from the file of the last save,
    /// the change is appended to the loglet as one external operation (UNDO-6). The text that is
    /// not saved stays out of the loglet: it moves over the change, and so does the cursor
    /// (UNDO-13). True when the file changed.
    pub fn external(&mut self, disk: &str) -> bool {
        if disk == self.history.logged {
            return false;
        }
        let theirs = Op::diff(&self.history.logged, disk);
        let unsaved = Op::diff(&self.history.logged, &self.text());
        let (_, over) = Op::transform(&unsaved, &theirs);
        let Some(text) = over.apply(&self.text()) else { return false };
        let at = over.map(self.offset());
        self.set_lines(&text);
        (self.row, self.col) = self.position(at);
        self.anchor = None;
        let h = &mut self.history;
        h.log.record_external(theirs);
        (h.logged, h.open) = (disk.to_string(), false);
        true
    }

    /// The next undo takes back a change of another writer (UNDO-12).
    pub fn external_next(&self) -> bool {
        self.history.log.external_next()
    }

    /// Keep the external change before the head: the next undo takes back the change of the user
    /// before it (UNDO-14).
    pub fn keep_external(&mut self) {
        self.history.log.keep_external();
    }

    /// A document goes on with a stored loglet whose head is the file of its last save, `head`.
    /// The text of the editor is the file now: a change since then is one external operation
    /// (UNDO-4, UNDO-6, UNDO-15). A loglet that does not fit its head goes (UNDO-11).
    pub fn resume(&mut self, log: Loglet, head: &str) {
        let fits = log.last().is_none_or(|op| op.invert().apply(head).is_some());
        let log = if fits { log } else { Loglet::default() };
        let now = self.text();
        self.history = History { log, logged: head.to_string(), document: true, ..Default::default() };
        if now != head {
            self.history.log.record_external(Op::diff(head, &now));
            self.history.logged = now;
        }
    }

    /// The loglet and the text at its head, for `.zendo/db.log.json` (UNDO-4).
    pub fn history(&mut self) -> (Loglet, String) {
        self.commit();
        (self.history.log.clone(), self.history.logged.clone())
    }

    /// A new text from the app, such as a title of the agent; the loglet stays.
    pub fn set_text(&mut self, text: &str) {
        self.set_lines(text);
        self.anchor = None;
        self.touch();
    }

    /// Empty the text and keep the loglet: the clear can be undone.
    pub fn clear_text(&mut self) {
        self.set_lines("");
        (self.row, self.col, self.anchor) = (0, 0, None);
        self.touch();
    }

    fn set_lines(&mut self, text: &str) {
        self.lines = text.split('\n').map(str::to_string).collect();
    }

    /// The char index of the cursor in the text.
    fn offset(&self) -> usize {
        self.lines[..self.row].iter().map(|l| l.chars().count() + 1).sum::<usize>() + self.col
    }

    /// The (line, char) of the char index `at` in the text.
    fn position(&self, at: usize) -> (usize, usize) {
        let mut left = at;
        for (row, line) in self.lines.iter().enumerate() {
            let n = line.chars().count();
            if left <= n {
                return (row, left);
            }
            left -= n + 1;
        }
        let last = self.lines.len() - 1;
        (last, self.len(last))
    }
}
