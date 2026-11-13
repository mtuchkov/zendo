//! The session view: the screen of a Claude Code session, and the bottom line of zendo with the five
//! agent sessions and their states (docs/zendo/sessions.md).
use super::*;
use ratatui::buffer::Buffer;

fn color(c: vt100::Color) -> Color {
    match c {
        vt100::Color::Default => Color::Reset,
        vt100::Color::Idx(i) => Color::Indexed(i),
        vt100::Color::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}

fn style(c: &vt100::Cell) -> Style {
    let marks = [(c.bold(), Modifier::BOLD), (c.dim(), Modifier::DIM), (c.italic(), Modifier::ITALIC), (c.underline(), Modifier::UNDERLINED), (c.inverse(), Modifier::REVERSED)];
    marks.into_iter().filter(|(on, _)| *on).fold(Style::new().fg(color(c.fgcolor())).bg(color(c.bgcolor())), |s, (_, m)| s.add_modifier(m))
}

/// Paint the screen of a session into `area`; the cursor, when the program shows it.
fn paint(s: &vt100::Screen, buf: &mut Buffer, area: Rect) -> Option<(u16, u16)> {
    let cells = (0..area.height).flat_map(|row| (0..area.width).map(move |col| (row, col)));
    for (row, col) in cells {
        let (Some(c), cell) = (s.cell(row, col), &mut buf[(area.x + col, area.y + row)]) else { continue };
        if c.is_wide_continuation() {
            cell.reset();
            continue;
        }
        cell.set_symbol(if c.has_contents() { c.contents() } else { " " });
        cell.set_style(style(c));
    }
    let (row, col) = s.cursor_position();
    let shown = !s.hide_cursor() && s.scrollback() == 0 && row < area.height && col < area.width;
    shown.then_some((area.x + col, area.y + row))
}

/// A session on the full screen: its screen, which the mouse can select from (TERM-46), and the
/// bottom line.
pub(crate) fn draw_cli(f: &mut Frame, app: &mut App, t: &Theme, n: usize, area: Rect) {
    let [screen, bar] = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(area);
    app.regions.push((Region::Panel, screen));
    f.render_widget(Block::new().style(Style::reset()), screen);
    if let Some(Some(pos)) = app.cli.with_screen(n, |s| paint(s, f.buffer_mut(), screen)) {
        f.set_cursor_position(pos);
    }
    draw_cli_bar(f, app, t, n, bar);
}

/// The session view that waits for the end of a turn: the answer of the chat as it streams, and
/// a bottom line that says when Claude Code starts and how to stop the turn (HANDOFF-1, HANDOFF-7,
/// HANDOFF-8).
pub(crate) fn draw_hold(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let [screen, bar] = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(area);
    f.render_widget(Block::new().style(Style::new().bg(t.bg).fg(t.fg)), area);
    let inner = Rect { x: screen.x + 2, width: screen.width.saturating_sub(4), ..screen };
    let (lines, _) = crate::ui::thread::panel_lines(app, t, inner.width as usize);
    let tail = lines.len().saturating_sub(inner.height as usize);
    f.render_widget(Paragraph::new(lines[tail..].to_vec()), inner);
    let mut spans = vec![
        Span::raw(" "),
        Span::styled(" CLAUDE CODE ", Style::new().bg(t.acc).fg(t.bg).add_modifier(Modifier::BOLD)),
        Span::styled(format!("  {}", app.hold_line()), Style::new().fg(t.acc)),
    ];
    spans.extend(key_spans(app, t, "leave", " back to zendo"));
    f.render_widget(Paragraph::new(Line::from(spans)).style(Style::new().bg(t.bg)), bar);
}

/// A terminal in the chat pane: a head line with its name and keys, and its screen (TERM-37).
pub(crate) fn draw_pane_terminal(f: &mut Frame, app: &mut App, t: &Theme, n: usize, head: Rect, body: Rect) {
    use crate::keys::Pane;
    let dim = Style::new().fg(t.dim);
    let keys = [app.keys.hint(Pane::Chat, "handoff", " full screen"), app.keys.hint(Pane::Session, "leave", " back")];
    let line = vec![Span::styled(name(app, n), dim.add_modifier(Modifier::BOLD)), Span::styled(keys.iter().filter(|h| !h.is_empty()).map(|h| format!("  {h}")).collect::<String>(), dim)];
    f.render_widget(Paragraph::new(Line::from(line)), head);
    f.render_widget(Block::new().style(Style::reset()), body);
    let pos = app.cli.with_screen(n, |s| paint(s, f.buffer_mut(), body)).flatten();
    if let Some(pos) = pos.filter(|_| app.focus == Focus::Claude) {
        f.set_cursor_position(pos);
    }
    app.regions.push((crate::text::select::Region::Panel, body));
}

/// The name of session `n`: the F key of an agent session or of the `F5` terminal, `[>]` for
/// another terminal. With other keys than F6 to F10 in the keymap, the agent sessions are `1` to
/// `5` (CLAUDE-35); with another key than F5, the first terminal is `[>]` too (TERM-48).
pub(crate) fn name(app: &App, n: usize) -> String {
    use crate::keys::Pane;
    let f_keys = (1..=crate::app::SESSIONS).all(|k| app.keys.has_default_key(Pane::Global, &format!("session_{k}")));
    match n.checked_sub(crate::app::SESSIONS) {
        None if f_keys => app.keys.first(Pane::Global, &format!("session_{}", n + 1)).to_uppercase(),
        None => (n + 1).to_string(),
        Some(0) if app.keys.has_default_key(Pane::Global, "terminal") => app.keys.first(Pane::Global, "terminal").to_uppercase(),
        Some(_) => "[>]".into(),
    }
}

/// The sessions on the bottom line and the buttons: the agent sessions, the `F5` terminal, and the
/// `[>]` terminals that run.
fn shown(app: &App) -> impl Iterator<Item = usize> + '_ {
    let all = crate::app::SESSIONS + crate::session::TERMINALS;
    (0..all).filter(|n| *n <= crate::app::SESSIONS || app.cli.running(*n))
}

