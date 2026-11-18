//! The notes list and search results.
use super::*;

pub(crate) fn draw_notes(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let w = 40.min(area.width / 2);
    let [left, right] = Layout::horizontal([Constraint::Length(w), Constraint::Min(1)]).areas(area);
    f.render_widget(Block::new().style(Style::new().bg(t.panel)), left);
    let inner = Rect { x: left.x, y: left.y, width: left.width, height: left.height };
    let dim = Style::new().fg(t.dim);
    let rows = note_rows(app, t, inner.width as usize);
    // Linked-from block at the bottom.
    let mut foot: Vec<Line<'static>> = Vec::new();
    if let Some(sel) = app.files.notes().get(app.lists.note()) {
        let from: Vec<&Note> = app.files.notes().iter().filter(|n| n.links_to(sel)).collect();
        if !from.is_empty() {
            foot.push(Line::styled("─".repeat(inner.width as usize), Style::new().fg(t.line)));
            foot.push(Line::styled("  LINKED FROM", dim));
            for n in from.iter().take(4) {
                foot.push(Line::styled(format!("  {}", n.title), Style::new().fg(t.acc2)));
            }
        }
    }
    let list_h = (inner.height as usize).saturating_sub(foot.len() + 1);
    let sel_row = rows.iter().position(|(_, i)| *i == Some(app.lists.note())).unwrap_or(0);
    let start = sel_row.saturating_sub(list_h.saturating_sub(3));
    let mut lines: Vec<Line> = vec![Line::raw("")];
    lines.extend(rows.into_iter().skip(start).take(list_h).map(|(l, _)| l));
    if app.files.notes().is_empty() {
        let press = app.keys.key(crate::keys::Pane::Notes, "new").map(|k| format!(" Press {k}.")).unwrap_or_default();
        lines.push(Line::styled(format!("  No notes yet.{press}"), dim));
    }
    f.render_widget(Paragraph::new(lines), inner);
    if !foot.is_empty() {
        let fh = foot.len() as u16;
        f.render_widget(Paragraph::new(foot), Rect { y: inner.y + inner.height - fh, height: fh, ..inner });
    }

    // Preview.
    if let Some(n) = app.files.notes().get(app.lists.note()).cloned() {
        let col = column(Rect { y: right.y + 1, height: right.height.saturating_sub(1), ..right }, app.column_width());
        let (lines, _, _) = read_lines(app, &n, col.width as usize, t);
        f.render_widget(Paragraph::new(lines.into_iter().take(col.height as usize).collect::<Vec<_>>()), col);
    }
}

/// The notes in groups by age (today, this week, older); each row knows its note.
fn note_rows(app: &App, t: &Theme, width: usize) -> Vec<(Line<'static>, Option<usize>)> {
    let today = Local::now().date_naive();
    let dim = Style::new().fg(t.dim);
    let mut rows: Vec<(Line<'static>, Option<usize>)> = Vec::new();
    let mut group = "";
    for (i, n) in app.files.notes().iter().enumerate() {
        let d = n.modified.date_naive();
        let days = (today - d).num_days();
        let g = if days == 0 { "TODAY" } else if days < 7 { "THIS WEEK" } else { "OLDER" };
        if g != group {
            if !group.is_empty() {
                rows.push((Line::raw(""), None));
            }
            rows.push((Line::styled(format!("  {g}"), dim), None));
            group = g;
        }
        let when = if days == 0 { n.modified.format("%H:%M").to_string() } else if days < 7 { n.modified.format("%a").to_string() } else if d.year() == today.year() { n.modified.format("%-d %b").to_string() } else { n.modified.format("%b %Y").to_string() };
        let sel = i == app.lists.note();
        let marker = if sel { "▸ " } else { "  " };
        // Δ: changed since your last visit that counted.
        let changed = app.state.files.get(&app.rel(&n.path)).and_then(|f| f.seen.as_ref()).is_some_and(|s| s.hash != crate::setup::state::hash(&n.body));
        let when = if changed { format!("Δ {when}") } else { when };
        let room = width.saturating_sub(when.width() + 8);
        let mut title: String = n.title.clone();
        if title.width() > room {
            title = title.chars().take(room.saturating_sub(1)).collect::<String>() + "…";
        }
        let pad = width.saturating_sub(title.width() + when.width() + 6);
        let mut line = Line::from(vec![
            Span::raw("  "),
            Span::styled(marker, Style::new().fg(t.acc)),
            Span::styled(title, Style::new().fg(t.fg)),
            Span::raw(" ".repeat(pad)),
            Span::styled(when, dim),
            Span::raw("  "),
        ]);
        if sel {
            line = line.style(Style::new().bg(t.sel));
        }
        rows.push((line, Some(i)));
    }
    rows
}
