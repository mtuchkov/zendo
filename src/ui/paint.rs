//! Backgrounds: the cursor line, the selection, and the area past the word limit.
use super::*;

/// The row where the word after the limit is, in a list of words per row.
pub(crate) fn over_limit(words: &[usize], limit: usize) -> Option<usize> {
    if limit == 0 {
        return None;
    }
    let mut sum = 0;
    words.iter().position(|w| {
        sum += w;
        sum > limit
    })
}

/// From the row of the word after the limit down, the whole pane is slightly red; the left margin
/// says why, and how to split the document.
pub(crate) fn draw_over_limit(f: &mut Frame, app: &App, t: &Theme, area: Rect, col: Rect, y0: u16, words: usize) {
    let y0 = y0.max(area.y);
    if y0 >= area.bottom() {
        return;
    }
    let red = delta::mix(t.bg, t.del, 0.10);
    f.render_widget(Block::new().style(Style::new().bg(red)), Rect { y: y0, height: area.bottom() - y0, ..area });
    let margin = col.x.saturating_sub(area.x);
    if margin < 14 {
        return;
    }
    let w = (margin - 3) as usize;
    let key = app.keys.first(crate::keys::Pane::Read, "split");
    let text = vec![
        Line::styled(format!("{words} words"), Style::new().fg(t.del).add_modifier(Modifier::BOLD)),
        Line::styled(format!("over {}", app.word_limit), Style::new().fg(t.del)),
        Line::raw(""),
    ];
    let mut text = text;
    text.extend(markdown::wrap(vec![Span::styled(format!("{key} split it with Claude, keep the altitude"), Style::new().fg(t.dim))], w, "", ""));
    let h = (text.len() as u16).min(area.bottom() - y0);
    f.render_widget(Paragraph::new(text), Rect { x: area.x + 1, y: y0, width: margin - 2, height: h });
}

/// The background of the cursor line: clearly brighter (or darker, on a light theme) than the page.
pub fn cursor_line(t: &Theme) -> Color {
    delta::mix(t.bg, t.fg, 0.16)
}

/// The same line with a background on every span, across the whole width.
pub(crate) fn tint(l: Line<'static>, bg: Color, width: usize) -> Line<'static> {
    let mut spans = l.spans.into_iter().map(|s| { let st = s.style.bg(bg); s.style(st) }).collect::<Vec<_>>();
    let used: usize = spans.iter().map(|s| s.content.width()).sum();
    if used < width {
        spans.push(Span::styled(" ".repeat(width - used), Style::new().bg(bg)));
    }
    Line { spans, style: l.style.bg(bg), alignment: l.alignment }
}

/// Give the chars `from..to` of a line a background; past the end, one cell shows it.
pub fn paint(l: Line<'static>, from: usize, to: usize, bg: Color) -> Line<'static> {
    let mut out = Vec::new();
    let mut pos = 0;
    for s in l.spans {
        let chars: Vec<char> = s.content.chars().collect();
        let n = chars.len();
        let (a, b) = (from.clamp(pos, pos + n) - pos, to.clamp(pos, pos + n) - pos);
        if a >= b {
            out.push(s);
        } else {
            let part = |x: usize, y: usize| chars[x..y].iter().collect::<String>();
            if a > 0 {
                out.push(Span::styled(part(0, a), s.style));
            }
            out.push(Span::styled(part(a, b), s.style.bg(bg)));
            if b < n {
                out.push(Span::styled(part(b, n), s.style));
            }
        }
        pos += n;
    }
    if from >= pos && to > from {
        out.push(Span::styled(" ", Style::new().bg(bg)));
    }
    Line { spans: out, style: l.style, alignment: l.alignment }
}
