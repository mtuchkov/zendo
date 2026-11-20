//! The mouse: the selection that it makes, the pane edge that it drags, and the text that goes to
//! the clipboard or to the chat (ADR 0008). The pointer does not see the app.
use crate::app::Focus;
use crate::text::select::{Region, Selection};
use ratatui::crossterm::event::KeyModifiers;
use std::time::{Duration, Instant};

/// A pane edge in a drag: the pane, its fixed edge, and whether it grows to the right.
pub type Grab = (Focus, u16, bool);

#[derive(Default)]
pub struct Pointer {
    sel: Option<Selection>,
    /// The selection is one of the rows of the chat transcript, not of the screen (PANES-50).
    rows: bool,
    /// The pointer in the last drag, also outside the pane: the edge scroll (PANES-52).
    at: (u16, u16),
    grab: Option<Grab>,
    /// Take the text of the selection at the next draw (PANES-44).
    copy_pending: bool,
    /// Text for the clipboard, and for a chip in the chat, until the event loop takes it.
    clip: Option<String>,
    to_chat: Option<String>,
    /// The last text that zendo copied, for paste when the system clipboard cannot be read.
    last_copy: Option<String>,
    /// The time of the last scroll event: a fast run of them is a trackpad.
    last_scroll: Option<Instant>,
}

impl Pointer {
    pub fn sel(&self) -> Option<Selection> {
        self.sel
    }

    /// A press starts a selection, or with `None` ends the one before.
    pub fn press(&mut self, sel: Option<Selection>) {
        (self.sel, self.rows) = (sel, false);
    }

    /// The selection of the press is one of the rows of the chat transcript (PANES-50).
    pub fn in_rows(&mut self) {
        self.rows = true;
    }

    pub fn rows(&self) -> bool {
        self.rows
    }

    /// The pointer in the last drag.
    pub fn at(&self) -> (u16, u16) {
        self.at
    }

    /// A selection grows: the button is down.
    pub fn dragging(&self) -> bool {
        self.sel.is_some_and(|s| !s.done)
    }

    pub fn clear(&mut self) {
        self.sel = None;
    }

    pub fn take(&mut self) -> Option<Selection> {
        self.sel.take()
    }

    fn live(&mut self) -> Option<&mut Selection> {
        self.sel.as_mut().filter(|s| !s.done)
    }

    /// The modifier keys of a mouse event count for the selection that it makes (PANES-43, PANES-44).
    pub fn hold(&mut self, m: KeyModifiers) {
        if let Some(s) = self.live() {
            s.hold(m);
        }
    }

    /// A drag moves the end of the selection, until the button is up.
    pub fn drag(&mut self, x: u16, y: u16, m: KeyModifiers) {
        self.at = (x, y);
        self.hold(m);
        if let Some(s) = self.live() {
            s.drag(x, y);
        }
    }

    /// The button is up: the selection is final, and the next draw takes its text; an empty one
    /// goes away. The result is the region of the selection, and whether it goes to the chat.
    pub fn release(&mut self, m: KeyModifiers) -> Option<(Region, bool)> {
        self.hold(m);
        let s = self.sel.as_mut()?;
        if s.is_empty() {
            self.sel = None;
            return None;
        }
        s.done = true;
        self.copy_pending = true;
        Some((s.region, s.chat))
    }

    /// The draw takes the text of a final selection once.
    pub fn take_copy_request(&mut self) -> bool {
        std::mem::take(&mut self.copy_pending)
    }

    /// The draw of the screen behind the float: without the selection, which comes back after it.
    pub(crate) fn set_aside(&mut self) -> (Option<Selection>, bool) {
        (self.sel.take(), std::mem::take(&mut self.copy_pending))
    }

    pub(crate) fn bring_back(&mut self, aside: (Option<Selection>, bool)) {
        (self.sel, self.copy_pending) = aside;
    }

    pub fn grabbed(&self) -> Option<Grab> {
        self.grab
    }

    pub fn grab(&mut self, grab: Grab) {
        self.grab = Some(grab);
    }

    pub fn let_go(&mut self) {
        self.grab = None;
    }

    /// Lines for one scroll event: 3 for a wheel notch, 1 inside a fast run of events (a trackpad).
    pub fn scroll_step(&mut self, now: Instant) -> usize {
        let fast = self.last_scroll.is_some_and(|t| now.duration_since(t) < Duration::from_millis(40));
        self.last_scroll = Some(now);
        if fast { 1 } else { 3 }
    }

    /// Text for the clipboard: the event loop takes it.
    pub fn copy(&mut self, text: String) {
        self.clip = Some(text);
    }

    pub fn clip(&self) -> Option<&str> {
        self.clip.as_deref()
    }

    pub fn take_clip(&mut self) -> Option<String> {
        self.clip.take()
    }

    /// Text for a chip in the chat input: the event loop takes it.
    pub fn send_to_chat(&mut self, text: String) {
        self.to_chat = Some(text);
    }

    pub fn take_to_chat(&mut self) -> Option<String> {
        self.to_chat.take()
    }

    /// The text went to the system clipboard.
    pub fn copied(&mut self, text: String) {
        self.last_copy = Some(text);
    }

    pub fn last_copy(&self) -> Option<String> {
        self.last_copy.clone()
    }
}

#[cfg(test)]
#[path = "pointer_tests.rs"]
mod tests;
