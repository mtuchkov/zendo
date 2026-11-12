//! Write mode: the note looks as in read mode; the line with the cursor shows its source.
use super::*;
use crate::app::write_rows::{self, RowSpan};

pub(crate) fn draw_write(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let code = app.editor_path.as_deref().is_some_and(code::is_code);
    let (gut, col) = if code { code_column(area, app.code_width(), app.editor.lines.len()) } else { (Rect::default(), column(area, app.column_width())) };
    let w = col.width as usize;
    app.text_width = w;
    app.regions.push((crate::text::select::Region::Note, col));
    let (mut r, head, title) = write_lines(app, t, w);
    let spans = row_spans(app, &r, head, title, w);
    let cur = write_rows::cursor_span(&spans, (app.editor.row, app.editor.col)).unwrap_or(0);
    let (vrow, vx) = spans.get(cur).map_or((0, 0), |s| (s.row, s.x_of((app.editor.row, app.editor.col))));
    // The cursor row shows the source of its own words: the rows around it keep their wrap. Code
    // shows its source on every row.
    if let Some(s) = spans.get(cur).filter(|s| s.row < r.lines.len() && !code) {
        r.lines[s.row] = Line::styled(s.text(), Style::new().fg(t.fg));
    }
    if let Some(p) = app.editor_path.clone() {
        app.word_limit = app.word_limit_for(&p);
    }
    app.word_count = r.words.iter().sum();
    let mut words = vec![0; head];
    words.extend(&r.words);
    app.over_limit = over_limit(&words, app.word_limit);
    let h = col.height as usize;
    follow(app, vrow, h, r.lines.len());
    if let Some(row) = app.over_limit {
        let y0 = col.y as i64 + row as i64 - app.scroll as i64;
        draw_over_limit(f, app, t, area, col, y0.max(col.y as i64) as u16, app.word_count);
    }
    let lit = selected_rows(app, &spans, cur);
    app.write_rows = spans;
    let rows: Vec<usize> = r.rows.iter().map(|x| x + head).collect();
    let rel = app.editor_path.as_deref().map(|p| app.rel(p)).unwrap_or_default();
    let marks = confusing_rows(app, &rel, &rows, r.lines.len());
    let sel_bg = delta::mix(t.bg, t.acc2, 0.42);
    let view: Vec<Line> = r
        .lines
        .into_iter()
        .enumerate()
        .skip(app.scroll)
        .take(h)
        .map(|(i, l)| match lit.iter().find(|(row, ..)| *row == i) {
            Some((_, a, b)) => paint(l, *a, *b, sel_bg),
            None if i == vrow => tint(l, cursor_line(t), w),
            None if marks.contains(&i) => tint(l, confusing(t), w),
            None => l,
        })
        .collect();
    // The markup of the cursor row can make it longer than the column: it shows on the margin.
    let wide = Rect { width: (area.x + area.width).saturating_sub(col.x).min(col.width + 12), ..col };
    f.render_widget(Paragraph::new(view), wide);
    if code {
        draw_gutter(f, t, gut, &rows, app.scroll, Some(vrow));
    }
    if let Some(say) = app.say.as_ref().filter(|s| app.editor_path.as_ref() == Some(&s.path)) {
        let row = if say.line == app.editor.row { vrow } else if title == Some(say.line) { 0 } else { head + r.rows.get(say.line).copied().unwrap_or(0) };
        say_arrow(f, app, t, area, col, row);
    }
    app.cursor_y = vrow.saturating_sub(app.scroll);
    if !(app.panel && app.focus == Focus::Claude) {
        f.set_cursor_position((col.x + vx.min(wide.width as usize) as u16, col.y + (vrow - app.scroll) as u16));
    }
}

/// The view follows the cursor row, with a margin, as in read mode.
fn follow(app: &mut App, vrow: usize, h: usize, total: usize) {
    if let Some(y) = app.keep_y.take() {
        app.scroll = vrow.saturating_sub(y);
    }
    let margin = 3.min(h / 4);
    if vrow < app.scroll + margin {
        app.scroll = vrow.saturating_sub(margin);
    } else if vrow + margin >= app.scroll + h {
        app.scroll = vrow + margin + 1 - h;
    }
    app.scroll = app.scroll.min(total.saturating_sub(h));
}

