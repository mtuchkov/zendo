//! The find panel: the query field, the list, and the preview of the selected file beside the list,
//! or under it in a narrow panel (FIND-3, FIND-5, FIND-10, FIND-13).
use super::*;
use crate::app::{FindItem, FindMode};

/// A panel narrower than this shows the preview under the list (FIND-13).
const SIDE_BY_SIDE: u16 = 100;

pub(crate) fn draw_find(f: &mut Frame, app: &mut App, fr: &mut Cache, t: &Theme, area: Rect) {
    let w = area.width.saturating_sub(4).min(160);
    let col = Rect { x: area.x + (area.width - w) / 2, y: area.y, width: w, height: area.height };
    draw_modes(f, app, t, Rect { height: 1, ..col });
    let boxr = Rect { y: col.y + 1, height: 3, ..col };
    f.render_widget(Block::new().style(Style::new().bg(t.panel)), boxr);
    let q = app.files.find().query.clone();
    let hint = match q.chars().next() {
        None => "recent files",
        Some('?') if app.files.find().mode == FindMode::Notes => "enter asks Claude",
        Some('?') => "? asks Claude in Notes",
        Some(_) => "",
    };
    let line = Line::from(vec![Span::styled("  / ", Style::new().fg(t.acc)), Span::raw(q.clone()), Span::raw(" "), Span::styled(hint, Style::new().fg(t.dim))]);
    let row = Rect { y: boxr.y + 1, height: 1, ..boxr };
    f.render_widget(Paragraph::new(line), row);
    fr.find_regex = Some(draw_regex_button(f, app.files.find().regex, app.files.find().bad_regex, t, row));
    f.set_cursor_position((boxr.x + 4 + q.width() as u16, boxr.y + 1));
    let body = Rect { y: col.y + 5, height: col.height.saturating_sub(5), ..col };
    let (list, preview) = if w >= SIDE_BY_SIDE {
        let lw = (w * 9 / 20).max(40);
        (Rect { width: lw, ..body }, Rect { x: body.x + lw + 2, width: w.saturating_sub(lw + 2), ..body })
    } else {
        let lh = (list_lines(app, t, w).len() as u16).clamp(1, body.height / 2);
        (Rect { height: lh, ..body }, Rect { y: body.y + lh + 1, height: body.height.saturating_sub(lh + 1), ..body })
    };
    draw_list(f, app, t, list);
    draw_preview(f, app, t, preview);
}

/// `[.*]` at the right end of a query line: the accent color while on, the error color for an
/// expression that does not compile (FIND-42, FIND-44, FILES-59). Returns its area, for the mouse.
pub(crate) fn draw_regex_button(f: &mut Frame, on: bool, bad: bool, t: &Theme, row: Rect) -> Rect {
    let fg = if bad { t.del } else if on { t.acc } else { t.dim };
    let style = Style::new().fg(fg).add_modifier(if on { Modifier::BOLD } else { Modifier::empty() });
    let r = Rect { x: row.right().saturating_sub(6), width: 4, ..row };
    f.render_widget(Paragraph::new(Line::styled("[.*]", style)), r);
    r
}

/// The modes after the key hint `tab`, in the colors of the session buttons; the number of results
/// of each mode, or `…` while the search runs (FIND-23, FIND-24, FIND-41).
fn draw_modes(f: &mut Frame, app: &App, t: &Theme, row: Rect) {
    let find = app.files.find();
    let mut spans = vec![Span::styled(format!("  {}  ", app.keys.first(crate::keys::Pane::Global, "next_pane")), Style::new().fg(t.dim))];
    for (i, m) in FindMode::ALL.iter().enumerate() {
        let tail = if find.query.trim().is_empty() || find.query.starts_with('?') {
            String::new()
        } else if find.running {
            " …".into()
        } else {
            format!(" {}", find.counts[i])
        };
        let bg = if *m == find.mode { t.acc } else { t.acc2 };
        spans.push(Span::styled(format!(" {}{tail} ", m.name()), Style::new().bg(bg).fg(t.bg).add_modifier(Modifier::BOLD)));
        spans.push(Span::raw(" "));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), row);
}

/// The rows of the list: each file, and under a match in the text its lines; the row of the
/// selection.
fn list_lines(app: &App, t: &Theme, w: u16) -> Vec<(Line<'static>, bool)> {
    let mut lines = Vec::new();
    for (i, item) in app.files.find().items.iter().enumerate() {
        let sel = i == app.files.find().sel;
        lines.push((item_row(app, t, item, sel, w as usize), sel));
        for s in &item.snippets {
            lines.push((snippet_line(s, t), false));
        }
    }
    lines
}

