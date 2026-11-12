//! The note pane for a code file: the column of the code, and the line numbers on its left. A
//! wrapped line has its number on its first row only.
use super::*;
use crate::text::code::gutter;

/// The line numbers and the text column, `width` chars wide, of a code file of `n` lines.
pub(crate) fn code_column(area: Rect, width: u16, n: usize) -> (Rect, Rect) {
    let all = column(area, width + gutter(n));
    let g = gutter(n).min(all.width / 4);
    (Rect { width: g, ..all }, Rect { x: all.x + g, width: all.width - g, ..all })
}

/// The line numbers of the rows on the screen. `rows` has the first row of each line; the line of
/// `cursor` shows in the text color.
pub(crate) fn draw_gutter(f: &mut Frame, t: &Theme, area: Rect, rows: &[usize], scroll: usize, cursor: Option<usize>) {
    let w = area.width.saturating_sub(1) as usize;
    let here = cursor.map(|c| rows.partition_point(|r| *r <= c).saturating_sub(1));
    let lines: Vec<Line> = (scroll..scroll + area.height as usize)
        .map(|row| match rows.binary_search(&row) {
            Ok(l) => Line::styled(format!("{:>w$} ", l + 1), Style::new().fg(if here == Some(l) { t.fg } else { t.dim })),
            Err(_) => Line::raw(""),
        })
        .collect();
    f.render_widget(Paragraph::new(lines), area);
}
