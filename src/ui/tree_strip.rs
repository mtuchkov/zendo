//! The strip of view letters on the left edge of the tree, and its hint after `v`
//! (files-view-column.md: FILES-68, FILES-70 to FILES-74).
use super::*;
use crate::docs::tree::View;

/// The width of the strip: one column of letters and one empty column.
pub(crate) const STRIP_W: u16 = 2;

/// The name of a view in the hint.
fn hint_name(v: View) -> &'static str {
    match v {
        View::WorktreeChanged => "worktree ∆",
        v => v.name(),
    }
}

/// The rows of the letters: the views of the files one under the other with an empty row between
/// them, a line, then `N`; without the empty rows where the pane is too low for them.
fn places(views: &[View], area: Rect) -> (Vec<(View, u16)>, Option<u16>) {
    let rows = views.len() as u16 + u16::from(views.contains(&View::Notes));
    let step = if rows * 2 - 1 <= area.height { 2 } else { 1 };
    let mut out = Vec::new();
    let mut y = area.y;
    let mut rule = None;
    for v in views {
        if *v == View::Notes {
            rule = Some(y);
            y += step;
        }
        out.push((*v, y));
        y += step;
    }
    let shown: Vec<(View, u16)> = out.into_iter().filter(|(_, y)| *y < area.bottom()).collect();
    // The line stands only over a letter that shows.
    let rule = rule.filter(|_| shown.iter().any(|(v, _)| *v == View::Notes));
    (shown, rule)
}

/// The strip, or the hint over the tree while it shows; both take clicks (FILES-73).
pub(crate) fn draw_strip(f: &mut Frame, app: &App, fr: &mut Cache, t: &Theme, area: Rect) {
    fr.view_buttons.clear();
    let tree = app.files.tree();
    // The first letter stands two rows under the head of the tree, at the first row of files.
    let area = Rect { y: area.y + 2, height: area.height.saturating_sub(2), ..area };
    let (rows, rule) = places(&tree.views(), area);
    let hint = app.files.view_hint();
    let names = rows.iter().map(|(v, _)| hint_name(*v).width() as u16 + 2).max().unwrap_or(1);
    let w = if hint { names.min(area.width) } else { 1 };
    if hint {
        // FILES-74: a box at 80% opacity, one column wider than the names: the tree under it shows
        // at 20%, mixed with the background of the tree.
        let bottom = rows.last().map_or(area.y, |(_, y)| y + 1);
        let under = Rect { width: w + 1, height: bottom - area.y, ..area }.intersection(f.area());
        veil(f.buffer_mut(), under, t);
    }
    if let Some(y) = rule {
        f.render_widget(Paragraph::new(Line::styled("─".repeat(w as usize), Style::new().fg(t.dim))), Rect { y, width: w, height: 1, ..area });
    }
    for (v, y) in rows {
        let on = v == tree.view;
        // FILES-82: a view that the session cannot show stands in the dim color and takes no click.
        let can = tree.can_show(v);
        let style = match (on, can) {
            (true, _) => Style::new().bg(t.acc).fg(t.bg).add_modifier(Modifier::BOLD),
            (false, true) => Style::new().fg(t.acc2),
            (false, false) => Style::new().fg(t.dim),
        };
        let text = if hint { format!("{} {}", v.letter(), hint_name(v)) } else { v.letter().to_string() };
        let r = Rect { y, width: (text.width() as u16).min(w.max(1)), height: 1, ..area };
        f.render_widget(Paragraph::new(Line::styled(text, style)), r);
        if can {
            fr.view_buttons.push((r, v));
        }
    }
}

/// The cells under the hint at 20%: each color four fifths of the way to the background of the tree.
fn veil(buf: &mut ratatui::buffer::Buffer, area: Rect, t: &Theme) {
    let mix = |c: Color| match (c, t.panel) {
        (Color::Rgb(r, g, b), Color::Rgb(pr, pg, pb)) => {
            let m = |v: u8, p: u8| ((v as u16 + 4 * p as u16) / 5) as u8;
            Color::Rgb(m(r, pr), m(g, pg), m(b, pb))
        }
        _ => t.dim,
    };
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            let cell = &mut buf[(x, y)];
            let fg = mix(cell.fg);
            cell.set_fg(fg).set_bg(t.panel).set_style(Style::new().remove_modifier(Modifier::BOLD | Modifier::REVERSED));
        }
    }
}
