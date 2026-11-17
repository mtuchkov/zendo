//! The notes list and the search over the notes: what is selected, the query, and its matches
//! (ADR 0008). The lists do not see the app.
use crate::docs::notes::Note;
use crate::docs::search::{self, Group};

#[derive(Default)]
pub struct Lists {
    /// The selected note of the notes list.
    note: usize,
    query: String,
    results: Vec<Group>,
    /// The selected match of the search.
    result: usize,
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

    pub fn query(&self) -> &str {
        &self.query
    }

    /// Change the query; the caller searches again.
    pub fn edit_query(&mut self, edit: impl FnOnce(&mut String)) {
        edit(&mut self.query);
    }

    /// The matches of the query in the notes; `?` asks the agent and matches nothing.
    pub fn search(&mut self, notes: &[Note]) {
        self.results = if self.query.starts_with('?') { vec![] } else { search::search(notes, &self.query) };
        self.result = 0;
    }

    pub fn results(&self) -> &[Group] {
        &self.results
    }

    pub fn result(&self) -> usize {
        self.result
    }

    /// The selected match.
    pub fn picked(&self) -> Option<&Group> {
        self.results.get(self.result)
    }

    pub fn result_down(&mut self) {
        if !self.results.is_empty() {
            self.result = (self.result + 1).min(self.results.len() - 1);
        }
    }

    pub fn result_up(&mut self) {
        self.result = self.result.saturating_sub(1);
    }
}

#[cfg(test)]
#[path = "lists_tests.rs"]
mod tests;
