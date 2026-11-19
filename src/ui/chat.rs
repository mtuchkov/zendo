//! The chat pane: the thread and the input.
use super::*;

pub(crate) fn draw_panel(f: &mut Frame, app: &mut App, fr: &mut Cache, t: &Theme, area: Rect) {
    f.render_widget(Block::new().style(Style::new().bg(t.panel)), area);
    // A thin rule on the left edge separates the panel from the note. The buttons of the agent
    // sessions are in the first row, also above a terminal; on the right edge above the ones of
    // the terminals only when the row does not fit the width of the chat (NAMES-8).
    let width = area.width.saturating_sub(5 + BUTTON_W + 1);
    let side = Rect { x: area.right().saturating_sub(BUTTON_W + 1), width: BUTTON_W, ..area };
    let rows = match agent_row(app, width as usize) {
        Some(labels) => draw_agent_row(f, app, fr, t, labels, Rect { x: area.x + 3, width, height: 1, ..area }),
        None => draw_agent_column(f, app, fr, t, side),
    };
    draw_terminals(f, app, fr, t, side, rows == 0);
    let inner = Rect { x: area.x + 3, y: area.y + rows, width, height: area.height.saturating_sub(rows) };
    // A terminal shows in place of the thread and the input (TERM-37, TERM-40).
    let body = Rect { y: inner.y + 2, height: inner.height.saturating_sub(2), ..inner };
    app.cli.resize_pane(body.height, body.width);
    if let Some(n) = app.cli.pane {
        return draw_pane_terminal(f, app, fr, t, n, Rect { height: 1, ..inner }, body);
    }
    let w = inner.width as usize;
    // The input grows with its text, up to a quarter of the panel; then it scrolls.
    let in_w = (inner.width as usize).saturating_sub(4).max(10);
    fr.input_width = in_w;
    let in_rows = app.composer().input().segments(in_w).len().max(1);
    let in_h = in_rows.min((inner.height as usize / 4).max(1)) as u16;
    let slash_h = if app.composer().slash().open { app.composer().slash().hits.len().clamp(1, crate::app::AT_ROWS) } else { 0 };
    let at_h = app.composer().at().map_or(slash_h, |p| p.hits.len().clamp(1, crate::app::AT_ROWS)) as u16;
    let bg_h = super::background::height(app);
    let [head, _, body, bg, _, input, picks] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(bg_h),
        Constraint::Length(1),
        Constraint::Length(in_h + 2),
        Constraint::Length(at_h),
    ])
    .areas(inner);
    if let Some(p) = app.composer().at() {
        draw_at(f, t, picks, p);
    } else if app.composer().slash().open {
        draw_slash(f, t, picks, app.composer().slash());
    }

    let dim = Style::new().fg(t.dim);
    // The line of the open document says what the question is about; only all notes show here (CLAUDE-58).
    let scope = if app.scope() == ALL_KEY { "ALL NOTES" } else { "" };
    let agent = app.profile().map_or("AGENT".to_string(), |p| p.name.to_uppercase());
    let left = agent.as_str();
    let pad = w.saturating_sub(left.len() + scope.len());
    f.render_widget(Paragraph::new(Line::from(vec![Span::styled(left, dim.add_modifier(Modifier::BOLD)), Span::raw(" ".repeat(pad)), Span::styled(scope, dim)])), head);

    let (mut lines, blocks, steps) = panel_lines(app, t, w);
    app.threads.transcript_mut().show_blocks(blocks);
    app.threads.transcript_mut().show_steps(steps);
    if let Some(down) = app.link.down() {
        lines.extend(markdown::wrap(vec![Span::styled(format!("{down}. Run `zendo doctor` in a terminal to see why, then ask again. Log: .zendo/agent.log"), Style::new().fg(t.del))], w, "", ""));
    } else if !app.link.ready() && app.link.running() {
        lines.push(Line::styled(format!("starting {} {}", short_agent(&app.cfg.agent), SPIN[(app.writer.idle().as_millis() / 300) as usize % 4]), dim));
    }
    fr.regions.push((crate::text::select::Region::Panel, body));
    super::thread::draw_transcript(f, app, fr, t, body, lines, w);
    super::background::draw(f, app, fr, t, bg);
    draw_input(f, app, t, input, in_h, in_w);
}

