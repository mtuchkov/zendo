//! The chat pane: the thread and the input.
use super::*;

pub(crate) fn draw_panel(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    f.render_widget(Block::new().style(Style::new().bg(t.panel)), area);
    // A thin rule on the left edge separates the panel from the note.
    let inner = Rect { x: area.x + 3, y: area.y, width: area.width.saturating_sub(5), height: area.height };
    let w = inner.width as usize;
    let key = app.key();
    // The input grows with its text, up to a quarter of the panel; then it scrolls.
    let in_w = (inner.width as usize).saturating_sub(4).max(10);
    app.input_width = in_w;
    let in_rows = app.input.segments(in_w).len().max(1);
    let in_h = in_rows.min((inner.height as usize / 4).max(1)) as u16;
    let at_h = app.at.as_ref().map_or(0, |p| p.hits.len().clamp(1, crate::app::AT_ROWS)) as u16;
    let [head, _, body, _, input, picks] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(in_h + 2),
        Constraint::Length(at_h),
    ])
    .areas(inner);
    if let Some(p) = &app.at {
        draw_at(f, t, picks, p);
    }

    let dim = Style::new().fg(t.dim);
    let scope = if key == ALL_KEY { "ALL NOTES".to_string() } else { format!("THIS {}", app.file_word().to_uppercase()) };
    let agent = app.profile().map_or("AGENT".to_string(), |p| p.name.to_uppercase());
    let left = agent.as_str();
    let pad = w.saturating_sub(left.len() + scope.len());
    f.render_widget(Paragraph::new(Line::from(vec![Span::styled(left, dim.add_modifier(Modifier::BOLD)), Span::raw(" ".repeat(pad)), Span::styled(scope, dim)])), head);

    let mut lines = panel_lines(app, t, w);
    if let Some(down) = &app.agent_down {
        lines.extend(markdown::wrap(vec![Span::styled(format!("{down}. Run `zendo doctor` in a terminal to see why, then ask again. Log: .zendo/agent.log"), Style::new().fg(t.del))], w, "", ""));
    } else if app.agent_name.is_none() && app.agent.is_some() {
        lines.push(Line::styled(format!("starting {} {}", short_agent(&app.cfg.agent), SPIN[(app.last_edit.elapsed().as_millis() / 300) as usize % 4]), dim));
    }
    app.regions.push((crate::text::select::Region::Panel, body));
    let h = body.height as usize;
    let max = lines.len().saturating_sub(h);
    app.panel_scroll = app.panel_scroll.min(max);
    let start = max - app.panel_scroll;
    let view: Vec<Line> = lines.into_iter().skip(start).take(h).collect();
    f.render_widget(Paragraph::new(view), body);
    draw_input(f, app, t, input, in_h, in_w);
}

/// The input box: the text with its chips, or a hint while it is empty.
fn draw_input(f: &mut Frame, app: &App, t: &Theme, input: Rect, in_h: u16, in_w: usize) {
    let key = app.key();
    let pending = app.threads.get(&key).and_then(|th| th.pending()).is_some();
    f.render_widget(Block::new().style(Style::new().bg(t.bg)), input);
    let box_in = Rect { x: input.x + 1, y: input.y + 1, width: input.width.saturating_sub(2), height: in_h };
    let focused = app.focus == Focus::Claude;
    let mark = Span::styled("› ", Style::new().fg(if focused { t.acc } else { t.dim }));
    let text_area = Rect { x: box_in.x + 2, width: box_in.width.saturating_sub(2), ..box_in };
    if pending || app.input.is_empty() {
        let hint = input_hint(app, &key, pending);
        let room = text_area.width as usize;
        let hint = if hint.width() <= room { hint } else { hint.chars().take(room.saturating_sub(1)).collect::<String>() + "…" };
        f.render_widget(Paragraph::new(Line::from(vec![mark, Span::styled(hint, Style::new().fg(t.dim))])), Rect { height: 1, ..box_in });
        if focused && !pending {
            f.set_cursor_position((text_area.x, text_area.y));
        }
        return;
    }
    let segs = app.input.segments(in_w);
    let (vrow, vx) = app.input.cursor_visual(&segs, in_w);
    let top = vrow.saturating_sub(in_h as usize - 1);
    let chip = Style::new().fg(t.bg).bg(t.acc2);
    let hot = app.chip_in_focus();
    let lines: Vec<Line> = segs
        .iter()
        .skip(top)
        .take(in_h as usize)
        .map(|sg| {
            let text: String = app.input.lines[sg.line].chars().skip(sg.start).take(sg.end - sg.start).collect();
            let hot = hot.filter(|(row, ..)| *row == sg.line).map(|(_, a, b)| (a, b, Style::new().fg(t.bg).bg(t.acc).add_modifier(Modifier::BOLD)));
            Line::from(chip_spans_at(&text, sg.start, Style::new().fg(t.fg), chip, hot))
        })
        .collect();
    f.render_widget(Paragraph::new(Line::from(mark)), Rect { height: 1, width: 2, ..box_in });
    f.render_widget(Paragraph::new(lines), text_area);
    if focused {
        f.set_cursor_position((text_area.x + (vx as u16).min(text_area.width.saturating_sub(1)), text_area.y + (vrow - top) as u16));
    }
}

fn input_hint(app: &App, key: &str, pending: bool) -> String {
    let busy = app.threads.get(key).is_some_and(|th| th.busy.is_some());
    if pending {
        "answer the proposal above: a, e or x".to_string()
    } else if busy {
        "esc to stop".to_string()
    } else if app.ask_context.is_some() {
        format!("ask about {}, or press enter", if app.ask_about.is_empty() { "it" } else { app.ask_about })
    } else if key == ALL_KEY {
        "ask about all notes".to_string()
    } else {
        format!("ask about this {}", app.file_word())
    }
}

pub(crate) fn short_agent(cmd: &str) -> String {
    let last = cmd.split_whitespace().find(|p| !p.starts_with('-') && *p != "npx").unwrap_or(cmd);
    last.rsplit('/').next().unwrap_or(last).split('@').next().unwrap_or(last).to_string()
}

/// The file suggestions of `@`: 4 at a time, a page that follows the selection.
fn draw_at(f: &mut Frame, t: &Theme, area: Rect, p: &crate::app::AtPick) {
    let dim = Style::new().fg(t.dim);
    if p.hits.is_empty() {
        f.render_widget(Paragraph::new(Line::styled(format!("  no {} matches @{}", if p.templates { "template" } else { "file" }, p.query), dim)), area);
        return;
    }
    let rows = crate::app::AT_ROWS;
    let start = p.sel / rows * rows;
    let w = area.width as usize;
    let lines: Vec<Line> = p
        .hits
        .iter()
        .enumerate()
        .skip(start)
        .take(rows)
        .map(|(i, rel)| {
            let text = rel.display().to_string();
            let room = w.saturating_sub(4);
            let text = if text.width() > room { format!("…{}", text.chars().rev().take(room.saturating_sub(1)).collect::<Vec<_>>().into_iter().rev().collect::<String>()) } else { text };
            let page = if i == start && p.hits.len() > rows { format!(" {}/{}", start / rows + 1, p.hits.len().div_ceil(rows)) } else { String::new() };
            let line = Line::from(vec![Span::styled(if i == p.sel { "› " } else { "  " }, Style::new().fg(t.acc)), Span::styled(text, Style::new().fg(t.fg)), Span::styled(page, dim)]);
            if i == p.sel { line.style(Style::new().bg(t.sel)) } else { line }
        })
        .collect();
    f.render_widget(Paragraph::new(lines), area);
}
