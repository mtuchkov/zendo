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
    if let Some(sel) = app.notes.get(app.notes_sel) {
        let from: Vec<&Note> = app.notes.iter().filter(|n| n.links_to(sel)).collect();
        if !from.is_empty() {
            foot.push(Line::styled("─".repeat(inner.width as usize), Style::new().fg(t.line)));
            foot.push(Line::styled("  LINKED FROM", dim));
            for n in from.iter().take(4) {
                foot.push(Line::styled(format!("  {}", n.title), Style::new().fg(t.acc2)));
            }
        }
    }
    let list_h = (inner.height as usize).saturating_sub(foot.len() + 1);
    let sel_row = rows.iter().position(|(_, i)| *i == Some(app.notes_sel)).unwrap_or(0);
    let start = sel_row.saturating_sub(list_h.saturating_sub(3));
    let mut lines: Vec<Line> = vec![Line::raw("")];
    lines.extend(rows.into_iter().skip(start).take(list_h).map(|(l, _)| l));
    if app.notes.is_empty() {
        lines.push(Line::styled("  No notes yet. Press n.", dim));
    }
    f.render_widget(Paragraph::new(lines), inner);
    if !foot.is_empty() {
        let fh = foot.len() as u16;
        f.render_widget(Paragraph::new(foot), Rect { y: inner.y + inner.height - fh, height: fh, ..inner });
    }

    // Preview.
    if let Some(n) = app.notes.get(app.notes_sel).cloned() {
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
    for (i, n) in app.notes.iter().enumerate() {
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
        let sel = i == app.notes_sel;
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

pub(crate) fn draw_search(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let w = 96.min(area.width.saturating_sub(4));
    let col = Rect { x: area.x + (area.width - w) / 2, y: area.y + 1, width: w, height: area.height.saturating_sub(1) };
    let dim = Style::new().fg(t.dim);
    let boxr = Rect { height: 3, ..col };
    f.render_widget(Block::new().style(Style::new().bg(t.panel)), boxr);
    let count = if app.query.starts_with('?') {
        "enter asks Claude".to_string()
    } else if app.query.trim().is_empty() {
        String::new()
    } else {
        let m: usize = app.results.iter().map(|g| g.count).sum();
        format!("{m} matches · {} notes", app.results.len())
    };
    let q = app.query.to_string();
    let pad = (w as usize).saturating_sub(q.width() + count.width() + 8);
    let line = Line::from(vec![Span::styled("  / ", Style::new().fg(t.acc)), Span::raw(q.clone()), Span::raw(" ".repeat(pad)), Span::styled(count, dim)]);
    f.render_widget(Paragraph::new(line), Rect { y: boxr.y + 1, height: 1, ..boxr });
    f.set_cursor_position((boxr.x + 4 + q.width() as u16, boxr.y + 1));

    let list = Rect { y: col.y + 4, height: col.height.saturating_sub(4), ..col };
    let mut lines: Vec<Line> = Vec::new();
    let mut sel_row = 0;
    for (gi, g) in app.results.iter().enumerate() {
        let n = &app.notes[g.note];
        let sel = gi == app.result_sel;
        if sel {
            sel_row = lines.len();
        }
        let bg = if sel { Style::new().bg(t.sel) } else { Style::new() };
        let when = format!("{} · {} match{}", n.modified.format("%-d %b"), g.count, if g.count == 1 { "" } else { "es" });
        let pad = (w as usize).saturating_sub(n.title.width() + when.width() + 6);
        lines.push(Line::from(vec![
            Span::styled(if sel { "  ▸ " } else { "    " }, Style::new().fg(t.acc)),
            Span::styled(n.title.clone(), Style::new().fg(t.fg).add_modifier(Modifier::BOLD)),
            Span::raw(" ".repeat(pad)),
            Span::styled(when, dim),
            Span::raw("  "),
        ]).style(bg));
        for s in &g.snippets {
            let mut spans = vec![Span::raw("      ")];
            let mut at = 0;
            for (a, b) in &s.hits {
                spans.push(Span::styled(s.text[at..*a].to_string(), dim));
                spans.push(Span::styled(s.text[*a..*b].to_string(), Style::new().fg(t.acc).add_modifier(Modifier::BOLD)));
                at = *b;
            }
            spans.push(Span::styled(s.text[at..].to_string(), dim));
            let mut l = Line::from(spans);
            if sel {
                l = l.style(bg);
            }
            lines.push(l);
        }
        lines.push(Line::raw(""));
    }
    if app.query.starts_with('?') {
        lines.push(Line::styled(format!("    Ask Claude across all notes: {}", app.query[1..].trim()), Style::new().fg(t.fg)));
    } else if app.query.trim().is_empty() || app.results.is_empty() {
        if !app.query.trim().is_empty() {
            lines.push(Line::styled("    Nothing found.", Style::new().fg(t.fg)));
            lines.push(Line::raw(""));
        }
        lines.push(Line::styled("    Start with # to search tags. Start with ? to ask Claude across all notes.", dim));
    }
    let h = list.height as usize;
    let start = sel_row.saturating_sub(h.saturating_sub(4));
    f.render_widget(Paragraph::new(lines.into_iter().skip(start).take(h).collect::<Vec<_>>()), list);
}
