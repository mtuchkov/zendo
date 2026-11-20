//! The new note in a float: the screen behind in dim colors, without key hints, and the float over
//! the note pane (FLOAT-2, FLOAT-6 to FLOAT-9).
use super::*;
use ratatui::layout::Margin;

/// The margin of the float on each side of the text column (FLOAT-8).
const MARGIN: u16 = 4;

pub(crate) fn draw_float(f: &mut Frame, app: &mut App, fr: &mut Cache) {
    let Some(mut fl) = app.writer.take_float() else { return };
    // The screen behind, as it was: its own state, without the mouse selection of the float.
    let aside = app.pointer.set_aside();
    fl.swap(app);
    fl.swap_rows(fr);
    draw_into(f, app, fr);
    fl.swap(app);
    fl.swap_rows(fr);
    app.pointer.bring_back(aside);
    let t = *app.theme();
    let area = f.area();
    dim(f, &t, area);
    let r = float_rect(app, fr, area);
    fl.rect = r;
    app.writer.set_float(fl);
    fr.regions.clear();
    fr.pane_rects.clear();
    fr.spec_button = None;
    fr.upgrade_button = None;
    f.render_widget(Clear, r);
    // The float has a background of its own (FLOAT-16).
    let t = t.on_float();
    f.render_widget(Block::bordered().border_style(Style::new().fg(t.line)).style(Style::new().bg(t.bg).fg(t.fg)), r);
    let inner = r.inner(Margin::new(1, 1));
    if inner.height < 3 {
        return;
    }
    draw_float_top(f, app, &t, Rect { height: 1, ..inner });
    f.render_widget(Paragraph::new("─".repeat(inner.width as usize)).style(Style::new().fg(t.line)), Rect { y: inner.y + 1, height: 1, ..inner });
    draw_write(f, app, fr, &t, Rect { y: inner.y + 2, height: inner.height - 2, ..inner });
    if app.mode == Mode::Help {
        fr.overlay = Some(draw_help(f, app, fr, &t, area));
    }
    draw_selection(f, app, &t);
    overlays::draw_external_question(f, app, fr, &t, area);
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
fn float_rect(app: &App, fr: &Cache, area: Rect) -> Rect {
    let w = (app.column_width() + 2 * MARGIN + 2).min(area.width.saturating_sub(2));
    let over = fr.pane_rects.iter().find(|(p, r)| *p == Focus::Note && r.width >= w).map_or(area, |(_, r)| *r);
    Rect { x: over.x + (over.width - w) / 2, y: area.y + 1, width: w, height: area.height.saturating_sub(2) }
}

/// The top line of the float: the path of the note, or the notes folder and `new note` before the
/// first save; the branch of the note (FILES-62) and the key that closes the float (FLOAT-6).
fn draw_float_top(f: &mut Frame, app: &App, t: &Theme, area: Rect) {
    let path = match app.writer.path() {
        Some(p) => app.rel(p),
        None => format!("{}/new note", app.rel(&app.cfg.notes_dir)),
    };
    let esc = app.keys.first(crate::keys::Pane::Write, "done");
    let mut right = vec![Span::styled("[ ", Style::new().fg(t.dim)), Span::styled(esc, Style::new().fg(t.bg).bg(t.fg)), Span::styled(" close ]", Style::new().fg(t.dim))];
    if let Some(b) = app.doc_branch() {
        right.insert(0, Span::styled(format!("{b}  "), Style::new().fg(t.dim)));
    }
    let rw: u16 = right.iter().map(|s| s.content.width() as u16).sum();
    let x = area.right().saturating_sub(rw + 2).max(area.x);
    // Two spaces before the path and between the path and the branch (FLOAT-24).
    let room = usize::from(x - area.x).saturating_sub(4);
    f.render_widget(Paragraph::new(Line::styled(format!("  {}", fit_head(&path, room)), Style::new().fg(t.dim))), Rect { width: x - area.x, ..area });
    f.render_widget(Paragraph::new(Line::from(right)), Rect { x, width: area.right() - x, ..area });
}

/// The text in `width` columns: when it is longer, its first chars go, and `…` stands in their
/// place (FLOAT-24).
pub(crate) fn fit_head(text: &str, width: usize) -> String {
    if text.width() <= width {
        return text.to_string();
    }
    let mut w = 1;
    let tail: Vec<char> = text.chars().rev().take_while(|c| { w += crate::text::editor::char_width(*c); w <= width }).collect();
    std::iter::once('…').chain(tail.into_iter().rev()).filter(|_| width > 0).collect()
}

/// The date and the time when the user opened the float, under the title (FLOAT-7).
pub(crate) fn float_date(app: &App, t: &Theme) -> Vec<Line<'static>> {
    let Some(fl) = app.writer.float() else { return Vec::new() };
    vec![Line::styled(float_date_label(fl.opened), Style::new().fg(t.dim)), Line::raw("")]
}

/// The text of the line of the date of the float (FLOAT-7), where `[ → spec ]` stands (SPEC-6).
pub(crate) fn float_date_label(opened: chrono::DateTime<chrono::Local>) -> String {
    opened.format("%a %-d %b · %H:%M").to_string()
}
