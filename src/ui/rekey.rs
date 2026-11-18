//! The key float over the screen, and the keys that a click can change: in the status bar and in
//! the help (docs/zendo/panes/key-change.md, REKEY-1, REKEY-26).
use super::*;
use crate::app::rekey::{Choice, FloatButton, KeyTarget};
use crate::keys::{Key, Pane};
use ratatui::crossterm::event::KeyCode;
use std::time::Duration;

/// The time without a key after which the float says that the OS or the terminal can take it
/// (REKEY-15).
const NO_KEY: Duration = Duration::from_secs(5);

/// A target for a key of an action, but not for `esc` (REKEY-28).
fn target(pane: Pane, action: &'static str, key: Option<Key>, rect: Rect) -> Option<(Rect, KeyTarget)> {
    key.filter(|k| k.code != KeyCode::Esc).map(|key| (rect, KeyTarget { pane, action, key }))
}

/// The targets of a hint of the status bar at column `x`: the first key of its action, or one key
/// for each part of a group label such as `h/j/k/l`.
pub(super) fn hint_targets(app: &App, label: &str, acts: &[(Pane, &'static str)], x: u16, y: u16) -> Vec<(Rect, KeyTarget)> {
    let parts: Vec<&str> = if acts.len() > 1 { label.split('/').collect() } else { vec![label] };
    if parts.len() != acts.len() {
        return Vec::new();
    }
    let mut cx = x;
    parts
        .iter()
        .zip(acts)
        .filter_map(|(part, (p, a))| {
            let rect = Rect { x: cx, y, width: part.width() as u16, height: 1 };
            cx += part.width() as u16 + 1;
            target(*p, a, app.keys.keys_of(*p, a).first().copied(), rect)
        })
        .collect()
}

/// The keys of an action in a row of the help, from column `x`, as `alt-b ^⇧b`; the key that a
/// click made a button shows as one.
pub(super) fn help_keys(app: &App, t: &Theme, pane: Pane, action: &'static str, x: u16, y: u16, targets: &mut Vec<(Rect, KeyTarget)>) -> Vec<Span<'static>> {
    let mut cx = x;
    let mut spans = Vec::new();
    for k in app.keys.keys_of(pane, action) {
        let label = k.label();
        let rect = Rect { x: cx, y, width: label.width() as u16, height: 1 };
        targets.extend(target(pane, action, Some(k), rect));
        let on = app.rekey.button() == Some(KeyTarget { pane, action, key: k });
        spans.push(Span::styled(label.clone(), if on { Style::new().fg(t.bg).bg(t.acc2) } else { Style::new().fg(t.fg) }));
        spans.push(Span::raw(" "));
        cx += label.width() as u16 + 1;
    }
    spans
}

/// The key float, over everything: the action, its keys, the choices, the clash and the
/// suggestions (REKEY-2, REKEY-3, REKEY-11, REKEY-25, REKEY-26, REKEY-29).
pub(crate) fn draw_key_float(f: &mut Frame, app: &App, fr: &mut Cache, t: &Theme, area: Rect) {
    let Some(kf) = app.rekey.float() else { return };
    let tg = kf.target;
    let dim = Style::new().fg(t.dim);
    let help = crate::keys::ACTIONS.iter().find(|a| a.pane == tg.pane && a.name == tg.action).map_or("", |a| a.help);
    let mut keys = vec![Span::raw("  keys: ")];
    for k in app.keys.keys_of(tg.pane, tg.action) {
        let style = if k == tg.key { Style::new().fg(t.bg).bg(t.acc2) } else { Style::new().fg(t.fg) };
        keys.extend([Span::styled(k.label(), style), Span::raw("  ")]);
    }
    let ask = match kf.choice {
        Choice::Replace => format!("press the new key for {}", tg.key.label()),
        Choice::Add => "press the key to add".to_string(),
    };
    let (choice_line, choice_buttons) = buttons(t, "  ", choices(kf.choice));
    let mut lines = vec![
        Line::from(vec![Span::styled(format!("  {}", tg.action), Style::new().fg(t.acc).add_modifier(Modifier::BOLD)), Span::styled(format!(" · {} · {help}", tg.pane.name()), dim)]),
        Line::from(keys),
        Line::styled("  the hints and the help show the first key", dim),
        Line::raw(""),
        choice_line,
        Line::raw(""),
        Line::styled(format!("  {ask} · esc closes"), Style::new().fg(t.fg)),
    ];
    let mut rows = vec![(4, choice_buttons)];
    if let Some(m) = kf.message() {
        lines.push(Line::styled(format!("  {m}"), Style::new().fg(t.del)));
    } else if kf.opened.elapsed() > NO_KEY {
        lines.push(Line::styled("  no key came? the OS or the terminal can take a key first", dim));
    }
    if !kf.suggestions().is_empty() {
        let tries = kf.suggestions().iter().map(|k| (format!("[{}]", k.label()), FloatButton::Suggestion(*k), false)).collect();
        let (mut l, b) = buttons(t, "  try: ", tries);
        l.spans.push(Span::styled(if kf.from_model { "from the model" } else { "" }, dim));
        lines.push(Line::raw(""));
        rows.push((lines.len(), b));
        lines.push(l);
    }
    let w = 70.min(area.width);
    let h = (lines.len() as u16 + 2).min(area.height);
    let r = Rect { x: area.x + (area.width - w) / 2, y: area.y + (area.height - h) / 2, width: w, height: h };
    // Only the rows inside the float take a click.
    for (i, list) in rows.into_iter().filter(|(i, _)| (*i as u16) + 2 < h) {
        fr.float_buttons.extend(list.into_iter().map(|(x, width, b)| (Rect { x: r.x + 1 + x, y: r.y + 1 + i as u16, width, height: 1 }, b)));
    }
    f.render_widget(Clear, r);
    let block = Block::bordered().title(" change a key ").border_style(Style::new().fg(t.acc2)).style(Style::new().bg(t.panel));
    f.render_widget(Paragraph::new(lines).block(block), r);
}

fn choices(on: Choice) -> Vec<(String, FloatButton, bool)> {
    [("[replace]", Choice::Replace), ("[add]", Choice::Add)]
        .into_iter()
        .map(|(s, c)| (s.to_string(), FloatButton::Choice(c), c == on))
        .chain([("[remove]".to_string(), FloatButton::Remove, false)])
        .collect()
}

/// A line of buttons after `prefix`, and the column, the width and the button of each; an active
/// button stands out.
fn buttons(t: &Theme, prefix: &str, list: Vec<(String, FloatButton, bool)>) -> (Line<'static>, Vec<(u16, u16, FloatButton)>) {
    let mut x = prefix.width() as u16;
    let mut spans = vec![Span::raw(prefix.to_string())];
    let mut at = Vec::new();
    for (label, b, on) in list {
        at.push((x, label.width() as u16, b));
        x += label.width() as u16 + 2;
        spans.push(Span::styled(label, if on { Style::new().fg(t.bg).bg(t.acc) } else { Style::new().fg(t.fg) }));
        spans.push(Span::raw("  "));
    }
    (Line::from(spans), at)
}
