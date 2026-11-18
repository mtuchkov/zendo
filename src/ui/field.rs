//! A one-line text field on the screen: its text, with the selection in the selection color (TEXT-2).
use super::theme::Theme;
use crate::text::field::Field;
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthStr;

/// The spans of a field: the text in `style`, the selection in `sel`.
pub(crate) fn spans(f: &Field, style: Style, sel: Style) -> Vec<Span<'static>> {
    let chars: Vec<char> = f.text().chars().collect();
    let (a, b) = f.selection().unwrap_or((chars.len(), chars.len()));
    let part = |r: std::ops::Range<usize>| chars[r].iter().collect::<String>();
    [(part(0..a), style), (part(a..b), sel), (part(b..chars.len()), style)].into_iter().filter(|(t, _)| !t.is_empty()).map(|(t, s)| Span::styled(t, s)).collect()
}

/// The query line of a search box of width `w` in `boxr`: `/`, the field and its cursor, and `note`
/// at the right end.
pub(crate) fn draw_query(f: &mut Frame, t: &Theme, boxr: Rect, q: &Field, note: Span<'static>) {
    let pad = (boxr.width as usize).saturating_sub(q.width() + note.content.width() + 8);
    let mut line = vec![Span::styled("  / ", Style::new().fg(t.acc))];
    line.extend(spans(q, Style::new(), Style::new().bg(t.sel)));
    line.extend([Span::raw(" ".repeat(pad)), note]);
    f.render_widget(Paragraph::new(Line::from(line)), Rect { y: boxr.y + 1, height: 1, ..boxr });
    f.set_cursor_position((boxr.x + 4 + q.cursor_width() as u16, boxr.y + 1));
}
