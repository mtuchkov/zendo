//! The layout: which panes fit, their widths, the top bar.
use super::*;

/// The width of a pane: the one you dragged, else a default for the layout.
pub fn pane_width(app: &App, p: Focus, total: u16) -> u16 {
    let key = App::pane_key(p).unwrap_or("note");
    if let Some(w) = app.state.widths.get(key) {
        return *w;
    }
    match p {
        Focus::Tree => (total / 4).clamp(28, 44),
        Focus::Claude if !app.state.wide() => (total * 2 / 5).clamp(40, 60),
        Focus::Claude => (total / 4).clamp(40, 60),
        Focus::Delta => (total / 3).clamp(56, 100),
        Focus::Note => 50,
    }
}

/// Which panes to show, left to right.
/// Standard: one main pane (delta replaces the note while reading) and the chat.
/// Wide screen: every open pane side by side.
pub fn panes(app: &App, width: u16) -> Vec<Focus> {
    let chat = app.panel;
    let delta = app.delta_open && app.current.is_some() && app.pinned.is_none();
    let mut p = Vec::new();
    if app.state.wide() {
        // Wide: note | chat | delta.
        p.push(Focus::Note);
        if chat {
            p.push(Focus::Claude);
        }
        if delta {
            p.push(Focus::Delta);
        }
    } else if app.mode == Mode::Write {
        p.push(Focus::Note);
        if chat {
            p.push(Focus::Claude);
        } else if delta {
            p.push(Focus::Delta);
        }
    } else {
        p.push(if delta { Focus::Delta } else { Focus::Note });
        if chat {
            p.push(Focus::Claude);
        }
    }
    // The file tree sits on the left in both layouts.
    if app.state.tree {
        p.insert(0, Focus::Tree);
    }
    let need = |f: &Focus| if *f == Focus::Note { 50 } else { pane_width(app, *f, width) };
    hide_to_fit(p, app.focus, |ps| ps.iter().map(need).sum::<u16>() <= width)
}

/// Too narrow for all panes: hide from the edges, by turns, starting on the right:
/// the rightmost pane right of the main pane, then the leftmost pane left of it, then the
/// rightmost again. The pane in focus stays. The main pane goes last.
pub fn hide_to_fit(mut p: Vec<Focus>, focus: Focus, fits: impl Fn(&[Focus]) -> bool) -> Vec<Focus> {
    let mut right = true;
    while p.len() > 1 && !fits(&p) {
        // The main pane: the note, or the pane in its place (delta in standard).
        let main = p.iter().position(|f| *f == Focus::Note).or_else(|| p.iter().position(|f| *f != Focus::Tree)).unwrap_or(0);
        let on_right = (main + 1..p.len()).rev().find(|i| p[*i] != focus);
        let on_left = (0..main).find(|i| p[*i] != focus);
        let pick = if right { on_right.or(on_left) } else { on_left.or(on_right) };
        let i = match pick {
            Some(i) => i,
            // Only the main pane and the pane in focus are left.
            None => match p.iter().position(|f| *f != focus) {
                Some(i) => i,
                None => break,
            },
        };
        p.remove(i);
        right = !right;
    }
    p
}

pub(crate) fn draw_left(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    if app.mode == Mode::Write {
        draw_write(f, app, t, area);
    } else {
        draw_read(f, app, t, area);
    }
}

pub(crate) fn column(area: Rect, width: u16) -> Rect {
    let w = width.min(area.width.saturating_sub(4)).max(10);
    Rect { x: area.x + (area.width.saturating_sub(w)) / 2, width: w, ..area }
}

pub(crate) fn draw_top(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let left = match app.mode {
        Mode::Write => match &app.editor_path {
            Some(p) => app.rel(p),
            None => {
                let title = crate::docs::notes::title_of(&app.editor.text()).unwrap_or_default();
                let p = app.store.path_for(if title.is_empty() { "note" } else { &title }, Local::now().date_naive());
                format!("new note → {}", app.rel(&p))
            }
        },
        _ => match app.pinned.as_deref() {
            Some(ALL_KEY) => format!("{} · all notes", app.rel(&app.cfg.notes_dir)),
            _ => app.note().map(|n| app.rel(&n.path)).unwrap_or_else(|| app.rel(&app.cfg.notes_dir)),
        },
    };
    let right = app.branch.clone().unwrap_or_default();
    let pad = (area.width as usize).saturating_sub(left.width() + right.width() + 4);
    let line = Line::from(vec![Span::raw("  "), Span::raw(left.clone()), Span::raw(" ".repeat(pad)), Span::raw(right.clone()), Span::raw("  ")]);
    f.render_widget(Paragraph::new(line).style(Style::new().fg(t.dim)), area);

    // No buttons: every action has a key. A newer zendo shows as a quiet hint with its key.
    if let Some(v) = app.update.as_ref().filter(|_| matches!(app.mode, Mode::Read | Mode::Write)) {
        let hint = format!("zendo {v} is out · {} to upgrade", app.keys.first(crate::keys::Pane::Read, "upgrade"));
        let w = hint.width() as u16;
        let x = area.right().saturating_sub(right.width() as u16 + 5 + w);
        if x > area.x + left.width() as u16 + 4 {
            f.render_widget(Paragraph::new(Line::styled(hint, Style::new().fg(t.acc))), Rect { x, width: w, ..area });
        }
    }
}

pub(crate) fn date_label(n: &Note) -> String {
    let d = n.created.unwrap_or_else(|| n.modified.date_naive());
    let time = if n.created.is_none_or(|c| c == n.modified.date_naive()) { n.modified.format(" · %H:%M").to_string() } else { String::new() };
    format!("{}{}", d.format("%a %-d %b"), time)
}
