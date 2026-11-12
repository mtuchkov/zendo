//! The cursor stays on the same char when the mode changes: the read cursor is a row and a char of
//! the formatted text, the write cursor a line and a char of the source.
use super::*;
use crate::text::align::{align, joined, split, to_shown, to_source};

/// The rendered rows of the source line `line`, and the source lines that share them (a joined
/// paragraph): `(first row, end row, first line, last line)`.
fn group(rows: &[usize], total: usize, line: usize) -> (usize, usize, usize, usize) {
    let v = rows[line];
    let first = (0..=line).rev().take_while(|k| rows[*k] == v).last().unwrap_or(line);
    let last = (line..rows.len()).take_while(|k| rows[*k] == v).last().unwrap_or(line);
    let end = rows.get(last + 1).copied().unwrap_or(total).max(v + 1);
    (v, end, first, last)
}

/// A source line that read mode does not show: a blank, the title, a line of only tags.
fn unseen(src: &[&str], k: usize) -> bool {
    let t = src[k].trim();
    t.is_empty() || (k == 0 && t.starts_with("# ")) || t.split_whitespace().all(|w| w.len() > 1 && w.starts_with('#') && !w.starts_with("##"))
}

/// The source lines of a group without the ones read mode does not show at its start.
fn seen(src: &[&str], first: usize, last: usize) -> usize {
    (first..=last).find(|k| !unseen(src, *k)).unwrap_or(first)
}

/// The source line and char under a cursor of the formatted text.
pub fn shown_to_source(shown: &[String], rows: &[usize], src: &[&str], (row, col): (usize, usize)) -> (usize, usize) {
    let Some(line) = rows.iter().rposition(|r| *r <= row) else { return (0, 0) };
    let (v, end, first, last) = group(rows, shown.len(), line);
    let first = seen(src, first, last);
    let (view, view_starts) = joined(&shown[v..end.min(shown.len())]);
    let (source, src_starts) = joined(&src[first..=last]);
    let i = view_starts.get(row - v).copied().unwrap_or(0) + col;
    let j = to_source(&align(&view, &source), i, source.len());
    let (k, c) = split(&src_starts, j);
    (first + k, c)
}

/// The formatted row and char of a source line and char.
pub fn source_to_shown(shown: &[String], rows: &[usize], src: &[&str], (line, col): (usize, usize)) -> (usize, usize) {
    let Some(&_) = rows.get(line) else { return (rows.last().copied().unwrap_or(0), 0) };
    let (v, end, first, last) = group(rows, shown.len(), line);
    let first = seen(src, first, last).min(line);
    let (view, view_starts) = joined(&shown[v..end.min(shown.len())]);
    let (source, src_starts) = joined(&src[first..=last]);
    let j = src_starts[line - first] + col;
    let i = to_shown(&align(&view, &source), j).min(view.len().saturating_sub(1));
    let (k, c) = split(&view_starts, i);
    (v + k, c)
}

impl App {
    /// Read → write: the editor cursor on the char of the read cursor.
    pub(crate) fn cursor_to_editor(&mut self) {
        let src: Vec<String> = self.editor.lines.clone();
        let src: Vec<&str> = src.iter().map(String::as_str).collect();
        let (line, col) = shown_to_source(&self.read_text, &self.read_rows, &src, (self.read_cursor, self.read_col));
        self.editor.row = line.min(self.editor.lines.len().saturating_sub(1));
        self.editor.col = col.min(self.editor.lines[self.editor.row].chars().count());
    }

    /// Write → read: the read cursor on the char of the editor cursor, in the text as it is now.
    pub(crate) fn cursor_to_read(&mut self) {
        let Some(note) = self.note().cloned() else { return };
        let (lines, rows, _) = crate::ui::read_lines(self, &note, self.text_width, self.theme());
        let shown: Vec<String> = lines.iter().map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect()).collect();
        let src: Vec<&str> = note.body.lines().collect();
        let (row, col) = source_to_shown(&shown, &rows, &src, (self.editor.row, self.editor.col));
        self.read_cursor = row;
        self.read_col = col;
        self.read_want = col;
    }
}

#[cfg(test)]
#[path = "cursor_map_tests.rs"]
mod tests;
