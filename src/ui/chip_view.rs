//! The chips of the chat input on the screen, the line of the open document above the input, and
//! the chip window (CLAUDE-48 to CLAUDE-54).
use super::*;
use crate::app::chips::{CLOSE, OPEN, chip_ranges};
use ratatui::widgets::{Padding, Wrap};

/// How much of the color of the chip the background of the chip window takes (CLAUDE-49).
pub const CHIP_DIM: f32 = 0.3;

/// The place of each chip in the rows of the chat input that show in `area`; a row is a line of
/// the input and the chars `start..end` of it that show. A chip that the wrap cuts has a place in
/// each of its rows.
pub(crate) fn chip_rects(area: Rect, rows: &[(&str, usize, usize)]) -> Vec<(Rect, String)> {
    let width = |cs: &[char]| cs.iter().map(|c| if *c == OPEN || *c == CLOSE { 1 } else { crate::text::editor::char_width(*c) }).sum::<usize>() as u16;
    let at = |(i, (line, start, end)): (usize, &(&str, usize, usize))| {
        let chars: Vec<char> = line.chars().collect();
        let piece = move |(a, b): (usize, usize)| {
            let (from, to) = (a.max(*start), b.min(*end));
            (from < to).then(|| (Rect { x: area.x + width(&chars[*start..from]), y: area.y + i as u16, width: width(&chars[from..to]), height: 1 }, chars[a..b].iter().collect()))
        };
        chip_ranges(line).into_iter().filter_map(piece).collect::<Vec<_>>()
    };
    rows.iter().enumerate().flat_map(at).filter(|(r, _)| r.x < area.right()).collect()
}

/// The places of the chips of the transcript on the screen: the rows from `start` show in `body`.
pub(crate) fn transcript_chip_rects(chips: &[(usize, u16, u16, String)], body: Rect, start: usize) -> Vec<(Rect, String)> {
    let shown = |(row, ..): &&(usize, u16, u16, String)| *row >= start && *row < start + body.height as usize;
    chips.iter().filter(shown).map(|(row, x, w, c)| (Rect { x: body.x + x, y: body.y + (row - start) as u16, width: *w, height: 1 }, c.clone())).collect()
}

/// `[x] open document` or `[ ] open document` above the chat input; its place for the mouse.
pub(crate) fn draw_doc_line(f: &mut Frame, app: &App, t: &Theme, area: Rect) -> Option<Rect> {
    let on = app.doc_line()?;
    let (mark, color) = if on { ("[x]", t.acc) } else { ("[ ]", t.dim) };
    let text = vec![Span::styled(mark, Style::new().fg(color)), Span::styled(" open document", Style::new().fg(if on { t.fg } else { t.dim }))];
    let r = Rect { x: area.x + 1, width: 17.min(area.width.saturating_sub(1)), height: 1, ..area };
    f.render_widget(Paragraph::new(Line::from(text)), r);
    Some(r)
}

/// The chip window: its bottom right corner at the top right corner of the chat input, as wide as
/// its longest line up to the text column, as high as its text up to the top of the transcript.
pub(crate) fn draw_chip_view(f: &mut Frame, app: &App, fr: &mut Cache, t: &Theme) {
    let (Some(v), Some(input)) = (app.composer().view(), fr.input_rect) else { return };
    let top = fr.regions.iter().find(|(r, _)| *r == Region::Panel).map_or(0, |(_, b)| b.y);
    let close = " [ esc close ] ";
    let longest = v.lines.iter().map(|(l, _)| l.width()).max().unwrap_or(0).max(v.label.width() + close.width() + 3);
    let inner = (longest as u16).min(app.column_width()).min(input.right().saturating_sub(4)).max(1);
    let rows: usize = v.lines.iter().map(|(l, _)| l.width().div_ceil(inner as usize).max(1)).sum();
    let h = (rows as u16 + 2).min(input.y.saturating_sub(top));
    if h < 3 {
        return;
    }
    let w = inner + 4;
    let r = Rect { x: input.right() - w, y: input.y - h, width: w, height: h };
    let color = chip_styles(t)[v.style].bg.unwrap_or(t.acc2);
    let bg = delta::mix(t.panel, color, CHIP_DIM);
    let block = Block::bordered()
        .border_style(Style::new().fg(color))
        .style(Style::new().bg(bg).fg(t.fg))
        .padding(Padding::horizontal(1))
        .title_top(Line::from(Span::styled(format!(" {} ", v.label), Style::new().fg(t.bg).bg(color))))
        .title_top(Line::from(Span::styled(close, Style::new().fg(t.dim))).right_aligned());
    let lines: Vec<Line> = v.lines.iter().map(|(l, dim)| Line::styled(l.clone(), Style::new().fg(if *dim { t.dim } else { t.fg }))).collect();
    f.render_widget(Clear, r);
    f.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: false }).scroll((v.scroll as u16, 0)), r);
    fr.chip_view_rect = Some(r);
}
