//! The strip of view letters on the left edge of the tree, and its hint after `v`
//! (files-view-column.md: FILES-68, FILES-70 to FILES-74).
use super::*;
use crate::docs::tree::View;

/// The width of the strip: one column of letters and one empty column.
pub(crate) const STRIP_W: u16 = 2;

/// The name of a view in the hint.
fn hint_name(v: View) -> &'static str {
    match v {
        View::WorktreeChanged => "worktree Δ",
        v => v.name(),
    }
}

/// The rows of the letters: the views of the files one under the other with an empty row between
/// them, a line, then `N`.
fn places(views: &[View], area: Rect) -> (Vec<(View, u16)>, Option<u16>) {
    let mut out = Vec::new();
    let mut y = area.y;
    let mut rule = None;
    for v in views {
        if *v == View::Notes {
            rule = Some(y);
            y += 2;
        }
        out.push((*v, y));
        y += 2;
    }
    (out.into_iter().filter(|(_, y)| *y < area.bottom()).collect(), rule.filter(|y| *y < area.bottom()))
}

/// The strip, or the hint over the tree while it shows; both take clicks (FILES-73).
pub(crate) fn draw_strip(f: &mut Frame, app: &App, fr: &mut Cache, t: &Theme, area: Rect) {
    fr.view_buttons.clear();
    let tree = app.files.tree();
    let (rows, rule) = places(&tree.views(), area);
    let hint = app.files.view_hint();
    let w = if hint { 14.min(area.width) } else { 1 };
    if hint {
        let bottom = rows.last().map_or(area.y, |(_, y)| y + 1);
        f.render_widget(Block::new().style(Style::new().bg(t.float)), Rect { width: w + 1, height: bottom - area.y, ..area });
    }
    if let Some(y) = rule {
        f.render_widget(Paragraph::new(Line::styled("─".repeat(w as usize), Style::new().fg(t.dim))), Rect { y, width: w, height: 1, ..area });
    }
    for (v, y) in rows {
        let on = v == tree.view;
        let style = if on { Style::new().bg(t.acc).fg(t.bg).add_modifier(Modifier::BOLD) } else { Style::new().fg(t.acc2) };
        let text = if hint { format!("{} {}", v.letter(), hint_name(v)) } else { v.letter().to_string() };
        let r = Rect { y, width: (text.width() as u16).min(w.max(1)), height: 1, ..area };
        f.render_widget(Paragraph::new(Line::styled(text, style)), r);
        fr.view_buttons.push((r, v));
    }
}
