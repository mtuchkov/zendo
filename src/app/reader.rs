//! The view of the note: the read cursor, the selection, and the scroll (ADR 0008). The reader does
//! not see the app: its methods take the rendered rows of the last frame.
use crate::text::boxtable::{self, Range};
use crate::text::softstop::{Dir, SoftStop};
use std::time::Instant;

/// A position in the rendered rows: (row, col in chars).
pub type Pos = (usize, usize);

#[derive(Default)]
pub struct Reader {
    /// The first row on the screen; write mode scrolls with it too.
    scroll: usize,
    row: usize,
    col: usize,
    /// The column that the cursor wants on up and down.
    want: usize,
    /// The other end of the selection.
    anchor: Option<Pos>,
    /// The stop at the edge of a table cell (PANES-42).
    stop: SoftStop,
    /// The screen row of the cursor at the last draw: a mode change keeps it.
    cursor_y: usize,
    /// After a mode change: the screen row that the cursor keeps at the next draw.
    keep_y: Option<usize>,
}

fn len_of(text: &[String], row: usize) -> usize {
    text.get(row).map_or(0, |l| l.trim_end().chars().count())
}

impl Reader {
    pub fn row(&self) -> usize {
        self.row
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn cursor(&self) -> Pos {
        (self.row, self.col)
    }

    pub fn anchor(&self) -> Option<Pos> {
        self.anchor
    }

    pub fn scroll(&self) -> usize {
        self.scroll
    }

    pub fn cursor_y(&self) -> usize {
        self.cursor_y
    }

    /// The cursor on a row; the column stays.
    pub fn to_row(&mut self, row: usize) {
        self.row = row;
    }

    /// The cursor on a char; up and down keep its column.
    pub fn place(&mut self, row: usize, col: usize) {
        (self.row, self.col, self.want) = (row, col, col);
    }

    /// The cursor on a row, three rows under the top of the view.
    pub fn jump(&mut self, row: usize) {
        self.row = row;
        self.scroll = row.saturating_sub(3);
    }

    /// Back in a document: the view and the cursor where the user left them, without a selection.
    pub fn set_place(&mut self, scroll: usize, row: usize, col: usize) {
        (self.scroll, self.row, self.col, self.want, self.anchor) = (scroll, row, col, col, None);
    }

    /// Another document: the first row, without a selection. The column stays.
    pub fn to_top(&mut self) {
        (self.scroll, self.row, self.anchor) = (0, 0, None);
    }

    /// Another document: the first char, without a selection.
    pub fn to_start(&mut self) {
        self.to_top();
        self.col = 0;
    }

    /// Start a selection at the cursor, when there is none.
    pub fn select(&mut self) {
        self.anchor.get_or_insert((self.row, self.col));
    }

    /// A selection from `anchor` to the cursor.
    pub fn select_from(&mut self, anchor: Pos) {
        self.anchor = Some(anchor);
    }

    pub fn unselect(&mut self) {
        self.anchor = None;
    }

    pub fn toggle_select(&mut self) {
        self.anchor = if self.anchor.is_some() { None } else { Some((self.row, self.col)) };
    }

    /// Move the cursor by rows; the column stays where it can.
    pub fn move_rows(&mut self, text: &[String], by: isize) {
        let last = text.len().saturating_sub(1) as isize;
        self.row = (self.row as isize + by).clamp(0, last.max(0)) as usize;
        self.col = self.want.min(len_of(text, self.row).saturating_sub(1));
    }

    /// Move the cursor by one char, across rows.
    pub fn move_col(&mut self, text: &[String], right: bool) {
        let len = len_of(text, self.row);
        if !right {
            if self.col > 0 {
                self.col = self.col.min(len).saturating_sub(1);
            } else if self.row > 0 {
                self.row -= 1;
                self.col = len_of(text, self.row).saturating_sub(1);
            }
        } else if self.col + 1 < len {
            self.col += 1;
        } else if self.row + 1 < text.len() {
            self.row += 1;
            self.col = 0;
        }
        self.want = self.col;
    }

    /// The char `back` chars before the cursor is a space, or there is none.
    fn space(&self, text: &[String], back: usize) -> bool {
        text.get(self.row).and_then(|r| r.chars().nth(self.col - back)).is_none_or(char::is_whitespace)
    }

    /// One char to the side; true when the cursor moved.
    fn step(&mut self, text: &[String], right: bool, steps: &mut u32) -> bool {
        let before = self.cursor();
        self.move_col(text, right);
        *steps += 1;
        self.cursor() != before
    }

    /// The cursor to the start of the next word, or of this or the previous word. The result is
    /// the count of the moves by one char.
    pub fn move_word(&mut self, text: &[String], right: bool) -> u32 {
        let mut n = 0;
        if right {
            // Over the rest of this word, then over the spaces.
            while !self.space(text, 0) && self.step(text, true, &mut n) {}
            while self.space(text, 0) && self.step(text, true, &mut n) {}
            return n;
        }
        // Over the spaces before, then to the first char of the word.
        while self.step(text, false, &mut n) && self.space(text, 0) {}
        while self.col > 0 && !self.space(text, 1) {
            self.move_col(text, false);
            n += 1;
        }
        n
    }

    /// The wheel: the view and the cursor move by `n` rows.
    pub fn wheel(&mut self, up: bool, n: usize) {
        self.scroll = if up { self.scroll.saturating_sub(n) } else { self.scroll + n };
        self.row = if up { self.row.saturating_sub(n) } else { self.row + n };
    }

    /// The selection, start before end, both ends in it.
    pub fn selection(&self) -> Option<(Pos, Pos)> {
        let a = self.anchor?;
        let c = self.cursor();
        Some(if a <= c { (a, c) } else { (c, a) })
    }

    /// The selection, when it starts in a cell of a table with box borders (PANES-34).
    pub fn table(&self, text: &[String]) -> Option<Range> {
        boxtable::in_table(text, self.anchor?, self.cursor())
    }

    /// A selection in a table: the cursor stops at the edge of its cell, and two quick presses go
    /// on to the next cell (PANES-42). `before` is the cursor before the key `act`.
    pub fn soft_stop(&mut self, text: &[String], act: &str, before: Pos) {
        let (Some(dir), Some(anchor)) = (Dir::of(act), self.anchor) else { return };
        (self.row, self.col) = self.stop.step(text, anchor, before, self.cursor(), dir, Instant::now());
        if dir.horizontal() {
            self.want = self.col;
        }
    }

    /// The selected rows; without a selection, the paragraph under the cursor.
    pub fn selection_rows(&self, text: &[String]) -> (usize, usize) {
        if let Some(((a, _), (b, _))) = self.selection() {
            return (a, b);
        }
        let blank = |i: usize| text.get(i).is_none_or(|l| l.trim().is_empty());
        let (mut a, mut b) = (self.row, self.row);
        while a > 0 && !blank(a - 1) {
            a -= 1;
        }
        while b + 1 < text.len() && !blank(b + 1) {
            b += 1;
        }
        (a, b)
    }

    /// The first row of the selection, and the selected text of each row.
    pub fn selected_rows(&self, text: &[String]) -> Option<(usize, Vec<String>)> {
        let ((r1, c1), (r2, c2)) = self.selection()?;
        let part = |(i, line): (usize, &String)| {
            let r = r1 + i;
            let chars: Vec<char> = line.trim_end().chars().collect();
            let from = if r == r1 { c1.min(chars.len()) } else { 0 };
            let to = if r == r2 { (c2 + 1).min(chars.len()) } else { chars.len() };
            chars[from..to.max(from)].iter().collect::<String>()
        };
        let lines = text.get(r1..=r2.min(text.len().saturating_sub(1))).unwrap_or(&[]);
        Some((r1, lines.iter().enumerate().map(part).collect()))
    }

    /// The text of the selection; without a selection, of the paragraph under the cursor.
    pub fn selection_text(&self, text: &[String]) -> String {
        if let Some(t) = self.table(text) {
            return t.text(text).trim().to_string();
        }
        if let Some((_, parts)) = self.selected_rows(text) {
            return parts.join("\n").trim().to_string();
        }
        let last = text.len().saturating_sub(1);
        let (a, b) = self.selection_rows(text);
        let rows: Vec<&str> = text.get(a..=b.min(last)).unwrap_or(&[]).iter().map(|s| s.trim_end()).collect();
        rows.join("\n").trim().to_string()
    }

    /// A mode change: the cursor keeps its row on the screen at the next draw.
    pub fn keep_screen_row(&mut self) {
        self.keep_y = Some(self.cursor_y);
    }

    /// The draw of read mode: the cursor stays in the text, and the view of `h` rows follows it.
    pub fn fit(&mut self, text: &[String], h: usize) {
        self.row = self.row.min(text.len().saturating_sub(1));
        self.follow(self.row, h, text.len());
        self.col = self.col.min(len_of(text, self.row).saturating_sub(1));
    }

    /// The view of `h` rows follows the cursor row `vrow` of `total` rows, with a margin.
    pub fn follow(&mut self, vrow: usize, h: usize, total: usize) {
        if let Some(y) = self.keep_y.take() {
            self.scroll = vrow.saturating_sub(y);
        }
        let margin = 3.min(h / 4);
        if vrow < self.scroll + margin {
            self.scroll = vrow.saturating_sub(margin);
        } else if vrow + margin >= self.scroll + h {
            self.scroll = vrow + margin + 1 - h;
        }
        self.scroll = self.scroll.min(total.saturating_sub(h));
        self.cursor_y = vrow.saturating_sub(self.scroll);
    }

    /// The float and the screen behind it each have a view: the cursor, the selection, the scroll.
    pub(crate) fn swap_view(&mut self, other: &mut Reader) {
        use std::mem::swap;
        swap(&mut self.scroll, &mut other.scroll);
        swap(&mut self.row, &mut other.row);
        swap(&mut self.col, &mut other.col);
        swap(&mut self.want, &mut other.want);
        swap(&mut self.anchor, &mut other.anchor);
        swap(&mut self.cursor_y, &mut other.cursor_y);
        swap(&mut self.keep_y, &mut other.keep_y);
    }
}

#[cfg(test)]
#[path = "reader_tests.rs"]
mod tests;
