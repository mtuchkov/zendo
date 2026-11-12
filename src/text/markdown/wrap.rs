//! Wrapping styled text to a width.
use super::*;

/// Word-wrap styled spans to `width` columns.
pub fn wrap(spans: Vec<Span<'static>>, width: usize, first: &str, rest: &str) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut cur: Vec<Span<'static>> = vec![Span::raw(first.to_string())];
    let mut cur_w = first.width();
    // Break into (text, style) tokens at spaces, keeping trailing spaces.
    let mut tokens: Vec<(String, Style)> = Vec::new();
    for span in spans {
        for piece in span.content.split_inclusive(' ') {
            tokens.push((piece.to_string(), span.style));
        }
    }
    for (tok, style) in tokens {
        let w = tok.trim_end().width();
        if cur_w + w > width && cur_w > rest.width().max(first.width()) {
            lines.push(Line::from(std::mem::take(&mut cur)));
            cur.push(Span::raw(rest.to_string()));
            cur_w = rest.width();
        }
        if w > width.saturating_sub(cur_w) && w > 0 {
            // A single token longer than the line: hard-break it.
            let mut piece = String::new();
            for ch in tok.chars() {
                let cw = ch.width().unwrap_or(0);
                if cur_w + cw > width {
                    cur.push(Span::styled(std::mem::take(&mut piece), style));
                    lines.push(Line::from(std::mem::take(&mut cur)));
                    cur.push(Span::raw(rest.to_string()));
                    cur_w = rest.width();
                }
                piece.push(ch);
                cur_w += cw;
            }
            cur.push(Span::styled(piece, style));
            continue;
        }
        cur_w += tok.width();
        cur.push(Span::styled(tok, style));
    }
    lines.push(Line::from(cur));
    lines
}

/// Cut styled text into rows of `width` cells, at any char.
pub(super) fn hard_wrap_styled(pieces: Vec<(Style, String)>, width: usize) -> Vec<Vec<Span<'static>>> {
    let mut out: Vec<Vec<Span<'static>>> = vec![vec![]];
    let mut w = 0;
    for (style, text) in pieces {
        let mut buf = String::new();
        for ch in text.chars() {
            let cw = ch.width().unwrap_or(0);
            if w + cw > width {
                if !buf.is_empty() {
                    out.last_mut().expect("one row").push(Span::styled(std::mem::take(&mut buf), style));
                }
                out.push(vec![]);
                w = 0;
            }
            buf.push(ch);
            w += cw;
        }
        if !buf.is_empty() {
            out.last_mut().expect("one row").push(Span::styled(buf, style));
        }
    }
    out
}

pub(super) fn pad(mut spans: Vec<Span<'static>>, width: usize, fill: Style) -> Line<'static> {
    let used: usize = spans.iter().map(|s| s.content.width()).sum();
    spans.push(Span::raw(" ".repeat(width.saturating_sub(used))));
    Line::from(spans).style(fill)
}
