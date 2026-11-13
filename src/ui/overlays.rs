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

/// All keys of the pane where the help key was pressed, and the global keys, from the keymap, in
/// groups (PANES-30); under them the keymap file and the version (PANES-31).
pub(crate) fn draw_help(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    use crate::keys::Pane;
    let mut groups = help_section(app, t, app.help_pane);
    if app.help_pane != Pane::Global {
        let mut global = help_section(app, t, Pane::Global);
        global[0].insert(0, Line::raw(""));
        groups.append(&mut global);
    }
    let dim = Style::new().fg(t.dim);
    let keymap = match &app.keymap_file {
        Some(path) => format!("  keymap: {path}"),
        None => "  keymap: none · start one: zendo keymap > ~/.zendo/keymap.toml".to_string(),
    };
    let footer = vec![Line::raw(""), Line::styled(keymap, dim), Line::styled(format!("  {} · any key closes this", crate::setup::doctor::help_version()), dim)];
    let cols = columns(groups, area.height.saturating_sub(footer.len() as u16 + 1) as usize);
    let h = (cols.iter().map(Vec::len).max().unwrap_or(0) + footer.len() + 1) as u16;
    let (w, h) = ((72 * cols.len() as u16).min(area.width), h.min(area.height));
    let r = Rect { x: area.x + (area.width - w) / 2, y: area.y + (area.height - h) / 2, width: w, height: h };
    f.render_widget(Clear, r);
    f.render_widget(Block::new().style(Style::new().bg(t.panel).fg(t.fg)), r);
    let fh = (footer.len() as u16).min(r.height);
    let keys = Rect { y: r.y + 1, height: r.height.saturating_sub(fh + 1), ..r };
    let bottom = Rect { y: r.y + r.height - fh, height: fh, ..r };
    let n = cols.len() as u32;
    let areas = Layout::horizontal((0..n).map(|_| Constraint::Ratio(1, n))).split(keys);
    for (col, a) in cols.into_iter().zip(areas.iter()) {
        f.render_widget(Paragraph::new(col), *a);
    }
    f.render_widget(Paragraph::new(footer), bottom);
}

/// The keys of one pane, a list of lines for each group: the name of the pane, then each group
/// with its heading and its actions.
fn help_section(app: &App, t: &Theme, pane: crate::keys::Pane) -> Vec<Vec<Line<'static>>> {
    let mut groups = vec![vec![Line::styled(format!("  {}", pane.name().to_uppercase()), Style::new().fg(t.acc).add_modifier(Modifier::BOLD))]];
    let mut seen = std::collections::BTreeSet::new();
    let mut group = None;
    // Two actions with one help (the edges): one row.
    for a in crate::keys::ACTIONS.iter().filter(|a| a.pane == pane && seen.insert(a.help)) {
        if group != Some(a.group) {
            if group.is_some() {
                groups.push(Vec::new());
            }
            groups.last_mut().expect("a group").push(Line::styled(format!("  {}", a.group.name()), Style::new().fg(t.acc2)));
            group = Some(a.group);
        }
        let help = if app.agent_can(a.name) { a.help.to_string() } else { format!("{} (not with {})", a.help, app.agent_label()) };
        let row = Line::from(vec![Span::styled(format!("  {:<16}", app.keys.label(pane, a.name)), Style::new().fg(t.fg)), Span::styled(help, Style::new().fg(t.dim))]);
        groups.last_mut().expect("a group").push(row);
    }
    groups
}

/// The groups in columns of at most `height` lines; a group stays whole in one column.
fn columns(groups: Vec<Vec<Line<'static>>>, height: usize) -> Vec<Vec<Line<'static>>> {
    let mut cols: Vec<Vec<Line<'static>>> = vec![Vec::new()];
    for g in groups {
        if cols.last().is_some_and(|c| !c.is_empty() && c.len() + g.len() > height) {
            cols.push(g);
        } else {
            cols.last_mut().expect("a column").extend(g);
        }
    }
    cols
}
