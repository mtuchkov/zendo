//! Write mode: where each rendered row starts in the source, so that the row with the cursor can
//! show the source of its own words and keep the wrap of read mode.
use super::*;
use crate::text::align::{align, joined, split, to_source};

/// The starts of the rows `from..` of `out`: the render of the source lines `first..first + n`.
/// A logical line that shows nothing (a line of tags) shows its source while the cursor is on it.
pub(super) fn add(r: &mut Renderer, starts: &mut Vec<Option<(usize, usize)>>, from: usize, src: &[(String, Mark)], (first, n): (usize, usize), cursor: bool) {
    starts.resize(from, None);
    if r.out.len() == from && cursor {
        let line = &src[first].0;
        r.out.extend(raw_rows(line, r.width, r.t));
        starts.extend(crate::text::editor::wrap_line(line, r.width).into_iter().map(|(a, _)| Some((first, a))));
        return;
    }
    let shown: Vec<String> = r.out[from..].iter().map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect()).collect();
    let lines: Vec<&str> = src[first..first + n].iter().map(|(l, _)| l.as_str()).collect();
    starts.extend(row_starts(&shown, &lines, first).into_iter().map(Some));
}

/// Where each row of one logical line starts: the first row at the start of its first line, the
/// others at the word that begins them, with the markup before the word.
fn row_starts(shown: &[String], src: &[&str], first: usize) -> Vec<(usize, usize)> {
    let (view, view_starts) = joined(shown);
    let (source, src_starts) = joined(src);
    let map = align(&view, &source);
    let mut prev = 0;
    view_starts
        .iter()
        .enumerate()
        .map(|(k, &i)| {
            let mut j = if k == 0 { 0 } else { to_source(&map, i, source.len()).max(prev) };
            while j > prev && !source[j - 1].is_whitespace() {
                j -= 1;
            }
            prev = j;
            let (l, c) = split(&src_starts, j);
            (first + l, c)
        })
        .collect()
}
