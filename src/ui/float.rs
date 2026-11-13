//! The new note in a float: the screen behind in dim colors, without key hints, and the float over
//! the note pane (FLOAT-2, FLOAT-6 to FLOAT-9).
use super::*;
use ratatui::layout::Margin;

/// The margin of the float on each side of the text column (FLOAT-8).
const MARGIN: u16 = 4;

pub(crate) fn draw_float(f: &mut Frame, app: &mut App) {
    let Some(mut fl) = app.float.take() else { return };
    // The screen behind, as it was: its own state, without the mouse selection of the float.
    let (sel, copy) = (app.sel.take(), std::mem::take(&mut app.copy_pending));
    fl.swap(app);
    draw(f, app);
    fl.swap(app);
    (app.sel, app.copy_pending) = (sel, copy);
    let t = *app.theme();
    let area = f.area();
    dim(f, &t, area);
    let r = float_rect(app, area);
    fl.rect = r;
    app.float = Some(fl);
    app.regions.clear();
    app.pane_rects.clear();
    f.render_widget(Clear, r);
    f.render_widget(Block::bordered().border_style(Style::new().fg(t.line)).style(Style::new().bg(t.bg).fg(t.fg)), r);
    let inner = r.inner(Margin::new(1, 1));
    if inner.height < 3 {
        return;
    }
    draw_float_top(f, app, &t, Rect { height: 1, ..inner });
    f.render_widget(Paragraph::new("─".repeat(inner.width as usize)).style(Style::new().fg(t.line)), Rect { y: inner.y + 1, height: 1, ..inner });
    draw_write(f, app, &t, Rect { y: inner.y + 2, height: inner.height - 2, ..inner });
    if app.mode == Mode::Help {
        draw_help(f, app, &t, area);
    }
    draw_selection(f, app, &t);
}

/// The screen behind the float: one dim color, and no key hints in the status bar (FLOAT-2).
fn dim(f: &mut Frame, t: &Theme, area: Rect) {
    let buf = f.buffer_mut();
    for (x, y) in (area.top()..area.bottom()).flat_map(|y| (area.left()..area.right()).map(move |x| (x, y))) {
        let c = &mut buf[(x, y)];
        c.set_fg(t.line).set_bg(t.bg);
        if y + 1 == area.bottom() {
            c.set_symbol(" ");
        }
    }
}

/// The float: the text column and its margins wide, the screen less one row at the top and at the
/// bottom high, over the note pane; at the center of the screen when the note pane is narrower
/// (FLOAT-8, FLOAT-9).
fn float_rect(app: &App, area: Rect) -> Rect {
    let w = (app.column_width() + 2 * MARGIN + 2).min(area.width.saturating_sub(2));
    let over = app.pane_rects.iter().find(|(p, r)| *p == Focus::Note && r.width >= w).map_or(area, |(_, r)| *r);
    Rect { x: over.x + (over.width - w) / 2, y: area.y + 1, width: w, height: area.height.saturating_sub(2) }
}

/// The top line of the float: the path of the note, or the notes folder and `new note` before the
/// first save; the branch and the key that closes the float (FLOAT-6).
fn draw_float_top(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    let path = match &app.editor_path {
        Some(p) => app.rel(p),
        None => format!("{}/new note", app.rel(&app.cfg.notes_dir)),
    };
    let esc = app.keys.first(crate::keys::Pane::Write, "done");
    let mut right = vec![Span::styled("[ ", Style::new().fg(t.dim)), Span::styled(esc, Style::new().fg(t.bg).bg(t.fg)), Span::styled(" close ]", Style::new().fg(t.dim))];
    if let Some(b) = app.branch.as_ref().filter(|b| !b.is_empty()) {
        right.insert(0, Span::styled(format!("{b}  "), Style::new().fg(t.dim)));
    }
    let rw: u16 = right.iter().map(|s| s.content.width() as u16).sum();
    f.render_widget(Paragraph::new(Line::styled(format!("  {path}"), Style::new().fg(t.dim))), area);
    let x = area.right().saturating_sub(rw + 2).max(area.x);
    f.render_widget(Paragraph::new(Line::from(right)), Rect { x, width: area.right() - x, ..area });
}

/// The date and the time when the user opened the float, under the title (FLOAT-7).
pub(crate) fn float_date(app: &App, t: &Theme) -> Vec<Line<'static>> {
    let Some(fl) = &app.float else { return Vec::new() };
    vec![Line::styled(fl.opened.format("%a %-d %b · %H:%M").to_string(), Style::new().fg(t.dim)), Line::raw("")]
}
