//! A plain-text editor with soft wrap, made for prose.

mod edit;
mod wrap;
mod motion;
mod select;

use unicode_width::UnicodeWidthChar;

/// A tab shows as this many spaces.
pub const TAB_WIDTH: usize = 4;

/// The columns of a char on the screen: a tab is TAB_WIDTH.
pub fn char_width(c: char) -> usize {
    if c == '\t' { TAB_WIDTH } else { c.width().unwrap_or(0) }
}

#[derive(Clone, Debug, Default)]
pub struct Editor {
    pub lines: Vec<String>,
    /// Cursor: logical line and char index in that line.
    pub row: usize,
    pub col: usize,
    /// Remembered visual column for up/down moves.
    pub want_x: Option<usize>,
    pub dirty: bool,
    /// The other end of the selection: (line, char). The cursor is the first end.
    pub anchor: Option<(usize, usize)>,
}

/// One visual row: logical line index and its char range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Seg {
    pub line: usize,
    pub start: usize,
    pub end: usize,
}

impl Editor {
    pub fn new(text: &str) -> Self {
        let mut lines: Vec<String> = text.split('\n').map(str::to_string).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        Self { lines, ..Default::default() }
    }

    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].is_empty()
    }

    /// Empty the editor; give the text.
    pub fn take(&mut self) -> String {
        let t = self.text();
        *self = Editor::new("");
        t
    }

    fn char_at(&self, row: usize, col: usize) -> Option<char> {
        self.lines.get(row)?.chars().nth(col)
    }

    fn text_between(&self, a: (usize, usize), b: (usize, usize)) -> String {
        let mut e = self.clone();
        e.anchor = Some(a);
        e.row = b.0;
        e.col = b.1;
        e.selected_text().unwrap_or_default()
    }

    fn len(&self, row: usize) -> usize {
        self.lines[row].chars().count()
    }

    fn byte(&self, row: usize, col: usize) -> usize {
        self.lines[row].char_indices().nth(col).map(|(b, _)| b).unwrap_or(self.lines[row].len())
    }

    fn touch(&mut self) {
        self.dirty = true;
        self.want_x = None;
    }


    /// Soft-wrap every line into visual rows.
    pub fn segments(&self, width: usize) -> Vec<Seg> {
        let mut out = Vec::new();
        for (i, line) in self.lines.iter().enumerate() {
            for (start, end) in wrap_line(line, width) {
                out.push(Seg { line: i, start, end });
            }
        }
        out
    }

    /// Cursor position as (visual row, x).
    pub fn cursor_visual(&self, segs: &[Seg], _width: usize) -> (usize, usize) {
        let chars: Vec<char> = self.lines[self.row].chars().collect();
        let mut best = 0;
        for (i, s) in segs.iter().enumerate() {
            if s.line == self.row && self.col >= s.start && (self.col < s.end || self.col == s.end && (s.end == chars.len())) {
                best = i;
                break;
            }
            if s.line == self.row {
                best = i;
            }
        }
        let s = segs[best];
        let x = chars[s.start..self.col.min(chars.len()).max(s.start)].iter().map(|c| char_width(*c)).sum();
        (best, x)
    }
}

/// Greedy word wrap of one line into char ranges.
pub fn wrap_line(line: &str, width: usize) -> Vec<(usize, usize)> {
    let width = width.max(4);
    let chars: Vec<char> = line.chars().collect();
    if chars.is_empty() {
        return vec![(0, 0)];
    }
    let mut out = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let mut w = 0;
        let mut end = start;
        let mut last_space = None;
        while end < chars.len() {
            let cw = char_width(chars[end]);
            if w + cw > width { break }
            if chars[end] == ' ' { last_space = Some(end) }
            w += cw;
            end += 1;
        }
        if end < chars.len()
            && let Some(sp) = last_space.filter(|sp| *sp > start) {
                end = sp + 1;
            }
        out.push((start, end));
        start = end;
    }
    out
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "select_tests.rs"]
mod select_tests;
