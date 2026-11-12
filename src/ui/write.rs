//! Write mode: the note looks as in read mode; the line with the cursor shows its source.
use super::*;

pub(crate) fn draw_write(f: &mut Frame, app: &mut App, t: &Theme, area: Rect) {
    let col = column(area, app.column_width());
    let w = col.width as usize;
    app.text_width = w;
    app.regions.push((crate::text::select::Region::Note, col));
    let src: Vec<(String, Mark)> = app.editor.lines.iter().map(|l| (l.clone(), Mark::None)).collect();
    let r = markdown::render_live(&src, w, t, app.editor.row);
    let (vrow, vx) = write_cursor(app, &r.rows, w);
    if let Some(p) = app.editor_path.clone() {
        app.word_limit = app.word_limit_for(&p);
    }
    app.word_count = r.words.iter().sum();
    app.over_limit = over_limit(&r.words, app.word_limit);
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
    let lit = selected_rows(app, &r.rows, r.lines.len());
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
    app.cursor_y = vrow.saturating_sub(app.scroll);
    if !(app.panel && app.focus == Focus::Claude) {
        f.set_cursor_position((col.x + vx.min(w) as u16, col.y + (vrow - app.scroll) as u16));
    }
}

/// The rendered row and the column of the editor cursor: on the source rows of its line.
fn write_cursor(app: &App, rows: &[usize], w: usize) -> (usize, usize) {
    let e = &app.editor;
    let line = &e.lines[e.row];
    let segs = crate::text::editor::wrap_line(line, w);
    let k = segs.iter().rposition(|(a, _)| *a <= e.col).unwrap_or(0);
    let x: usize = line.chars().skip(segs[k].0).take(e.col - segs[k].0).map(|c| unicode_width::UnicodeWidthChar::width(c).unwrap_or(0)).sum();
    (rows.get(e.row).copied().unwrap_or(0) + k, x)
}

/// The selected rows: `(row, from, to)` in chars. The cursor line is exact; other lines are whole.
fn selected_rows(app: &App, rows: &[usize], total: usize) -> Vec<(usize, usize, usize)> {
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