/// The lines of write mode: the head of the note as in read mode, then the text. The title line of
/// the source is in the head: as source while the cursor is on it. Returns the render with the head
/// in it, the rows of the head, and the source line of the title.
fn write_lines(app: &App, t: &Theme, w: usize) -> (markdown::Rendered, usize, Option<usize>) {
    let lines = &app.editor.lines;
    if let Some(p) = app.editor_path.as_deref().filter(|p| code::is_code(p)) {
        return code_lines(app, p, t, w);
    }
    let title = title_line(lines);
    let on_title = title.is_some_and(|i| i == app.editor.row);
    let title_rows = match title {
        Some(i) if on_title => markdown::raw_rows(&lines[i], w, t),
        Some(i) => title_rows(lines[i].trim_start_matches('#').trim(), w, t),
        None => Vec::new(),
    };
    let note = app.note().filter(|n| app.editor_path.as_ref() == Some(&n.path)).cloned();
    let mut out = match &note {
        Some(n) => note_head(app, n, w, t, title_rows),
        None => title_rows,
    };
    let head = out.len();
    let src: Vec<(String, Mark)> = lines.iter().map(|l| (l.clone(), Mark::None)).collect();
    let mut r = markdown::render_live(&src, w, t, (!on_title).then_some(app.editor.row));
    out.append(&mut r.lines);
    r.lines = out;
    (r, head, title)
}

/// The lines of write mode for a code file: the head with the file name, then the code as read mode
/// shows it.
fn code_lines(app: &App, p: &std::path::Path, t: &Theme, w: usize) -> (markdown::Rendered, usize, Option<usize>) {
    let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let mut out = match app.note().filter(|n| n.path == p) {
        Some(n) => note_head(app, n, w, t, title_rows(&n.title, w, t)),
        None => title_rows(&name, w, t),
    };
    let head = out.len();
    let mut r = code::render(&app.editor.lines, &code::lang_of(p), w, t);
    out.append(&mut r.lines);
    r.lines = out;
    (r, head, None)
}

/// The source line of the title: the first line with text after the front matter, when it is `# `.
fn title_line(lines: &[String]) -> Option<usize> {
    let front = lines.first().is_some_and(|l| l.trim() == "---");
    let after = if front { lines.iter().skip(1).position(|l| l.trim() == "---").map_or(0, |k| k + 2) } else { 0 };
    let i = (after..lines.len()).find(|k| !lines[*k].trim().is_empty())?;
    lines[i].starts_with("# ").then_some(i)
}

/// The rows on the screen as spans of the source: the title rows in the head, then the rows of the
/// text where they start.
fn row_spans(app: &App, r: &markdown::Rendered, head: usize, title: Option<usize>, w: usize) -> Vec<RowSpan> {
    let lines = &app.editor.lines;
    let shown = |row: usize| -> String { r.lines.get(row).map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect()).unwrap_or_default() };
    let mut starts: Vec<(usize, (usize, usize), usize)> = match title {
        Some(i) if i == app.editor.row => crate::text::editor::wrap_line(&lines[i], w).into_iter().enumerate().map(|(k, (a, _))| (k, (i, a), 0)).collect(),
        Some(i) => vec![(0, (i, 0), 0)],
        None => Vec::new(),
    };
    for (k, start) in r.starts.iter().enumerate() {
        let Some((l, c)) = *start else { continue };
        let code = app.editor_path.as_deref().is_some_and(code::is_code);
        let indent = if c > 0 && !code { shown(head + k).chars().take_while(|ch| *ch == ' ').count() } else { 0 };
        starts.push((head + k, (l, c), indent));
    }
    let line_rows: Vec<usize> = (0..lines.len()).map(|l| if title == Some(l) { 0 } else { head + r.rows.get(l).copied().unwrap_or(0) }).collect();
    write_rows::spans(lines, &starts, &line_rows)
}

/// The selected rows: `(row, from, to)` in columns. The cursor row is exact; the others are whole.
fn selected_rows(app: &App, spans: &[RowSpan], cur: usize) -> Vec<(usize, usize, usize)> {
    let Some((a, b)) = app.editor.selection() else { return Vec::new() };
    spans
        .iter()
        .enumerate()
        .filter_map(|(k, s)| {
            let start = s.chars.first().map_or(s.end, |c| c.0);
            if k == cur {
                let (from, to) = (s.x_of(a.max(start)), s.x_of(b.min(s.end)));
                (from < to).then_some((s.row, from, to))
            } else {
                (start < b && s.end > a).then_some((s.row, 0, usize::MAX))
            }
        })
        .collect()
}
