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
/// groups (PANES-30); under them the keymap file and the version (PANES-31). Returns its area.
pub(crate) fn draw_help(f: &mut Frame, app: &App, fr: &mut Cache, t: &Theme, area: Rect) -> Rect {
    use crate::keys::Pane;
    let mut groups = help_section(t, app.key_help.pane());
    // Where the user writes, the keys while text is selected (PANES-39).
    if matches!(app.key_help.pane(), Pane::Write | Pane::Chat) {
        let mut sel = help_section(t, Pane::Selected);
        sel[0].insert(0, Row::Text(Line::raw("")));
        groups.append(&mut sel);
    }
    if app.key_help.pane() != Pane::Global {
        let mut global = help_section(t, Pane::Global);
        global[0].insert(0, Row::Text(Line::raw("")));
        groups.append(&mut global);
    }
    let dim = Style::new().fg(t.dim);
    let keymap = match app.key_help.file() {
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
        let lines: Vec<Line> = col.into_iter().enumerate().map(|(i, row)| row.line(app, t, a.x, a.y + i as u16, &mut fr.key_targets)).collect();
        f.render_widget(Paragraph::new(lines), *a);
    }
    f.render_widget(Paragraph::new(footer), bottom);
    r
}

/// A row of the help: a heading, or the keys and the help of an action.
enum Row {
    Text(Line<'static>),
    Action(crate::keys::Pane, &'static str),
}

impl Row {
    /// The line at column `x`, row `y`; each key of an action is a target of a click (REKEY-1).
    fn line(self, app: &App, t: &Theme, x: u16, y: u16, targets: &mut Vec<(Rect, crate::app::rekey::KeyTarget)>) -> Line<'static> {
        let (pane, name) = match self {
            Row::Text(l) => return l,
            Row::Action(p, n) => (p, n),
        };
        let help = crate::keys::ACTIONS.iter().find(|a| a.pane == pane && a.name == name).map_or("", |a| a.help);
        let help = if app.agent_can(name) { help.to_string() } else { format!("{help} (not with {})", app.agent_label()) };
        let mut spans = vec![Span::raw("  ")];
        spans.extend(super::rekey::help_keys(app, t, pane, name, x + 2, y, targets));
        let width: usize = spans.iter().map(|s| s.content.width()).sum();
        spans.push(Span::raw(" ".repeat(18usize.saturating_sub(width))));
        spans.push(Span::styled(help, Style::new().fg(t.dim)));
        Line::from(spans)
    }
}

/// The keys of one pane, a list of rows for each group: the name of the pane, then each group
/// with its heading and its actions.
fn help_section(t: &Theme, pane: crate::keys::Pane) -> Vec<Vec<Row>> {
    let mut groups = vec![vec![Row::Text(Line::styled(format!("  {}", pane.name().to_uppercase()), Style::new().fg(t.acc).add_modifier(Modifier::BOLD)))]];
    let mut seen = std::collections::BTreeSet::new();
    let mut group = None;
    // Two actions with one help (the edges): one row.
    for a in crate::keys::ACTIONS.iter().filter(|a| a.pane == pane && seen.insert(a.help)) {
        if group != Some(a.group) {
            if group.is_some() {
                groups.push(Vec::new());
            }
            groups.last_mut().expect("a group").push(Row::Text(Line::styled(format!("  {}", a.group.name()), Style::new().fg(t.acc2))));
            group = Some(a.group);
        }
        groups.last_mut().expect("a group").push(Row::Action(pane, a.name));
    }
    groups
}

/// The groups in columns of at most `height` lines; a group stays whole in one column.
fn columns<T>(groups: Vec<Vec<T>>, height: usize) -> Vec<Vec<T>> {
    let mut cols: Vec<Vec<T>> = vec![Vec::new()];
    for g in groups {
        if cols.last().is_some_and(|c| !c.is_empty() && c.len() + g.len() > height) {
            cols.push(g);
        } else {
            cols.last_mut().expect("a column").extend(g);
        }
    }
    cols
}

/// FILES-37: the popup before a file action in a worktree while the agent of the session answers.
pub(crate) fn draw_confirm(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    let Some(text) = app.confirm_text() else { return };
    let w = 60.min(area.width);
    let lines = markdown::wrap(vec![Span::styled(text, Style::new().fg(t.fg))], w.saturating_sub(4) as usize, "  ", "  ");
    let h = (lines.len() as u16 + 2).min(area.height);
    let r = Rect { x: area.x + (area.width - w) / 2, y: area.y + (area.height - h) / 2, width: w, height: h };
    f.render_widget(Clear, r);
    let block = Block::bordered().title(" the agent works ").border_style(Style::new().fg(t.acc2)).style(Style::new().bg(t.panel));
    f.render_widget(Paragraph::new(lines).block(block), r);
}

/// The question of the first start, over the screen while the music plays: keep the lofi music on
/// at each start, or turn it off (LOFI-12). Its two answers take a click.
pub(crate) fn draw_music_question(f: &mut Frame, app: &App, fr: &mut Cache, t: &Theme, area: Rect) {
    fr.music_answers = None;
    if !app.music.asking {
        return;
    }
    let w = 58.min(area.width);
    let text = "zendo plays calm lofi music that it makes itself. Keep it on at each start? The [music] button and alt-m stop and start it any time.";
    let mut lines = markdown::wrap(vec![Span::styled(text, Style::new().fg(t.fg))], w.saturating_sub(4) as usize, "  ", "  ");
    let (keep, off) = ("[ keep on · y ]", "[ turn off · n ]");
    lines.push(Line::raw(""));
    lines.push(Line::from(vec![Span::raw("  "), Span::styled(keep, Style::new().fg(t.ext.doc)), Span::raw("   "), Span::styled(off, Style::new().fg(t.dim))]));
    let rows = lines.len() as u16;
    let h = (rows + 2).min(area.height);
    let r = Rect { x: area.x + (area.width - w) / 2, y: area.y + (area.height - h) / 2, width: w, height: h };
    f.render_widget(Clear, r);
    let block = Block::bordered().title(" lofi music ").border_style(Style::new().fg(t.ext.doc)).style(Style::new().bg(t.panel));
    f.render_widget(Paragraph::new(lines).block(block), r);
    // On a screen too small for the whole float, the answers take keys only: no rect points at a
    // line that is not drawn.
    let fits = h == rows + 2 && w as usize >= keep.width() + off.width() + 7;
    let (y, x) = (r.bottom().saturating_sub(2), r.x + 3);
    let button = |x: u16, s: &str| Rect { x, y, width: s.width() as u16, height: 1 };
    fr.music_answers = fits.then(|| (button(x, keep), button(x + keep.width() as u16 + 3, off)));
}