/// The bottom line: the badge, the sessions with their states, and the keys of zendo.
fn draw_cli_bar(f: &mut Frame, app: &App, t: &Theme, n: usize, area: Rect) {
    let badge = if n < crate::app::SESSIONS { " CLAUDE CODE " } else { " TERMINAL " };
    let mut spans = vec![Span::raw(" "), Span::styled(badge, Style::new().bg(t.acc).fg(t.bg).add_modifier(Modifier::BOLD)), Span::raw("  ")];
    for m in shown(app) {
        let key = name(app, m);
        let (text, style) = match app.cli.state(m) {
            Some(s) if m == n => (format!(" {key} {} ", s.word()), Style::new().bg(t.acc2).fg(t.bg).add_modifier(Modifier::BOLD)),
            Some(s) => (format!(" {key} {} ", s.word()), Style::new().fg(if s == crate::session::State::Waits { t.acc } else { t.fg })),
            None => (format!(" {key} "), Style::new().fg(t.dim)),
        };
        // A blinking session fades to the accent and back; its text to the background (ATTN-3).
        let style = match app.blink(m) {
            Some(_) if m == n => style.bg(pulse_bg(app, t, m, t.acc2)),
            Some(k) => style.fg(theme::mix(t.fg, t.bg, k)).bg(pulse_bg(app, t, m, t.bg)),
            None => style,
        };
        spans.push(Span::styled(text, style));
    }
    spans.extend(key_spans(app, t, "leave", " back to zendo"));
    spans.push(Span::styled("   wheel: earlier lines", Style::new().fg(t.dim)));
    // A message of zendo, such as a turn that did not stop (HANDOFF-6).
    if let Some((msg, _)) = &app.flash {
        spans.push(Span::styled(format!("   {msg}"), Style::new().fg(t.acc)));
    }
    f.render_widget(Paragraph::new(Line::from(spans)).style(Style::new().bg(t.bg)), area);
}

/// The key of a session action and what it does, for a bottom line; none without a key (PANES-29).
fn key_spans(app: &App, t: &Theme, action: &str, text: &str) -> Vec<Span<'static>> {
    let Some(k) = app.keys.key(crate::keys::Pane::Session, action) else { return Vec::new() };
    vec![Span::styled(format!("   {k}"), Style::new().fg(t.fg)), Span::styled(text.to_string(), Style::new().fg(t.dim))]
}

/// The text of a session button: its name, and the mark of the state of its program; a terminal
/// has the mark while it runs a command (TERM-16).
pub(crate) fn button_label(app: &App, n: usize) -> String {
    let key = name(app, n);
    match app.cli.state(n) {
        Some(crate::session::State::Idle) if n >= crate::app::SESSIONS => key,
        Some(s) => format!("{key}{}", s.mark()),
        None => key,
    }
}

/// The background of the button of session `n`: `bg`, and while it blinks a mix of `bg` and the
/// accent (the second accent for a button in the accent color) by its pulse (ATTN-3).
pub(crate) fn pulse_bg(app: &App, t: &Theme, n: usize, bg: Color) -> Color {
    let to = if bg == t.acc { t.acc2 } else { t.acc };
    app.blink(n).map_or(bg, |k| theme::mix(bg, to, k))
}
