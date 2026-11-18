//! The notes list: what is selected (ADR 0008). The list does not see the app.

#[derive(Default)]
pub struct Lists {
    /// The selected note of the notes list.
    note: usize,
}

impl Lists {
    pub fn note(&self) -> usize {
        self.note
    }

    pub fn select_note(&mut self, i: usize) {
        self.note = i;
    }

    /// One note down in a list of `n` notes.
    pub fn note_down(&mut self, n: usize) {
        if n > 0 {
            self.note = (self.note + 1).min(n - 1);
        }
    }

    pub fn note_up(&mut self) {
        self.note = self.note.saturating_sub(1);
    }

    /// The last note of `n` notes.
    pub fn note_bottom(&mut self, n: usize) {
        self.note = n.saturating_sub(1);
    }

    /// The list has `n` notes now: the selection stays in it.
    pub fn keep_note_in(&mut self, n: usize) {
        self.note = self.note.min(n.saturating_sub(1));
    }
}

#[cfg(test)]
#[path = "lists_tests.rs"]
mod tests;
