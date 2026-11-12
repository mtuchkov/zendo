//! Cursor moves.
use super::*;

impl Editor {
    pub fn left(&mut self) {
        if self.col > 0 { self.col -= 1 } else if self.row > 0 { self.row -= 1; self.col = self.len(self.row) }
        self.want_x = None;
    }

    pub fn right(&mut self) {
        if self.col < self.len(self.row) { self.col += 1 } else if self.row + 1 < self.lines.len() { self.row += 1; self.col = 0 }
        self.want_x = None;
    }

    pub(super) fn word_left_col(&self) -> usize {
        let chars: Vec<char> = self.lines[self.row].chars().collect();
        let mut c = self.col;
        while c > 0 && chars[c - 1].is_whitespace() { c -= 1 }
        while c > 0 && !chars[c - 1].is_whitespace() { c -= 1 }
        c
    }

    pub fn word_left(&mut self) {
        if self.col == 0 { self.left() } else { self.col = self.word_left_col() }
        self.want_x = None;
    }

    pub fn word_right(&mut self) {
        let chars: Vec<char> = self.lines[self.row].chars().collect();
        if self.col >= chars.len() { self.right(); return }
        let mut c = self.col;
        while c < chars.len() && !chars[c].is_whitespace() { c += 1 }
        while c < chars.len() && chars[c].is_whitespace() { c += 1 }
        self.col = c;
        self.want_x = None;
    }

    pub fn home(&mut self) { self.col = 0; self.want_x = None }

    pub fn end(&mut self) { self.col = self.len(self.row); self.want_x = None }

    /// Move by visual rows, keeping the column.
    pub fn vertical(&mut self, delta: isize, width: usize) {
        let segs = self.segments(width);
        let (vr, vx) = self.cursor_visual(&segs, width);
        let x = *self.want_x.get_or_insert(vx);
        let target = (vr as isize + delta).clamp(0, segs.len() as isize - 1) as usize;
        let seg = segs[target];
        let chars: Vec<char> = self.lines[seg.line].chars().collect();
        let mut w = 0;
        let mut col = seg.start;
        while col < seg.end {
            let cw = chars[col].width().unwrap_or(0);
            if w + cw > x { break }
            w += cw;
            col += 1;
        }
        // On a wrapped row, the end belongs to the next row.
        if col == seg.end && seg.end < chars.len() && col > seg.start { col -= 1 }
        self.row = seg.line;
        self.col = col;
    }
}
