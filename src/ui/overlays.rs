//! The theme picker and the keys help.
use super::*;

pub(crate) fn draw_themes(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    f.render_widget(Block::new().style(Style::new().bg(t.panel)), area);
    let inner = Rect { x: area.x + 2, y: area.y + 1, width: area.width.saturating_sub(4), height: area.height.saturating_sub(1) };
    let dim = Style::new().fg(t.dim);
    let mut lines = vec![Line::styled("THEME", dim), Line::raw("")];
    for (i, th) in THEMES.iter().enumerate() {
        let sel = i == app.theme;
        let mut spans = vec![
            Span::styled(if sel { "▸ " } else { "  " }, Style::new().fg(t.acc)),
            Span::styled(format!("{:<9}", th.name), Style::new().fg(t.fg)),
            Span::styled(format!("{:<14}", th.blurb), dim),
        ];
        for c in [th.bg, th.fg, th.acc, th.acc2] {
            spans.push(Span::styled("██", Style::new().fg(c)));
            spans.push(Span::raw(" "));
        }
        let mut l = Line::from(spans);
        if sel {
            l = l.style(Style::new().bg(t.sel));
        }
        lines.push(l);
        lines.push(Line::raw(""));
    }
    lines.push(Line::raw(""));
    for s in ["The note changes as you move.", "Your choice is saved in .zendo/."] {
        lines.push(Line::styled(s, dim));
    }
    f.render_widget(Paragraph::new(lines), inner);
}

/// All keys of the pane where `?` was pressed, and the global keys, from the keymap.
pub(crate) fn draw_help(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    use crate::keys::{ACTIONS, Pane};
    let pane = app.help_pane;
    let head = |p: Pane| Line::styled(format!("  {}", p.name().to_uppercase()), Style::new().fg(t.acc).add_modifier(Modifier::BOLD));
    let row = |p: Pane, name: &str, help: &str| {
        Line::from(vec![Span::styled(format!("  {:<16}", app.keys.label(p, name)), Style::new().fg(t.fg)), Span::styled(help.to_string(), Style::new().fg(t.dim))])
    };
    let mut items: Vec<Line> = vec![Line::raw(""), head(pane)];
    let mut seen = std::collections::BTreeSet::new();
    for a in ACTIONS.iter().filter(|a| a.pane == pane) {
        // Two actions with one help (the edges): one row.
        if seen.insert(a.help) {
            let help = if app.agent_can(a.name) { a.help.to_string() } else { format!("{} (not with {})", a.help, app.agent_label()) };
            items.push(row(pane, a.name, &help));
        }
    }
    items.push(Line::raw(""));
    items.push(head(Pane::Global));
    for a in ACTIONS.iter().filter(|a| a.pane == Pane::Global) {
        items.push(row(Pane::Global, a.name, a.help));
    }
    items.push(Line::raw(""));
    items.push(Line::styled("  Other keys: [keys.<pane>] <action> = \"key\" in ~/.zendo/config.toml", Style::new().fg(t.dim)));
    items.push(Line::styled(format!("  {} · any key closes this", crate::setup::doctor::version_line()), Style::new().fg(t.dim)));
    // Two columns when one does not fit.
    let (left, right): (Vec<Line>, Vec<Line>) = if items.len() + 2 > area.height as usize {
        let mut right = items.split_off(items.len().div_ceil(2));
        right.insert(0, Line::raw(""));
        (items, right)
    } else {
        (items, vec![])
    };
    let h = (left.len().max(right.len()) as u16 + 1).min(area.height);
    let w = if right.is_empty() { 64 } else { 120 }.min(area.width);
    let r = Rect { x: area.x + (area.width - w) / 2, y: area.y + (area.height - h) / 2, width: w, height: h };
    f.render_widget(Clear, r);
    f.render_widget(Block::new().style(Style::new().bg(t.panel).fg(t.fg)), r);
    if right.is_empty() {
        f.render_widget(Paragraph::new(left), r);
    } else {
        let [lc, rc] = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(r);
        f.render_widget(Paragraph::new(left), lc);
        f.render_widget(Paragraph::new(right), rc);
    }
}
