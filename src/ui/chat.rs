//! The chat pane: the thread and the input.
use super::*;

pub(crate) fn draw_panel(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    f.render_widget(Block::new().style(Style::new().bg(t.panel)), area);
    // A thin rule on the left edge separates the panel from the note; the session buttons are on
    // the right edge.
    let inner = Rect { x: area.x + 3, y: area.y, width: area.width.saturating_sub(5 + BUTTON_W + 1), height: area.height };
    draw_sessions(f, app, t, Rect { x: area.right().saturating_sub(BUTTON_W + 1), width: BUTTON_W, ..area });
    // A terminal shows in place of the thread and the input (TERM-37, TERM-40).
    let body = Rect { y: inner.y + 2, height: inner.height.saturating_sub(2), ..inner };
    app.cli.resize_pane(body.height, body.width);
    if let Some(n) = app.cli.pane {
        return draw_pane_terminal(f, app, t, n, Rect { height: 1, ..inner }, body);
    }
    let w = inner.width as usize;
    // The input grows with its text, up to a quarter of the panel; then it scrolls.
    let in_w = (inner.width as usize).saturating_sub(4).max(10);
    app.input_width = in_w;
    let in_rows = app.input.segments(in_w).len().max(1);
    let in_h = in_rows.min((inner.height as usize / 4).max(1)) as u16;
    let slash_h = if app.slash.open { app.slash.hits.len().clamp(1, crate::app::AT_ROWS) } else { 0 };
    let at_h = app.at.as_ref().map_or(slash_h, |p| p.hits.len().clamp(1, crate::app::AT_ROWS)) as u16;
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
    } else if app.slash.open {
        draw_slash(f, t, picks, &app.slash);
    }

    let dim = Style::new().fg(t.dim);
    let scope = if app.scope() == ALL_KEY { "ALL NOTES".to_string() } else { format!("THIS {}", app.file_word().to_uppercase()) };
    let agent = app.profile().map_or("AGENT".to_string(), |p| p.name.to_uppercase());
    let left = agent.as_str();
    let pad = w.saturating_sub(left.len() + scope.len());
    f.render_widget(Paragraph::new(Line::from(vec![Span::styled(left, dim.add_modifier(Modifier::BOLD)), Span::raw(" ".repeat(pad)), Span::styled(scope, dim)])), head);

    let (mut lines, blocks) = panel_lines(app, t, w);
    app.transcript.blocks = blocks;
    if let Some(down) = &app.agent_down {
        lines.extend(markdown::wrap(vec![Span::styled(format!("{down}. Run `zendo doctor` in a terminal to see why, then ask again. Log: .zendo/agent.log"), Style::new().fg(t.del))], w, "", ""));
    } else if app.agent_name.is_none() && app.agent.is_some() {
        lines.push(Line::styled(format!("starting {} {}", short_agent(&app.cfg.agent), SPIN[(app.last_edit.elapsed().as_millis() / 300) as usize % 4]), dim));
    }
    app.regions.push((crate::text::select::Region::Panel, body));
    draw_transcript(f, app, t, body, lines, w);
    draw_input(f, app, t, input, in_h, in_w);
}

/// The thread, from the bottom up. While the transcript has the keys, the view follows its cursor,
/// and the cursor row and the selection show.
fn draw_transcript(f: &mut Frame, app: &mut App, t: &Theme, body: Rect, lines: Vec<Line<'static>>, w: usize) {
    let h = body.height as usize;
    app.transcript.rows = lines.iter().map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect()).collect();
    let on = app.transcript.on && app.focus == Focus::Claude;
    let max = lines.len().saturating_sub(h);
    let tr = &app.transcript;
    let bottom = max - app.panel_scroll.min(max);
    let start = if on { bottom.min(tr.row).max((tr.row + 1).saturating_sub(h)).min(max) } else { bottom };
    app.panel_scroll = max - start;
    let sel = delta::mix(t.bg, t.acc2, 0.42);
    let view: Vec<Line> = lines
        .into_iter()
        .enumerate()
        .skip(start)
        .take(h)
        .map(|(i, l)| match tr.selection() {
            Some(((r1, c1), (r2, c2))) if on && i >= r1 && i <= r2 => {
                let n = tr.rows[i].trim_end().chars().count();
                paint(l, if i == r1 { c1 } else { 0 }, if i == r2 { c2 + 1 } else { n.max(1) }, sel)
            }
            _ if on && i == tr.row => tint(l, cursor_line(t), w),
            _ => l,
        })
        .collect();
    f.render_widget(Paragraph::new(view), body);
    let shown = |(a, ..): &&(usize, usize, String)| *a >= start && *a < start + h;
    let play = app.transcript.blocks.iter().filter(shown).map(|(a, _, c)| (Rect { x: body.x + (w as u16).saturating_sub(2), y: body.y + (a - start) as u16, width: 2, height: 1 }, c.clone())).collect::<Vec<_>>();
    app.play_buttons = play;
    if on && tr.row >= start && tr.row < start + h {
        let x: usize = tr.rows[tr.row].chars().take(tr.col).map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0)).sum();
        f.set_cursor_position((body.x + (x as u16).min(body.width.saturating_sub(1)), body.y + (tr.row - start) as u16));
    }
}

