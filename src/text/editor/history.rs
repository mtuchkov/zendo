//! Undo and redo: the loglet of the editor (UNDO-1 to UNDO-11, ADR 0014). The app commits after
//! each key, paste and mouse event: the difference between the text at the head of the loglet and
//! the text now is one operation.
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

    /// Another writer changed the text to `text`: the loglet moves over the change, so that undo
    /// keeps it (UNDO-6). The cursor stays in its line, if the line is still there.
    pub fn rebase_to(&mut self, text: &str) {
        self.commit();
        self.set_lines(text);
        self.row = self.row.min(self.lines.len() - 1);
        self.col = self.col.min(self.len(self.row));
        self.absorb();
    }

    /// Go on with a stored loglet whose head text is `head`. The text now may differ: the loglet
    /// moves over the difference (UNDO-4, UNDO-6).
    pub fn resume(&mut self, log: Loglet, head: &str) {
        self.history = History { log, logged: head.to_string(), ..Default::default() };
        self.absorb();
    }

    /// The loglet and the text at its head, for the file of the document (UNDO-4).
    pub fn history(&mut self) -> (Loglet, String) {
        self.commit();
        (self.history.log.clone(), self.history.logged.clone())
    }

    /// Empty the text and keep the loglet: the clear can be undone.
    pub fn clear_text(&mut self) {
        self.set_lines("");
        (self.row, self.col, self.anchor) = (0, 0, None);
        self.touch();
    }

    /// The text now differs from the head of the loglet by the change of another writer. A
    /// loglet that does not fit the old text goes (UNDO-11).
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
