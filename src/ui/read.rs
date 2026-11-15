//! Read mode: the note formatted for the pane, with its recap and the cursor line.
use super::*;

/// The lines of the read view for a note, and the row of each source line.
/// The title of a note, bold, wrapped to the width.
pub(crate) fn title_rows(title: &str, width: usize, t: &Theme) -> Vec<Line<'static>> {
    markdown::wrap(vec![Span::styled(title.to_string(), Style::new().fg(t.fg).add_modifier(Modifier::BOLD))], width, "", "")
}

/// The head of a note in both modes: the title rows, the date and the tags, and the recap.
pub(crate) fn note_head(app: &App, note: &Note, width: usize, t: &Theme, title: Vec<Line<'static>>) -> Vec<Line<'static>> {
    let mut lines = title;
    let mut meta = vec![Span::styled(date_label(note), Style::new().fg(t.dim))];
    for tag in note.tags() {
        meta.push(Span::styled(" ", Style::new()));
        meta.push(Span::styled(format!("#{tag}"), Style::new().fg(t.acc2)));
    }
    let meta: Vec<Span> = if note.tags().is_empty() { meta } else {
        let mut m = vec![meta.remove(0), Span::styled(" ·", Style::new().fg(t.dim))];
        m.extend(meta);
        m
    };
    lines.extend(markdown::wrap(meta, width, "", ""));
    lines.push(Line::raw(""));
    lines.extend(recap_lines(app, note, width, t));
    lines.extend(review_lines(app, note, width, t));
    lines
}

pub(crate) fn read_lines(app: &App, note: &Note, width: usize, t: &Theme) -> (Vec<Line<'static>>, Vec<usize>, Vec<usize>) {
    let lines = note_head(app, note, width, t, title_rows(&note.title, width, t));
    let head = lines.len();

    if code::is_code(&note.path) {
        let body: Vec<String> = note.body.lines().map(str::to_string).collect();
        return with_head(lines, head, code::render(&body, &code::lang_of(&note.path), width, t));
    }
    with_head(lines, head, markdown::render(&markdown::plain(&note.body), width, t, true))
}

/// The head lines, then the text; the rows of the source lines and the words count the head too.
fn with_head(mut lines: Vec<Line<'static>>, head: usize, r: markdown::Rendered) -> (Vec<Line<'static>>, Vec<usize>, Vec<usize>) {
    let rows = r.rows.iter().map(|x| x + head).collect();
    let mut words = vec![0; head];
    words.extend(r.words);
    lines.extend(r.lines);
    (lines, rows, words)
}

/// The file shows as code with line numbers: it is not markdown.
pub(crate) fn shows_code(path: &std::path::Path) -> bool {
    code::is_code(path)
}

/// The recap block under the title, when there is one.
pub(crate) fn recap_lines(app: &App, note: &Note, width: usize, t: &Theme) -> Vec<Line<'static>> {
    if !app.state.show_recap {
        return vec![];
    }
    let rel = app.rel(&note.path);
    let dim = Style::new().fg(t.dim);
    let mut out = Vec::new();
    let writing = app.recap_job.as_ref().is_some_and(|j| j.rel == rel);
    let recap = app.state.files.get(&rel).and_then(|f| f.recap.as_ref());
    let bar = Span::styled("▎ ", Style::new().fg(t.acc));
    match recap {
        Some(r) => {
            let mut head = format!("RECAP · {} · {}", r.at.format("%a %-d %b %H:%M"), if r.auto { "while idle" } else { "on request" });
            if r.of_hash != crate::setup::state::hash(&note.body) {
                head.push_str(" · the note changed since");
            }
            if writing {
                head.push_str(" · writing a new one…");
            }
            out.push(Line::from(vec![bar.clone(), Span::styled(head, dim.add_modifier(Modifier::BOLD))]));
            let body = markdown::render(&markdown::plain(&r.text), width.saturating_sub(2), t, false);
            for l in body.lines {
                let mut spans = vec![bar.clone()];
                spans.extend(l.spans);
                out.push(Line::from(spans));
            }
            out.push(Line::raw(""));
        }
        None if writing => {
            out.push(Line::from(vec![bar, Span::styled("RECAP · writing…", dim.add_modifier(Modifier::BOLD))]));
            out.push(Line::raw(""));
        }
        None => {}
    }
    out
}