/// The input box: the text with its chips, or a hint while it is empty.
fn draw_input(f: &mut Frame, app: &App, t: &Theme, input: Rect, in_h: u16, in_w: usize) {
    let key = app.key();
    f.render_widget(Block::new().style(Style::new().bg(t.bg)), input);
    let box_in = Rect { x: input.x + 1, y: input.y + 1, width: input.width.saturating_sub(2), height: in_h };
    let focused = app.focus == Focus::Claude && !app.threads.transcript().on();
    let mark = Span::styled("› ", Style::new().fg(if focused { t.acc } else { t.dim }));
    let text_area = Rect { x: box_in.x + 2, width: box_in.width.saturating_sub(2), ..box_in };
    if app.composer().input().is_empty() {
        let hint = input_hint(app, &key);
        let room = text_area.width as usize;
        let hint = if hint.width() <= room { hint } else { hint.chars().take(room.saturating_sub(1)).collect::<String>() + "…" };
        f.render_widget(Paragraph::new(Line::from(vec![mark, Span::styled(hint, Style::new().fg(t.dim))])), Rect { height: 1, ..box_in });
        if focused {
            f.set_cursor_position((text_area.x, text_area.y));
        }
        return;
    }
    let segs = app.composer().input().segments(in_w);
    let (vrow, vx) = app.composer().input().cursor_visual(&segs, in_w);
    let top = vrow.saturating_sub(in_h as usize - 1);
    let chip = chip_styles(t);
    let hot = app.composer().chip_in_focus();
    let gone = app.composer().gone();
    let lines: Vec<Line> = segs
        .iter()
        .skip(top)
        .take(in_h as usize)
        .map(|sg| {
            let text: String = app.composer().input().lines[sg.line].chars().skip(sg.start).take(sg.end - sg.start).collect();
            let hot = hot.filter(|(row, ..)| *row == sg.line).map(|(_, a, b)| (a, b, Style::new().fg(t.bg).bg(t.acc).add_modifier(Modifier::BOLD)));
            Line::from(chip_spans_at(&text, sg.start, Style::new().fg(t.fg), chip, hot, &gone))
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
        let send = app.keys.key(crate::keys::Pane::Chat, "send").map(|k| format!("{k} queues the next question"));
        let stop = app.keys.key(crate::keys::Pane::Chat, "back").map(|k| format!("{k} stops"));
        send.into_iter().chain(stop).collect::<Vec<_>>().join(" · ")
    } else if app.composer().has_context() {
        let about = if app.composer().about().is_empty() { "it" } else { app.composer().about() };
        format!("ask about {about}{}", app.keys.key(crate::keys::Pane::Chat, "send").map(|k| format!(", or press {k}")).unwrap_or_default())
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

/// The shortest part of a name on a button in the row (NAMES-8).
const SHORTEST_NAME: usize = 3;

/// The texts of the buttons of the agent sessions in one row of `room` columns: the key, the name
/// and the mark of the state; the names as long as the row lets them be, not shorter than
/// `SHORTEST_NAME`. None: the row does not fit (NAMES-6, NAMES-8).
fn agent_row(app: &App, room: usize) -> Option<Vec<String>> {
    use crate::app::SESSIONS;
    let names: Vec<String> = (0..SESSIONS).map(|n| app.session_name(n).unwrap_or_default()).collect();
    let labels = |k: usize| -> Vec<String> { (0..SESSIONS).map(|n| format!(" {} ", agent_label(app, n, &names[n].chars().take(k).collect::<String>()))).collect() };
    let longest = names.iter().map(|s| s.chars().count()).max().unwrap_or(0);
    let lengths: Vec<usize> = if longest == 0 { vec![0] } else { (SHORTEST_NAME.min(longest)..=longest).rev().collect() };
    lengths.into_iter().map(labels).find(|l| l.iter().map(|s| s.width() + 1).sum::<usize>() <= room + 1)
}

/// The session that the chat pane shows: the terminal in the chat pane, else the active agent
/// session (CLAUDE-32, TERM-15).
fn shown_session(app: &App) -> usize {
    app.cli.pane.unwrap_or(app.state.session)
}

/// The style of the button of agent session `n`: the one that the chat shows in the accent color,
/// the others in the second accent, as the badges of the modes.
fn agent_style(app: &App, t: &Theme, n: usize) -> Style {
    let bg = pulse_bg(app, t, n, if n == shown_session(app) { t.acc } else { t.acc2 });
    Style::new().bg(bg).fg(t.bg).add_modifier(Modifier::BOLD)
}

/// The buttons of the agent sessions in one row above the header (CLAUDE-32). Returns the rows
/// that it takes.
fn draw_agent_row(f: &mut Frame, app: &mut App, fr: &mut Cache, t: &Theme, labels: Vec<String>, row: Rect) -> u16 {
    let mut x = row.x;
    for (n, text) in labels.into_iter().enumerate() {
        let r = Rect { x, width: text.width() as u16, ..row };
        f.render_widget(Paragraph::new(Line::styled(text, agent_style(app, t, n))), r);
        fr.session_buttons.push((r, n));
        x = r.right() + 1;
    }
    1
}

/// The buttons of the agent sessions on the right edge, one under the other, with their keys and
/// the marks of the states: when the row does not fit (CLAUDE-32, NAMES-8). They take no row of
/// the chat.
fn draw_agent_column(f: &mut Frame, app: &mut App, fr: &mut Cache, t: &Theme, area: Rect) -> u16 {
    let w = area.width as usize;
    for (y, n) in (0..crate::app::SESSIONS).map(|n| (area.y + 2 + 2 * n as u16, n)).filter(|(y, _)| *y < area.bottom()) {
        let r = Rect { y, height: 1, ..area };
        let label = button_label(app, n);
        f.render_widget(Paragraph::new(Line::styled(format!("{label:^w$}"), agent_style(app, t, n))), r);
        fr.session_buttons.push((r, n));
    }
    0
}

/// The buttons of the terminals on the right edge, in the color of the text, and the one in the chat
/// pane in the accent color: `F5`, then each `[>]`, then `[+]` while a `[>]` is free (TERM-53). Under the agent buttons when they are on the right edge, after a line; else from the top of the
/// edge (TERM-14, TERM-15).
fn draw_terminals(f: &mut Frame, app: &mut App, fr: &mut Cache, t: &Theme, area: Rect, under_agents: bool) {
    let w = area.width as usize;
    let rule = area.y + 2 + 2 * crate::app::SESSIONS as u16;
    if under_agents && rule < area.bottom() {
        f.render_widget(Paragraph::new("─".repeat(w)).style(Style::new().fg(t.dim)), Rect { y: rule, height: 1, ..area });
    }
    let top = if under_agents { rule + 2 } else { area.y + 2 };
    let terms: Vec<usize> = (0..crate::session::TERMINALS).filter(|i| *i == 0 || app.cli.running(crate::session::shell(*i))).collect();
    let plus = top + 2 * terms.len() as u16;
    if terms.len() < crate::session::TERMINALS && plus < area.bottom() {
        let r = Rect { y: plus, height: 1, ..area };
        f.render_widget(Paragraph::new(Line::styled(format!("{:^w$}", "[+]"), Style::new().fg(t.fg).add_modifier(Modifier::BOLD))), r);
        fr.new_terminal_button = Some(r);
    }
    for (y, n) in terms.into_iter().enumerate().map(|(k, i)| (top + 2 * k as u16, crate::session::shell(i))).filter(|(y, _)| *y < area.bottom()) {
        let r = Rect { y, height: 1, ..area };
        let bg = pulse_bg(app, t, n, if n == shown_session(app) { t.acc } else { t.fg });
        let label = button_label(app, n);
        f.render_widget(Paragraph::new(Line::styled(format!("{label:^w$}"), Style::new().bg(bg).fg(t.bg).add_modifier(Modifier::BOLD))), r);
        fr.session_buttons.push((r, n));
    }
}
