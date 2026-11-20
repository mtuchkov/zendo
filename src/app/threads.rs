//! The chat threads, and how the chat pane shows them: the transcript cursor, the pinned thread,
//! and the scroll (ADR 0008). The threads do not see the app.
use super::frozen::Freeze;
use super::{Refreeze, Rows, Thread, Transcript};
use std::collections::HashMap;

#[derive(Default)]
pub struct Threads {
    map: HashMap<String, Thread>,
    /// The cursor in the chat transcript.
    transcript: Transcript,
    /// A thread shown regardless of the note, e.g. a question across all notes.
    pinned: Option<String>,
    /// The rows between the end of the thread and the bottom of the view.
    scroll: usize,
    /// The rows of the thread at the last draw.
    total: Option<usize>,
    /// The rows of the transcript while it has the keys or a selection (PANES-51).
    freeze: Freeze,
    details: bool,
    /// The chat pane is open.
    shown: bool,
    /// The last number of a question (`Outgoing::n`).
    asked: usize,
}

impl Threads {
    /// A new number for a question; a new thread does not start the numbers again (RESP-1).
    pub fn number(&mut self) -> usize {
        self.asked += 1;
        self.asked
    }

    /// The chat pane is open.
    pub fn shown(&self) -> bool {
        self.shown
    }

    pub fn show(&mut self) {
        self.shown = true;
    }

    pub fn hide(&mut self) {
        self.shown = false;
    }

    pub fn get(&self, key: &str) -> Option<&Thread> {
        self.map.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Thread> {
        self.map.get_mut(key)
    }

    /// The thread of `key`; a new one when there is none.
    pub fn entry(&mut self, key: &str) -> &mut Thread {
        self.map.entry(key.to_string()).or_default()
    }

    pub fn remove(&mut self, key: &str) -> Option<Thread> {
        self.map.remove(key)
    }

    /// The thread of `old` gets the key `new`.
    pub fn rename(&mut self, old: &str, new: String) {
        if let Some(t) = self.map.remove(old) {
            self.map.insert(new, t);
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.map.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &Thread> {
        self.map.values()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Thread> {
        self.map.values_mut()
    }

    /// The agent answers in the thread of `key`.
    pub fn busy(&self, key: &str) -> bool {
        self.map.get(key).is_some_and(|t| t.busy.is_some())
    }

    pub fn transcript(&self) -> &Transcript {
        &self.transcript
    }

    pub fn transcript_mut(&mut self) -> &mut Transcript {
        &mut self.transcript
    }

    pub fn pinned(&self) -> Option<&str> {
        self.pinned.as_deref()
    }

    pub fn pin(&mut self, key: &str) {
        self.pinned = Some(key.to_string());
    }

    pub fn unpin(&mut self) {
        self.pinned = None;
    }

    /// The float and the screen behind it each have a pinned thread, or none.
    pub(crate) fn swap_pinned(&mut self, other: &mut Option<String>) {
        std::mem::swap(&mut self.pinned, other);
    }

    pub fn details(&self) -> bool {
        self.details
    }

    /// The view shows the end of the thread.
    pub fn to_end(&mut self) {
        self.scroll = 0;
    }

    /// Scroll by `n` rows: up is away from the end.
    pub fn scroll_by(&mut self, up: bool, n: usize) {
        self.scroll = if up { self.scroll + n } else { self.scroll.saturating_sub(n) };
    }

    /// The draw of the chat pane: the first row of `total` rows in a view of `h` rows. While the
    /// transcript has the keys (`on`), the view follows its cursor.
    pub fn fit(&mut self, total: usize, h: usize, on: bool) -> usize {
        // A view away from the end keeps its rows when rows come or go below them (PANES-58).
        if let Some(before) = self.total.replace(total)
            && self.scroll > 0
        {
            self.scroll = (self.scroll + total).saturating_sub(before);
        }
        let max = total.saturating_sub(h);
        let bottom = max - self.scroll.min(max);
        let row = self.transcript.cursor().0;
        let start = if on { bottom.min(row).max((row + 1).saturating_sub(h)).min(max) } else { bottom };
        self.scroll = max - start;
        start
    }

    /// The first row of the view that `fit` chose for `total` rows in a view of `h` rows.
    pub fn view_start(&self, total: usize, h: usize) -> usize {
        let max = total.saturating_sub(h);
        max - self.scroll.min(max)
    }

    /// A click on row `y` and column `x` of a view of `h` rows: the transcript gets the keys, with
    /// the cursor under the pointer, so the view does not move (PANES-46).
    pub fn click(&mut self, y: usize, h: usize, x: usize) {
        let (row, col) = self.under(y, h, x);
        self.transcript.enter_at(row, col);
    }

    /// A drag to row `y` and column `x` of a view of `h` rows: the selection grows to the char
    /// there (PANES-50).
    pub fn drag(&mut self, y: usize, h: usize, x: usize) {
        let (row, col) = self.under(y, h, x);
        self.transcript.drag_to(row, col);
    }

    /// The row and the char of the transcript at row `y` and column `x` of a view of `h` rows.
    fn under(&self, y: usize, h: usize, x: usize) -> (usize, usize) {
        let rows = self.transcript.rows();
        let row = (self.view_start(rows.len(), h) + y).min(rows.len().saturating_sub(1));
        (row, rows.get(row).map_or(0, |r| crate::text::select::col_at(r, x)))
    }

    /// The rows to draw of the thread `key` at `width`, and the number of new rows below them.
    /// While `hold`, the rows stay as they were (PANES-51); the transcript gets them.
    pub fn view_rows(&mut self, key: &str, width: usize, hold: bool, live: Rows) -> (Rows, usize) {
        self.freeze.view((key, width), hold, live, &mut self.transcript)
    }

    /// The next draw freezes the rows again (PANES-54, PANES-56).
    pub fn refreeze(&mut self, why: Refreeze) {
        self.freeze.refreeze(why);
    }
}

impl<Q: AsRef<str> + ?Sized> std::ops::Index<&Q> for Threads {
    type Output = Thread;

    fn index(&self, key: &Q) -> &Thread {
        &self.map[key.as_ref()]
    }
}

#[cfg(test)]
#[path = "threads_tests.rs"]
mod tests;