/// The input box: the text with its chips, or a hint while it is empty.
fn draw_input(f: &mut Frame, app: &App, t: &Theme, input: Rect, in_h: u16, in_w: usize) {
    let key = app.key();
    f.render_widget(Block::new().style(Style::new().bg(t.bg)), input);
    let box_in = Rect { x: input.x + 1, y: input.y + 1, width: input.width.saturating_sub(2), height: in_h };
    let focused = app.focus == Focus::Claude && !app.transcript.on;
    let mark = Span::styled("› ", Style::new().fg(if focused { t.acc } else { t.dim }));
    let text_area = Rect { x: box_in.x + 2, width: box_in.width.saturating_sub(2), ..box_in };
    if app.input.is_empty() {
        let hint = input_hint(app, &key);
        let room = text_area.width as usize;
        let hint = if hint.width() <= room { hint } else { hint.chars().take(room.saturating_sub(1)).collect::<String>() + "…" };
        f.render_widget(Paragraph::new(Line::from(vec![mark, Span::styled(hint, Style::new().fg(t.dim))])), Rect { height: 1, ..box_in });
        if focused {
            f.set_cursor_position((text_area.x, text_area.y));
        }
        return;
    }
    let segs = app.input.segments(in_w);
    let (vrow, vx) = app.input.cursor_visual(&segs, in_w);
    let top = vrow.saturating_sub(in_h as usize - 1);
    let chip = chip_styles(t);
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

fn input_hint(app: &App, key: &str) -> String {
    let busy = app.threads.get(key).is_some_and(|th| th.busy.is_some());
    if app.cli.running(app.state.session.min(crate::app::SESSIONS - 1)) {
        "goes to Claude Code, which queues it while it works".to_string()
    } else if busy {
        "enter queues the next question · esc stops".to_string()
    } else if app.ask_context.is_some() {
        format!("ask about {}, or press enter", if app.ask_about.is_empty() { "it" } else { app.ask_about })
    } else if app.scope() == ALL_KEY {
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

/// The slash commands of the agent that match: 4 at a time, a page that follows the selection.
fn draw_slash(f: &mut Frame, t: &Theme, area: Rect, s: &crate::app::Slash) {
    let dim = Style::new().fg(t.dim);
    if s.hits.is_empty() {
        f.render_widget(Paragraph::new(Line::styled("  no command of the agent matches", dim)), area);
        return;
    }
    let rows = crate::app::AT_ROWS;
    let start = s.sel / rows * rows;
    let w = area.width as usize;
    let lines: Vec<Line> = s
        .hits
        .iter()
        .enumerate()
        .skip(start)
        .take(rows)
        .map(|(i, c)| {
            let name = format!("/{}", c.name);
            let about: String = c.description.chars().take(w.saturating_sub(name.width() + 6)).collect();
            let line = Line::from(vec![Span::styled(if i == s.sel { "› " } else { "  " }, Style::new().fg(t.acc)), Span::styled(name, Style::new().fg(t.fg)), Span::styled(format!("  {about}"), dim)]);
            if i == s.sel { line.style(Style::new().bg(t.sel)) } else { line }
        })
        .collect();
    f.render_widget(Paragraph::new(lines), area);
}

/// The width of a session button: ` F6 `.
const BUTTON_W: u16 = 4;

/// The buttons of the agent sessions, one under the other: the active one in the accent color,
/// the others in the second accent, as the badges of the modes. Under them, after a line, the
/// buttons of the terminals in the color of the text: `F5`, then each `[>]` (TERM-14, TERM-15).
fn draw_sessions(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    use crate::app::SESSIONS;
    let w = area.width as usize;
    let agents = (0..SESSIONS).map(|n| (area.y + 2 + 2 * n as u16, n));
    let rule = area.y + 2 + 2 * SESSIONS as u16;
    let terms = (0..crate::session::TERMINALS).filter(|i| *i == 0 || app.cli.running(crate::session::shell(*i)));
    let terms = terms.enumerate().map(|(k, i)| (rule + 2 + 2 * k as u16, crate::session::shell(i)));
    if rule < area.bottom() {
        f.render_widget(Paragraph::new("─".repeat(w)).style(Style::new().fg(t.dim)), Rect { y: rule, height: 1, ..area });
    }
    for (y, n) in agents.chain(terms).filter(|(y, _)| *y < area.bottom()) {
        let r = Rect { y, height: 1, ..area };
        let bg = match n {
            n if n >= SESSIONS => t.fg,
            n if n == app.state.session => t.acc,
            _ => t.acc2,
        };
        let bg = pulse_bg(app, t, n, bg);
        let label = button_label(app, n);
        f.render_widget(Paragraph::new(Line::styled(format!("{label:^w$}"), Style::new().bg(bg).fg(t.bg).add_modifier(Modifier::BOLD))), r);
        app.session_buttons.push((r, n));
    }
}
