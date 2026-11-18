//! Undo and redo: the loglet of the editor (UNDO-1 to UNDO-15, ADR 0014). The app commits after
//! each key, paste and mouse event: the difference between the text at the head of the loglet and
//! the text now is one operation. A change of the file by another writer is one external operation.
use super::*;
use crate::text::loglet::Loglet;
use crate::text::ot::Op;

/// The loglet of an editor, the text at its head, and whether the last operation can grow.
#[derive(Clone, Debug, Default)]
pub struct History {
    log: Loglet,
    /// The text after the last done operation.
    logged: String,
    /// The last operation is a typed char or one deleted char: the next one can join it (UNDO-7).
    open: bool,
    /// The cursor at the last commit: a move ends the group (UNDO-7).
    at: (usize, usize),
    /// The text of the file as zendo last read or saved it: a file that differs from it changed
    /// outside zendo (UNDO-6).
    disk: String,
}

impl History {
    pub(super) fn of(text: &str) -> Self {
        Self { logged: text.to_string(), disk: text.to_string(), ..Default::default() }
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
    /// Record the change since the last commit as an operation of the loglet (UNDO-2, UNDO-7).
    /// A new operation trims the undone ones (UNDO-3).
    pub fn commit(&mut self) {
        let now = self.text();
        let h = &mut self.history;
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

    /// `ctrl-z`: take back the last change (UNDO-1). False when there is none (UNDO-9).
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
            self.history = History::of(&self.history.logged);
            return false;
        };
        self.set_lines(&text);
        (self.row, self.col) = self.position(op.end());
        self.anchor = None;
        self.history.logged = text;
        self.history.open = false;
        self.history.at = (self.row, self.col);
        self.touch();
        true
    }

    /// The file is `disk` now. When it differs from the file that zendo last read or saved, the
    /// change goes into the loglet as one external operation, merged with the text that is not
    /// saved yet, and the cursor moves with it (UNDO-6, UNDO-13). True when the file changed.
    pub fn external(&mut self, disk: &str) -> bool {
        if disk == self.history.disk {
            return false;
        }
        self.commit();
        let h = &mut self.history;
        let theirs = Op::diff(&h.disk, disk);
        let (merged, _) = Op::transform(&theirs, &Op::diff(&h.disk, &h.logged));
        h.disk = disk.to_string();
        let Some(text) = merged.apply(&h.logged) else { return false };
        let at = merged.map(self.offset());
        self.set_lines(&text);
        (self.row, self.col) = self.position(at);
        self.anchor = None;
        let h = &mut self.history;
        h.log.record_external(merged);
        (h.logged, h.open, h.at) = (text, false, (self.row, self.col));
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

    /// The editor saved its text as `disk`: the file of the last save is no external change.
    pub fn saved_as(&mut self, disk: &str) {
        self.history.disk = disk.to_string();
    }

    /// Go on with a stored loglet: `head` is the text at its head, `disk` the file of its last
    /// save, and the text of the editor the file now. The trim of the save moves the loglet; a
    /// change of the file since then is one external operation (UNDO-4, UNDO-6, UNDO-15).
    pub fn resume(&mut self, log: Loglet, head: &str, disk: &str) {
        let now = self.text();
        self.history = History { log, logged: head.to_string(), disk: disk.to_string(), ..Default::default() };
        self.set_lines(disk);
        self.absorb();
        let (row, col) = (self.row, self.col);
        self.external(&now);
        (self.row, self.col) = (row.min(self.lines.len() - 1), col);
        self.col = self.col.min(self.len(self.row));
    }

    /// The loglet, the text at its head and the file of the last save, for `.zendo/db.log.json`
    /// (UNDO-4).
    pub fn history(&mut self) -> (Loglet, String, String) {
        self.commit();
        let h = &self.history;
        (h.log.clone(), h.logged.clone(), h.disk.clone())
    }

    /// Empty the text and keep the loglet: the clear can be undone.
    pub fn clear_text(&mut self) {
        self.set_lines("");
        (self.row, self.col, self.anchor) = (0, 0, None);
        self.touch();
    }

    /// The text now differs from the head of the loglet by a change that is no edit, such as the
    /// trim of a save: the loglet moves over it. A loglet that does not fit goes (UNDO-11).
    fn absorb(&mut self) {
        let now = self.text();
        let h = &mut self.history;
        let fits = h.log.last().is_none_or(|op| op.invert().apply(&h.logged).is_some());
        if !fits {
            h.log = Loglet::default();
        }
        h.log.rebase(Op::diff(&h.logged, &now));
        h.logged = now;
        h.open = false;
        h.at = (self.row, self.col);
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
