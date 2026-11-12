//! Write mode: each row on the screen as a span of the source. A row of a joined paragraph can hold
//! the end of one source line and the start of the next; the row with the cursor shows its span as
//! source, so the wrap of the paragraph stays as in read mode.
use super::*;

/// One row on the screen and the source chars on it. The end of a source line inside the row is a
/// space at the position after its last char.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RowSpan {
    /// The row on the screen, the head of the note included.
    pub row: usize,
    /// The space before the source chars: the indent of a continued list item.
    pub indent: usize,
    pub chars: Vec<((usize, usize), char)>,
    /// The position after the last char.
    pub end: (usize, usize),
}

/// The spans of the rows that have a start: from the start of each row to the start of the next
/// one, but never into a line that starts on a later row (a hidden line, the next block).
pub fn spans(lines: &[String], starts: &[(usize, (usize, usize), usize)], line_rows: &[usize]) -> Vec<RowSpan> {
    starts
        .iter()
        .enumerate()
        .map(|(k, &(row, start, indent))| {
            let next = starts.get(k + 1).map_or((lines.len(), 0), |s| s.1);
            let (mut chars, (mut l, mut c)) = (Vec::new(), start);
            while (l, c) < next && l < lines.len() {
                let line: Vec<char> = lines[l].chars().collect();
                if c < line.len() {
                    chars.push(((l, c), line[c]));
                    c += 1;
                } else if l + 1 < lines.len() && line_rows.get(l + 1).is_some_and(|r| *r <= row) {
                    chars.push(((l, c), ' '));
                    (l, c) = (l + 1, 0);
                } else {
                    break;
                }
            }
            RowSpan { row, indent, chars, end: (l, c) }
        })
        .collect()
}

impl RowSpan {
    /// The source text of the row.
    pub fn text(&self) -> String {
        let s: String = self.chars.iter().map(|(_, ch)| *ch).collect();
        format!("{}{}", " ".repeat(self.indent), s.trim_end())
    }

    /// The column of a source position on the row.
    pub fn x_of(&self, pos: (usize, usize)) -> usize {
        self.indent + self.chars.iter().take_while(|(p, _)| *p < pos).map(|(_, ch)| crate::text::editor::char_width(*ch)).sum::<usize>()
    }

    /// The source position at a column: the char there, or the last place on the row.
    pub fn at_x(&self, x: usize) -> (usize, usize) {
        let mut w = self.indent;
        for (p, ch) in &self.chars {
            let cw = crate::text::editor::char_width(*ch);
            if w + cw > x {
                return *p;
            }
            w += cw;
        }
        match self.chars.last() {
            // A row that breaks at a space: the place before the next row.
            Some((p, ch)) if ch.is_whitespace() => *p,
            _ => self.end,
        }
    }
}

/// The span with the cursor: the last one that starts at or before it.
pub fn cursor_span(spans: &[RowSpan], pos: (usize, usize)) -> Option<usize> {
    spans.iter().rposition(|s| s.chars.first().map_or(s.end, |c| c.0) <= pos).or((!spans.is_empty()).then_some(0))
}

impl App {
    /// Up or down by rows on the screen, as write mode shows them; the editor's own rows when the
    /// screen has none yet.
    pub(crate) fn write_vertical(&mut self, delta: isize) {
        let spans = &self.write_rows;
        let e = &mut self.editor;
        let Some(k) = cursor_span(spans, (e.row, e.col)) else { return e.vertical(delta, self.text_width) };
        let x = *e.want_x.get_or_insert(spans[k].x_of((e.row, e.col)));
        let target = (k as isize + delta).clamp(0, spans.len() as isize - 1) as usize;
        (e.row, e.col) = spans[target].at_x(x);
    }
}

#[cfg(test)]
#[path = "write_rows_tests.rs"]
mod tests;