/// A line with a match, the match in the accent color.
fn snippet_line(s: &crate::docs::grep::Snippet, t: &Theme) -> Line<'static> {
    let dim = Style::new().fg(t.dim);
    let mut spans = vec![Span::raw("      ")];
    let mut at = 0;
    for &(a, b) in &s.hits {
        if a < at || b > s.text.len() {
            continue;
        }
        spans.push(Span::styled(s.text[at..a].to_string(), dim));
        spans.push(Span::styled(s.text[a..b].to_string(), Style::new().fg(t.acc).add_modifier(Modifier::BOLD)));
        at = b;
    }
    spans.push(Span::styled(s.text[at..].to_string(), dim));
    Line::from(spans)
}

/// A row: the title, the path, and when the user opened the file (FIND-5).
fn item_row(app: &App, t: &Theme, item: &FindItem, sel: bool, w: usize) -> Line<'static> {
    let path = app.rel(&item.path);
    let when = item.at.map(when_label).unwrap_or_default();
    let pad = w.saturating_sub(item.title.width() + path.width() + when.width() + 8);
    let line = Line::from(vec![
        Span::styled(if sel { "  ▸ " } else { "    " }, Style::new().fg(t.acc)),
        Span::styled(item.title.clone(), Style::new().fg(t.fg).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  {path}"), Style::new().fg(t.dim)),
        Span::raw(" ".repeat(pad)),
        Span::styled(when, Style::new().fg(t.dim)),
    ]);
    if sel { line.style(Style::new().bg(t.sel)) } else { line }
}

/// The time today, the day this week, the date before.
fn when_label(at: chrono::DateTime<Local>) -> String {
    let days = (Local::now().date_naive() - at.date_naive()).num_days();
    let fmt = match days {
        0 => "%H:%M",
        1..=6 => "%a",
        _ => "%-d %b",
    };
    at.format(fmt).to_string()
}

fn draw_list(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    let lines = list_lines(app, t, area.width);
    if lines.is_empty() {
        let empty = if app.files.find().query.is_empty() { "No recent files yet." } else { "Nothing found." };
        f.render_widget(Paragraph::new(Line::styled(format!("    {empty}"), Style::new().fg(t.dim))), area);
        return;
    }
    let sel = lines.iter().position(|(_, s)| *s).unwrap_or(0);
    let skip = (sel + 1).saturating_sub(area.height as usize);
    f.render_widget(Paragraph::new(lines.into_iter().skip(skip).map(|(l, _)| l).collect::<Vec<_>>()), area);
}

/// The selected file as read mode shows it; a match in the text near the top, marked (FIND-3,
/// FIND-10).
fn draw_preview(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    let Some(note) = app.files.find().preview.as_ref() else { return };
    let inner = Rect { x: area.x + 2, width: area.width.saturating_sub(4).min(app.column_width()), ..area };
    let (mut lines, rows, _) = read_lines(app, note, inner.width as usize, t);
    let line = app.files.find().items.get(app.files.find().sel).and_then(|i| i.snippets.first()).map(|s| s.line);
    let row = line.and_then(|l| rows.get(l).copied());
    if let Some(r) = row.filter(|r| *r < lines.len()) {
        let mark = delta::mix(t.bg, t.acc2, 0.42);
        lines[r] = match match_cols(&lines[r], app.files.find().mark.as_ref()) {
            Some((a, b)) => paint(lines[r].clone(), a, b, mark),
            None => tint(lines[r].clone(), cursor_line(t), inner.width as usize),
        };
    }
    // The match near the top, when it is not on the screen from the top of the file.
    let top = row.filter(|r| r + 3 > inner.height as usize).map_or(0, |r| r.saturating_sub(2));
    f.render_widget(Paragraph::new(lines.into_iter().skip(top).collect::<Vec<_>>()), inner);
}

/// The columns of the first match in a row, without regard to letter case.
fn match_cols(l: &Line, mark: Option<&regex::Regex>) -> Option<(usize, usize)> {
    let text: String = l.spans.iter().map(|s| s.content.as_ref()).collect();
    let m = mark?.find(&text).filter(|m| !m.is_empty())?;
    let a = text[..m.start()].chars().count();
    Some((a, a + m.as_str().chars().count()))
}