/// Row of each source line in the read view, for scrolling to a line.
pub fn source_rows(app: &App, width: usize) -> Vec<usize> {
    match app.note() {
        Some(n) => read_lines(app, n, width, app.theme()).1,
        None => vec![],
    }
}

pub(crate) fn draw_read(f: &mut Frame, app: &mut App, fr: &mut Cache, t: &Theme, area: Rect) {
    let note = app.note().cloned();
    let code = note.as_ref().is_some_and(|n| shows_code(&n.path));
    let (gut, col) = match &note {
        Some(n) if code => code_column(area, app.code_width(), n.body.lines().count()),
        _ => (Rect::default(), column(area, app.column_width())),
    };
    fr.text_width = col.width as usize;
    let Some(note) = note else { return draw_no_version(f, app, t, col) };
    fr.regions.push((crate::text::select::Region::Note, col));
    let (lines, rows, words) = read_lines(app, &note, col.width as usize, t);
    fr.word_limit = app.word_limit_for(&note.path);
    fr.word_count = words.iter().sum();
    fr.over_limit = over_limit(&words, fr.word_limit);
    fr.read_text = lines.iter().map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect()).collect();
    fr.read_rows = rows;
    let h = col.height as usize;
    // The cursor stays in the text, and the view follows it, with a margin.
    app.reader.fit(&fr.read_text, h);
    if let Some(r) = fr.over_limit {
        let y0 = col.y as i64 + r as i64 - app.reader.scroll() as i64;
        draw_over_limit(f, app, fr, t, area, col, y0.max(col.y as i64) as u16);
    }
    // The cursor and the selection show only where the keys go.
    let active = app.mode == Mode::Read && app.focus == Focus::Note;
    let marks = confusing_rows(app, &app.rel(&note.path), &fr.read_rows, lines.len());
    let w = col.width as usize;
    // The frame of this draw, not the stored one: the app has none while `ui` draws.
    let table = if active { app.reader.table(&fr.read_text) } else { None };
    let view: Vec<Line> = lines
        .into_iter()
        .enumerate()
        .skip(app.reader.scroll())
        .take(h)
        .map(|(i, l)| if marks.contains(&i) { tint(l, confusing(t), w) } else { l })
        .zip(app.reader.scroll()..)
        .map(|(l, i)| if active { mark_row(app, fr, t, i, l, w, table.as_ref()) } else { l })
        .collect();
    f.render_widget(Paragraph::new(view), col);
    if code {
        draw_gutter(f, t, gut, &fr.read_rows, app.reader.scroll(), active.then_some(app.reader.row()));
    }
    if let Some(row) = say_row_read(app, fr, &note) {
        say_arrow(f, app, t, area, col, row);
    }
    // The block cursor on the current char; the terminal makes it blink.
    if active && app.reader.row() >= app.reader.scroll() && app.reader.row() < app.reader.scroll() + h {
        let row = fr.read_text.get(app.reader.row()).cloned().unwrap_or_default();
        let x: usize = row.chars().take(app.reader.col()).map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0)).sum();
        f.set_cursor_position((col.x + (x as u16).min(col.width.saturating_sub(1)), col.y + (app.reader.row() - app.reader.scroll()) as u16));
    }
}

/// The first hint without notes: the key of a new note, and where notes go (PANES-28).
fn no_notes_new(app: &App) -> String {
    let dir = app.rel(&app.cfg.notes_dir);
    app.keys.key(crate::keys::Pane::Read, "new").map_or(format!("A new note goes to {dir}."), |k| format!("Press {k} to write the first one. It goes to {dir}."))
}

