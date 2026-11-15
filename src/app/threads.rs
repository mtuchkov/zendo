//! The chat threads, and how the chat pane shows them: the transcript cursor, the pinned thread,
//! and the scroll (ADR 0008). The threads do not see the app.
use super::{Thread, Transcript};
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
    details: bool,
    /// The chat pane is open.
    shown: bool,
}

impl Threads {
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
        let max = total.saturating_sub(h);
        let bottom = max - self.scroll.min(max);
        let row = self.transcript.row;
        let start = if on { bottom.min(row).max((row + 1).saturating_sub(h)).min(max) } else { bottom };
        self.scroll = max - start;
        start
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
