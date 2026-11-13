//! Mouse selection that stays inside one panel; the clipboard is in `clipboard.rs`.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;
use unicode_width::UnicodeWidthStr;

use super::boxtable;
pub use super::clipboard::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Note,
    Delta,
    Panel,
}

#[derive(Clone, Copy, Debug)]
pub struct Selection {
    pub region: Region,
    pub rect: Rect,
    pub anchor: (u16, u16),
    pub head: (u16, u16),
    /// The mouse button is up; the text is final.
    pub done: bool,
    /// Alt was held: the text goes to the chat input as a chip (PANES-44).
    pub chat: bool,
    /// Ctrl or Shift was held: whole lines, also in a table (PANES-43).
    pub free: bool,
}

impl Selection {
    pub fn start(region: Region, rect: Rect, x: u16, y: u16) -> Self {
        let p = clamp(rect, x, y);
        Self { region, rect, anchor: p, head: p, done: false, chat: false, free: false }
    }

    /// The selection with the modifier keys of the press.
    pub fn with(mut self, m: KeyModifiers) -> Self {
        self.hold(m);
        self
    }

    /// A modifier key held at a press, a drag or the release counts for the whole selection.
    pub fn hold(&mut self, m: KeyModifiers) {
        self.chat |= m.contains(KeyModifiers::ALT);
        self.free |= m.intersects(KeyModifiers::CONTROL | KeyModifiers::SHIFT);
    }

    pub fn drag(&mut self, x: u16, y: u16) {
        let (cx, cy) = clamp(self.rect, x, y);
        // Left of the panel on a lower row: the selection ends with the row above.
        self.head = if x < self.rect.x && cy > self.anchor.1 { (self.rect.right().saturating_sub(1), cy - 1) } else { (cx, cy) };
    }

    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    /// Start and end in reading order.
    fn ordered(&self) -> ((u16, u16), (u16, u16)) {
        let (a, b) = (self.anchor, self.head);
        if (a.1, a.0) <= (b.1, b.0) { (a, b) } else { (b, a) }
    }

    /// The selected columns of row `y`, end exclusive.
    fn cols(&self, y: u16) -> Option<(u16, u16)> {
        let (s, e) = self.ordered();
        if y < s.1 || y > e.1 {
            return None;
        }
        let from = if y == s.1 { s.0 } else { self.rect.x };
        let to = if y == e.1 { e.0 + 1 } else { self.rect.right() };
        Some((from, to.min(self.rect.right())))
    }

    /// The rows of the panel as the screen shows them, and the selection in them, when it starts in
    /// a cell of a table with box borders (PANES-34).
    fn in_table(&self, buf: &Buffer) -> Option<(Vec<String>, boxtable::Range)> {
        if self.free {
            return None;
        }
        let rows: Vec<String> = (self.rect.y..self.rect.bottom()).map(|y| screen_row(buf, self.rect, y)).collect();
        let at = |(x, y): (u16, u16)| ((y - self.rect.y) as usize, (x - self.rect.x) as usize);
        let range = boxtable::resolve(&rows, at(self.anchor), at(self.head));
        range.cols.is_some().then_some((rows, range))
    }

    pub fn paint(&self, buf: &mut Buffer, bg: Color, fg: Color) {
        if let Some((rows, range)) = self.in_table(buf) {
            for (i, (a, b)) in (0..rows.len()).filter_map(|i| range.span(i, 0).map(|s| (i as u16, s))) {
                for x in a as u16..b as u16 {
                    if let Some(c) = buf.cell_mut(Position::new(self.rect.x + x, self.rect.y + i)) {
                        c.set_bg(bg).set_fg(fg);
                    }
                }
            }
            return;
        }
        let (s, e) = self.ordered();
        for y in s.1..=e.1 {
            let Some((from, to)) = self.cols(y) else { continue };
            for x in from..to {
                if let Some(c) = buf.cell_mut(Position::new(x, y)) {
                    c.set_bg(bg).set_fg(fg);
                }
            }
        }
    }

    /// The text under the selection, as the screen shows it.
    pub fn text(&self, buf: &Buffer) -> String {
        if let Some((rows, range)) = self.in_table(buf) {
            return range.text(&rows);
        }
        let (s, e) = self.ordered();
        let mut rows = Vec::new();
        for y in s.1..=e.1 {
            let Some((from, to)) = self.cols(y) else { continue };
            let mut row = String::new();
            let mut x = from;
            while x < to {
                let sym = buf.cell(Position::new(x, y)).map(|c| c.symbol()).unwrap_or(" ");
                row.push_str(sym);
                // A wide character covers the next cell too.
                x += sym.width().max(1) as u16;
            }
            rows.push(row.trim_end().to_string());
        }
        // Drop the common left margin, so indented panels copy cleanly.
        let margin = rows.iter().skip(1).filter(|r| !r.is_empty()).map(|r| r.len() - r.trim_start().len()).min().unwrap_or(0);
        let mut out = String::new();
        for (i, r) in rows.iter().enumerate() {
            if i > 0 {
                out.push('\n');
                out.push_str(r.get(margin.min(r.len() - r.trim_start().len())..).unwrap_or(r));
            } else {
                out.push_str(r);
            }
        }
        out.trim_end().to_string()
    }
}

/// Row `y` of `rect`, one char for each cell: a wide char and the cell that it covers are one char.
fn screen_row(buf: &Buffer, rect: Rect, y: u16) -> String {
    let mut row = String::new();
    let mut x = rect.x;
    while x < rect.right() {
        let sym = buf.cell(Position::new(x, y)).map(|c| c.symbol()).filter(|s| !s.is_empty()).unwrap_or(" ");
        row.push_str(sym);
        x += sym.width().max(1) as u16;
    }
    row
}

fn clamp(r: Rect, x: u16, y: u16) -> (u16, u16) {
    (x.clamp(r.x, r.right().saturating_sub(1)), y.clamp(r.y, r.bottom().saturating_sub(1)))
}

#[cfg(test)]
#[path = "select_tests.rs"]
mod tests;
