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

pub(crate) fn draw_cli(f: &mut Frame, app: &App, t: &Theme, n: usize, area: Rect) {
    let [screen, bar] = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(area);
    f.render_widget(Block::new().style(Style::reset()), screen);
    if let Some(Some(pos)) = app.cli.with_screen(n, |s| paint(s, f.buffer_mut(), screen)) {
        f.set_cursor_position(pos);
    }
    draw_cli_bar(f, app, t, n, bar);
}

/// The name of session `n`: the F key of an agent session or of the `F5` terminal, `[>]` for
/// another terminal.
pub(crate) fn name(app: &App, n: usize) -> String {
    use crate::keys::Pane;
    match n.checked_sub(crate::app::SESSIONS) {
        None => app.keys.first(Pane::Global, &format!("session_{}", n + 1)).to_uppercase(),
        Some(0) => app.keys.first(Pane::Global, "terminal").to_uppercase(),
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
    use crate::keys::Pane;
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
    let leave = app.keys.first(Pane::Session, "leave");
    spans.push(Span::styled(format!("   {leave}"), Style::new().fg(t.fg)));
    spans.push(Span::styled(" back to zendo   wheel: earlier lines", Style::new().fg(t.dim)));
    f.render_widget(Paragraph::new(Line::from(spans)).style(Style::new().bg(t.bg)), area);
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
