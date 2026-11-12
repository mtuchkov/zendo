//! The review on screen: its summary in the head of the file, and the confusing lines marked.
use super::*;

/// The review block under the recap: the summary, and each confusing line with why.
pub(crate) fn review_lines(app: &App, note: &Note, width: usize, t: &Theme) -> Vec<Line<'static>> {
    let rel = app.rel(&note.path);
    let dim = Style::new().fg(t.dim);
    let bar = Span::styled("▎ ", Style::new().fg(t.del));
    let reading = app.recap_job.as_ref().is_some_and(|j| j.review && j.rel == rel) || app.reviews.queue.contains(&rel);
    let Some(r) = app.review_of(&rel) else {
        return if reading { vec![Line::from(vec![bar, Span::styled("REVIEW · reading the file…", dim.add_modifier(Modifier::BOLD))]), Line::raw("")] } else { Vec::new() };
    };
    let n = r.lines.len();
    let mut head = format!("REVIEW · {} · {n} confusing line{}", r.at.format("%a %-d %b %H:%M"), if n == 1 { "" } else { "s" });
    if reading {
        head.push_str(" · reading again…");
    }
    let mut out = vec![Line::from(vec![bar.clone(), Span::styled(head, dim.add_modifier(Modifier::BOLD))])];
    let rows = markdown::wrap(vec![Span::styled(r.summary.clone(), Style::new().fg(t.fg))], width.saturating_sub(2), "", "");
    let marked = r.lines.iter().map(|(l, why)| markdown::wrap(vec![Span::styled(format!("line {}", l + 1), Style::new().fg(t.del)), Span::styled(format!(": {why}"), dim)], width.saturating_sub(2), "", ""));
    for l in rows.into_iter().chain(marked.flatten()) {
        out.push(Line::from(std::iter::once(bar.clone()).chain(l.spans).collect::<Vec<_>>()));
    }
    out.push(Line::raw(""));
    out
}

/// The screen rows of the confusing lines of the open file. `rows` has the first row of each source
/// line; `total` is the number of rows.
pub(crate) fn confusing_rows(app: &App, rel: &str, rows: &[usize], total: usize) -> Vec<usize> {
    let Some(r) = app.review_of(rel) else { return Vec::new() };
    r.lines
        .iter()
        .filter_map(|(l, _)| rows.get(*l).map(|a| (*a, rows.get(l + 1).copied().unwrap_or(total).max(a + 1))))
        .flat_map(|(a, b)| a..b)
        .collect()
}

/// The color of a confusing row.
pub(crate) fn confusing(t: &Theme) -> Color {
    delta::mix(t.bg, t.del, 0.22)
}
