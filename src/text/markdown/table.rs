//! Tables with solid borders that fit the width, the body rows striped.
use super::*;
use crate::ui::theme::mix;

/// How far the stripe of a table row goes from the page color to the text color (NOTE-34).
pub(super) const TABLE_STRIPE: f32 = 0.05;

pub(super) struct Table {
    pub(super) lines: Vec<Line<'static>>,
    /// For each source row of the table (separator included), its first rendered row.
    pub(super) rows: Vec<usize>,
}

pub(super) fn is_separator(t: &str) -> bool {
    let t = t.trim();
    t.starts_with('|') && t.contains('-') && t.chars().all(|c| matches!(c, '|' | '-' | ':' | ' '))
}

pub(super) fn cells(t: &str) -> Vec<String> {
    let t = t.trim();
    let t = t.strip_prefix('|').unwrap_or(t);
    let t = t.strip_suffix('|').unwrap_or(t);
    let mut out = vec![String::new()];
    let mut esc = false;
    for c in t.chars() {
        match c {
            '\\' if !esc => esc = true,
            '|' if !esc => out.push(String::new()),
            c => {
                if esc && c != '|' {
                    out.last_mut().expect("one cell").push('\\');
                }
                out.last_mut().expect("one cell").push(c);
                esc = false;
            }
        }
    }
    out.into_iter().map(|c| c.trim().to_string()).collect()
}

/// A markdown table that starts at logical `i`: drawn with solid borders, the cells wrapped to fit
/// the width. None when it is not a table, or when it does not fit even with narrow columns.
pub(super) fn table_at(logicals: &[Logical], i: usize, width: usize, t: &Theme, raw: Option<usize>) -> Option<(usize, Table)> {
    let row_ok = |k: usize| logicals.get(k).is_some_and(|l| !l.verbatim && l.text.trim_start().starts_with('|'));
    if !row_ok(i) || !logicals.get(i + 1).is_some_and(|l| is_separator(&l.text)) {
        return None;
    }
    let mut end = i + 2;
    while row_ok(end) {
        end += 1;
    }
    let align: Vec<char> = cells(&logicals[i + 1].text)
        .iter()
        .map(|c| match (c.starts_with(':'), c.ends_with(':')) {
            (true, true) => 'c',
            (false, true) => 'r',
            _ => 'l',
        })
        .collect();
    let body: Vec<Vec<String>> = std::iter::once(i).chain(i + 2..end).map(|k| cells(&logicals[k].text)).collect();
    let cols = body.iter().map(Vec::len).max().unwrap_or(0).max(align.len());
    if cols == 0 {
        return None;
    }
    let base = Style::new().fg(t.fg);
    let head = base.add_modifier(Modifier::BOLD);
    let spans = |r: usize, c: usize| -> Vec<Span<'static>> {
        let text = body[r].get(c).cloned().unwrap_or_default();
        inline(&text, if r == 0 { head } else { base }, t)
    };
    let natural: Vec<usize> = (0..cols).map(|c| (0..body.len()).map(|r| spans(r, c).iter().map(|s| s.content.width()).sum::<usize>()).max().unwrap_or(0).max(1)).collect();
    let w = fit_columns(natural, width)?;
    let border = Style::new().fg(t.dim);
    let rule = |l: &str, m: &str, r: &str| {
        let mid: Vec<String> = w.iter().map(|n| "─".repeat(n + 2)).collect();
        Line::from(Span::styled(format!("{l}{}{r}", mid.join(m)), border))
    };
    let mut lines = vec![rule("┌", "┬", "┐")];
    let mut rows = Vec::new();
    for r in 0..body.len() {
        rows.push(lines.len());
        // In write mode, the row with the cursor shows its source.
        let at = if r == 0 { i } else { i + 1 + r };
        if raw == Some(at) {
            lines.extend(raw_rows(&logicals[at].text, width, t));
        } else {
            let wrapped: Vec<Vec<Line<'static>>> = (0..cols).map(|c| wrap(spans(r, c), w[c], "", "")).collect();
            let h = wrapped.iter().map(Vec::len).max().unwrap_or(1).max(1);
            for k in 0..h {
                let mut out = vec![Span::styled("│", border)];
                for c in 0..cols {
                    let cell = trim_cell(wrapped[c].get(k).map(|l| l.spans.clone()).unwrap_or_default());
                    out.extend(align_cell(cell, w[c], align.get(c).copied()));
                    out.push(Span::styled("│", border));
                }
                lines.push(striped(Line::from(out), r, t));
            }
        }
        if r == 0 {
            rows.push(lines.len());
            if raw == Some(i + 1) {
                lines.extend(raw_rows(&logicals[i + 1].text, width, t));
            } else {
                lines.push(rule("├", "┼", "┤"));
            }
        }
    }
    lines.push(rule("└", "┴", "┘"));
    Some((end, Table { lines, rows }))
}

/// Source row `r` of a table (0 is the head): the second body row and each other row after it get
/// the stripe background (NOTE-34). A span with its own background, such as inline code, keeps it.
fn striped(line: Line<'static>, r: usize, t: &Theme) -> Line<'static> {
    if r == 0 || r % 2 == 1 {
        return line;
    }
    let bg = mix(t.bg, t.fg, TABLE_STRIPE);
    let spans = line.spans.into_iter().map(|s| if s.style.bg.is_some() { s } else { let st = s.style.bg(bg); s.style(st) }).collect::<Vec<_>>();
    Line::from(spans)
}

/// Column widths that fit the width: the widest column gives up a char at a time, down to 3.
fn fit_columns(natural: Vec<usize>, width: usize) -> Option<Vec<usize>> {
    let cols = natural.len();
    let frame = cols + 1 + 2 * cols;
    if width < frame + 3 * cols {
        return None;
    }
    let mut w = natural;
    while w.iter().sum::<usize>() + frame > width {
        let (k, _) = w.iter().enumerate().max_by_key(|(_, v)| **v).expect("columns");
        if w[k] <= 3 {
            break;
        }
        w[k] -= 1;
    }
    Some(w)
}

/// A wrapped row of a cell without the spaces at its ends: they would make it wider than its column.
fn trim_cell(mut cell: Vec<Span<'static>>) -> Vec<Span<'static>> {
    // A wrapped row can end in a space.
    while let Some(last) = cell.last_mut() {
        let t = last.content.trim_end().to_string();
        if t.is_empty() {
            cell.pop();
            continue;
        }
        last.content = t.into();
        break;
    }
    // And a wrapped row can start with the pad space of inline code.
    while let Some(first) = cell.first_mut() {
        let t = first.content.trim_start().to_string();
        if t.is_empty() {
            cell.remove(0);
            continue;
        }
        first.content = t.into();
        break;
    }
    cell
}

/// The cell padded to its column, aligned left, right or center.
fn align_cell(cell: Vec<Span<'static>>, width: usize, align: Option<char>) -> Vec<Span<'static>> {
    let used: usize = cell.iter().map(|s| s.content.width()).sum();
    let gap = width.saturating_sub(used);
    let (before, after) = match align {
        Some('r') => (gap, 0),
        Some('c') => (gap / 2, gap - gap / 2),
        _ => (0, gap),
    };
    let mut out = vec![Span::raw(" ".repeat(1 + before))];
    out.extend(cell);
    out.push(Span::raw(" ".repeat(after + 1)));
    out
}
