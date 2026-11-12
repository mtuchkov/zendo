//! Read mode: the note formatted for the pane, with its recap and the cursor line.
use super::*;

/// The lines of the read view for a note, and the row of each source line.
pub(crate) fn read_lines(app: &App, note: &Note, width: usize, t: &Theme) -> (Vec<Line<'static>>, Vec<usize>, Vec<usize>) {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let title = Span::styled(note.title.clone(), Style::new().fg(t.fg).add_modifier(Modifier::BOLD));
    lines.extend(markdown::wrap(vec![title], width, "", ""));
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
    let head = lines.len();

    // A pending edit to this note shows inline, marked.
    let key = app.rel(&note.path);
    let proposal = app.threads.get(&key).and_then(|th| {
        th.items.iter().rev().find_map(|it| match it {
            Item::Proposal { diffs, state: Answered::Pending, .. } => diffs.iter().find(|d| d.path == note.path).cloned(),
            _ => None,
        })
    });
    let mut src = match proposal {
        Some(d) => markdown::diff_lines(d.old.as_deref().unwrap_or(&note.body), &d.new),
        None => markdown::plain(&note.body),
    };
    // A file that is not markdown (e.g. a .toml from the project structure) shows as code.
    let ext = note.path.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default();
    if ext != "md" {
        src.insert(0, (format!("```{ext}"), Mark::None));
        src.push(("```".into(), Mark::None));
    }
    let r = markdown::render(&src, width, t, true);
    let rows = r.rows.iter().map(|x| x + head).collect();
    let mut words = vec![0; head];
    words.extend(r.words);
    lines.extend(r.lines);
    (lines, rows, words)
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

pub(crate) fn draw_read(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let col = column(area, app.column_width());
    app.text_width = col.width as usize;
    let Some(note) = app.note().cloned() else {
        let msg = vec![
            Line::styled("No notes yet.", Style::new().fg(t.fg).add_modifier(Modifier::BOLD)),
            Line::raw(""),
            Line::styled(format!("Press n to write the first one. It goes to {}.", app.rel(&app.cfg.notes_dir)), Style::new().fg(t.dim)),
            Line::styled("Press c to talk to Claude, ? for keys.", Style::new().fg(t.dim)),
        ];
        let y = col.y + col.height / 3;
        f.render_widget(Paragraph::new(msg), Rect { y, height: col.height.saturating_sub(y - col.y), ..col });
        return;
    };
    app.regions.push((crate::text::select::Region::Note, col));
    let (lines, rows, words) = read_lines(app, &note, col.width as usize, t);
    app.word_limit = app.word_limit_for(&note.path);
    app.word_count = words.iter().sum();
    app.over_limit = over_limit(&words, app.word_limit);
    app.read_text = lines.iter().map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect()).collect();
    app.read_rows = rows;
    let h = col.height as usize;
    app.read_cursor = app.read_cursor.min(lines.len().saturating_sub(1));
    if let Some(y) = app.keep_y.take() {
        app.scroll = app.read_cursor.saturating_sub(y);
    }
    // The view follows the cursor, with a margin.
    let margin = 3.min(h / 4);
    if app.read_cursor < app.scroll + margin {
        app.scroll = app.read_cursor.saturating_sub(margin);
    } else if app.read_cursor + margin >= app.scroll + h {
        app.scroll = app.read_cursor + margin + 1 - h;
    }
    let max = lines.len().saturating_sub(h);
    app.scroll = app.scroll.min(max);
    app.cursor_y = app.read_cursor.saturating_sub(app.scroll);
    if let Some(r) = app.over_limit {
        let y0 = col.y as i64 + r as i64 - app.scroll as i64;
        draw_over_limit(f, app, t, area, col, y0.max(col.y as i64) as u16, app.word_count);
    }
    // The cursor and the selection show only where the keys go.
    let active = app.mode == Mode::Read && app.focus == Focus::Note;
    let len = app.read_text.get(app.read_cursor).map_or(0, |r| r.trim_end().chars().count());
    app.read_col = app.read_col.min(len.saturating_sub(1));
    let mut view: Vec<Line> = lines.into_iter().enumerate().skip(app.scroll).take(h).map(|(i, l)| if active { mark_row(app, t, i, l, col.width as usize) } else { l }).collect();
    say_arrow(app, t, &note.path, &mut view, h);
    f.render_widget(Paragraph::new(view), col);
    // The block cursor on the current char; the terminal makes it blink.
    if active && app.read_cursor >= app.scroll && app.read_cursor < app.scroll + h {
        let row = app.read_text.get(app.read_cursor).cloned().unwrap_or_default();
        let x: usize = row.chars().take(app.read_col).map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0)).sum();
        f.set_cursor_position((col.x + (x as u16).min(col.width.saturating_sub(1)), col.y + (app.read_cursor - app.scroll) as u16));
    }
}

/// The cursor line and the selection on one row.
fn mark_row(app: &App, t: &Theme, i: usize, l: Line<'static>, width: usize) -> Line<'static> {
    let l = if i == app.read_cursor { tint(l, cursor_line(t), width) } else { l };
    match app.read_selection() {
        Some(((r1, c1), (r2, c2))) if i >= r1 && i <= r2 => {
            let n = app.read_text.get(i).map_or(0, |r| r.trim_end().chars().count());
            let from = if i == r1 { c1 } else { 0 };
            let to = if i == r2 { c2 + 1 } else { n.max(1) };
            paint(l, from, to, delta::mix(t.bg, t.acc2, 0.42))
        }
        _ => l,
    }
}

/// The arrow of `ctrl-h`: a line under the marked row, with `↑` under the marked char.
fn say_arrow(app: &App, t: &Theme, path: &std::path::Path, view: &mut Vec<Line<'static>>, h: usize) {
    let Some(say) = app.say.as_ref().filter(|s| s.path == path) else { return };
    if say.row < app.scroll || say.row >= app.scroll + h {
        return;
    }
    let row = app.read_text.get(say.row).cloned().unwrap_or_default();
    let x: usize = row.chars().take(say.col).map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0)).sum();
    let what = if say.rewrite { "say this differently" } else { "say it here" };
    let line = Line::from(vec![Span::raw(" ".repeat(x)), Span::styled(format!("↑ {what}"), Style::new().fg(t.acc).add_modifier(Modifier::BOLD))]);
    view.insert(say.row - app.scroll + 1, line);
    view.truncate(h);
}