/// The second hint without notes: the keys of the chat and of the help (PANES-28, PANES-29).
fn no_notes_keys(app: &App) -> String {
    use crate::keys::Pane;
    let chat = app.keys.key(Pane::Read, "chat").map(|k| format!("{k} to talk to Claude"));
    let help = app.keys.key(Pane::Global, "help").map(|k| format!("{k} for keys"));
    let parts: Vec<String> = chat.into_iter().chain(help).collect();
    if parts.is_empty() { String::new() } else { format!("Press {}.", parts.join(", ")) }
}

/// The read pane without a note: the open document has no version here (FILES-40, FILES-48), or
/// there are no notes.
fn draw_no_version(f: &mut Frame, app: &App, t: &Theme, col: Rect) {
    let msg = match app.peek.on {
        Some(o) => format!("This document does not exist in {}.", o.name()),
        None if app.doc_missing() => "This document does not exist in this session.".to_string(),
        None => return draw_no_notes(f, app, t, col),
    };
    let y = col.y + col.height / 3;
    f.render_widget(Paragraph::new(Line::styled(msg, Style::new().fg(t.fg).add_modifier(Modifier::BOLD))), Rect { y, height: 1, ..col });
}

/// The read pane when there is no note.
fn draw_no_notes(f: &mut Frame, app: &App, t: &Theme, col: Rect) {
    let msg = vec![
        Line::styled("No notes yet.", Style::new().fg(t.fg).add_modifier(Modifier::BOLD)),
        Line::raw(""),
        Line::styled(no_notes_new(app), Style::new().fg(t.dim)),
        Line::styled(no_notes_keys(app), Style::new().fg(t.dim)),
    ];
    let y = col.y + col.height / 3;
    f.render_widget(Paragraph::new(msg), Rect { y, height: col.height.saturating_sub(y - col.y), ..col });
}

/// The cursor line and the selection on one row; `table` is the selection in a table (PANES-34).
fn mark_row(app: &App, fr: &Cache, t: &Theme, i: usize, l: Line<'static>, width: usize, table: Option<&crate::text::boxtable::Range>) -> Line<'static> {
    let l = if i == app.reader.row() { tint(l, cursor_line(t), width) } else { l };
    if let Some(r) = table {
        return match r.chars(&fr.read_text, i) {
            Some((a, b)) => paint(l, a, b, delta::mix(t.bg, t.acc2, 0.42)),
            None => l,
        };
    }
    match app.read_selection() {
        Some(((r1, c1), (r2, c2))) if i >= r1 && i <= r2 => {
            let n = fr.read_text.get(i).map_or(0, |r| r.trim_end().chars().count());
            let from = if i == r1 { c1 } else { 0 };
            let to = if i == r2 { c2 + 1 } else { n.max(1) };
            paint(l, from, to, delta::mix(t.bg, t.acc2, 0.42))
        }
        _ => l,
    }
}

/// The row of the read view where the mark of "say it here" is, in this note.
fn say_row_read(app: &App, fr: &Cache, note: &Note) -> Option<usize> {
    let say = app.say.as_ref().filter(|s| s.path == note.path)?;
    let src: Vec<&str> = note.body.lines().collect();
    Some(crate::app::cursor_map::source_to_shown(&fr.read_text, &fr.read_rows, &src, (say.line, say.col)).0)
}

/// The arrow of "say it here": in the empty space left of the marked row.
pub(crate) fn say_arrow(f: &mut Frame, app: &App, t: &Theme, area: Rect, col: Rect, row: usize) {
    let h = col.height as usize;
    if row < app.reader.scroll() || row >= app.reader.scroll() + h || col.x < area.x + 2 {
        return;
    }
    let y = col.y + (row - app.reader.scroll()) as u16;
    let arrow = Span::styled("→", Style::new().fg(t.acc).add_modifier(Modifier::BOLD));
    f.render_widget(Paragraph::new(Line::from(arrow)), Rect { x: col.x - 2, y, width: 1, height: 1 });
}
