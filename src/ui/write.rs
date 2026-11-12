//! Write mode: the note looks as in read mode; the line with the cursor shows its source.
use super::*;

pub(crate) fn draw_write(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let col = column(area, app.column_width());
    let w = col.width as usize;
    app.text_width = w;
    app.regions.push((crate::text::select::Region::Note, col));
    let (r, head, title) = write_lines(app, t, w);
    let (vrow, vx) = write_cursor(app, &r.rows, head, title, w);
    if let Some(p) = app.editor_path.clone() {
        app.word_limit = app.word_limit_for(&p);
    }
    app.word_count = r.words.iter().sum();
    let mut words = vec![0; head];
    words.extend(&r.words);
    app.over_limit = over_limit(&words, app.word_limit);
    let h = col.height as usize;
    if let Some(y) = app.keep_y.take() {
        app.scroll = vrow.saturating_sub(y);
    }
    // The view follows the cursor, with a margin, as in read mode.
    let margin = 3.min(h / 4);
    if vrow < app.scroll + margin {
        app.scroll = vrow.saturating_sub(margin);
    } else if vrow + margin >= app.scroll + h {
        app.scroll = vrow + margin + 1 - h;
    }
    app.scroll = app.scroll.min(r.lines.len().saturating_sub(h));
    if let Some(row) = app.over_limit {
        let y0 = col.y as i64 + row as i64 - app.scroll as i64;
        draw_over_limit(f, app, t, area, col, y0.max(col.y as i64) as u16, app.word_count);
    }
    let lit = selected_rows(app, &r.rows, head, r.lines.len());
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
            None => l,
        })
        .collect();
    f.render_widget(Paragraph::new(view), col);
    if let Some(say) = app.say.as_ref().filter(|s| app.editor_path.as_ref() == Some(&s.path)) {
        let row = if say.line == app.editor.row { vrow } else if title == Some(say.line) { 0 } else { head + r.rows.get(say.line).copied().unwrap_or(0) };
        say_arrow(f, app, t, area, col, row);
    }
    app.cursor_y = vrow.saturating_sub(app.scroll);
    if !(app.panel && app.focus == Focus::Claude) {
        f.set_cursor_position((col.x + vx.min(w) as u16, col.y + (vrow - app.scroll) as u16));
    }
}

/// The lines of write mode: the head of the note as in read mode, then the text. The title line of
/// the source is in the head: as source while the cursor is on it. Returns the render with the head
/// in it, the rows of the head, and the source line of the title.
fn write_lines(app: &App, t: &Theme, w: usize) -> (markdown::Rendered, usize, Option<usize>) {
    let lines = &app.editor.lines;
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

/// The source line of the title: the first line with text after the front matter, when it is `# `.
fn title_line(lines: &[String]) -> Option<usize> {
    let front = lines.first().is_some_and(|l| l.trim() == "---");
    let after = if front { lines.iter().skip(1).position(|l| l.trim() == "---").map_or(0, |k| k + 2) } else { 0 };
    let i = (after..lines.len()).find(|k| !lines[*k].trim().is_empty())?;
    lines[i].starts_with("# ").then_some(i)
}

/// The rendered row and the column of the editor cursor: on the source rows of its line.
fn write_cursor(app: &App, rows: &[usize], head: usize, title: Option<usize>, w: usize) -> (usize, usize) {
    let e = &app.editor;
    let line = &e.lines[e.row];
    let segs = crate::text::editor::wrap_line(line, w);
    let k = segs.iter().rposition(|(a, _)| *a <= e.col).unwrap_or(0);
    let x: usize = line.chars().skip(segs[k].0).take(e.col - segs[k].0).map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0)).sum();
    let first = if title == Some(e.row) { 0 } else { head + rows.get(e.row).copied().unwrap_or(0) };
    (first + k, x)
}

/// The selected rows: `(row, from, to)` in chars. The cursor line is exact; other lines are whole.
fn selected_rows(app: &App, rows: &[usize], head: usize, total: usize) -> Vec<(usize, usize, usize)> {
    let rows: Vec<usize> = rows.iter().map(|r| r + head).collect();
    let total = total + head;
    let Some(((r1, c1), (r2, c2))) = app.editor.selection() else { return Vec::new() };
    let e = &app.editor;
    (r1..=r2)
        .flat_map(|line| {
            let first = rows.get(line).copied().unwrap_or(total);
            let end = rows.get(line + 1).copied().unwrap_or(total).max(first + 1);
            if line != e.row {
                return (first..end).map(|row| (row, 0, usize::MAX)).collect::<Vec<_>>();
            }
            let text = &e.lines[line];
            let (from, to) = (if line == r1 { c1 } else { 0 }, if line == r2 { c2 } else { text.chars().count() + 1 });
            crate::text::editor::wrap_line(text, app.text_width)
                .into_iter()
                .enumerate()
                .filter(|(_, (a, b))| from.max(*a) < to.min(b + 1))
                .map(|(k, (a, b))| (first + k, from.max(a) - a, to.min(b + 1) - a))
                .collect()
        })
        .collect()
}
