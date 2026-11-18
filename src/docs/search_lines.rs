//! A query with `⏎`: the text across lines (FIND-20). Each `⏎` matches a line break; a space
//! matches a space or a line break, because read mode joins the lines of a paragraph. Letter case,
//! indentation, list and heading marks and blank lines do not count.
use crate::docs::notes::Note;
use crate::docs::search::{Group, NEW_LINE, Snippet, find_all};

/// The notes whose text has the query across lines, with the first lines of the matches.
pub(super) fn search(notes: &[Note], q: &str) -> Vec<Group> {
    let needle: Vec<char> = flat(&q.replace(NEW_LINE, "\n"))
        .into_iter()
        .map(|(c, _)| c)
        .collect();
    let head = q.split(NEW_LINE).next().unwrap_or("").trim().to_lowercase();
    notes
        .iter()
        .enumerate()
        .filter_map(|(i, n)| group(i, n, &needle, &head))
        .collect()
}

fn group(note: usize, n: &Note, needle: &[char], head: &str) -> Option<Group> {
    let lines = starts(&flat(&n.body), needle);
    if lines.is_empty() {
        return None;
    }
    let src: Vec<&str> = n.body.lines().collect();
    let mut shown = lines.clone();
    shown.dedup();
    let snippet = |ln: usize| {
        let text = src[ln]
            .trim()
            .trim_start_matches(['#', '-', '*', ' '])
            .to_string();
        Snippet {
            line: ln,
            hits: find_all(&text, head),
            text,
        }
    };
    Some(Group {
        note,
        count: lines.len(),
        snippets: shown.into_iter().take(2).map(snippet).collect(),
    })
}

/// The source line where each match starts.
fn starts(hay: &[(char, usize)], needle: &[char]) -> Vec<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return Vec::new();
    }
    let fits = |i: &usize| {
        needle
            .iter()
            .zip(&hay[*i..])
            .all(|(n, (h, _))| n == h || (*n == ' ' && *h == '\n'))
    };
    (0..=hay.len() - needle.len())
        .filter(fits)
        .map(|i| hay[i].1)
        .collect()
}

/// The text in lower case, each char with its source line: without the marks at the start of a
/// line, and each run of white space as one `\n` when it has a line break, else as one space.
fn flat(text: &str) -> Vec<(char, usize)> {
    let chars = text.lines().enumerate().flat_map(|(ln, l)| {
        let body = unmark(l.trim()).chars().flat_map(char::to_lowercase);
        std::iter::once(('\n', ln)).chain(body.map(move |c| (c, ln)))
    });
    let mut out: Vec<(char, usize)> = Vec::new();
    let mut gap = None;
    for (c, ln) in chars {
        if c.is_whitespace() {
            gap = Some(if c == '\n' || gap == Some('\n') {
                '\n'
            } else {
                ' '
            });
            continue;
        }
        if let Some(g) = gap.take().filter(|_| !out.is_empty()) {
            out.push((g, ln));
        }
        out.push((c, ln));
    }
    out
}

/// The line without a heading, quote or list mark at its start: in the source and in read mode.
fn unmark(l: &str) -> &str {
    let l = l.trim_start_matches(['#', '>']).trim_start();
    let num = l.trim_start_matches(|c: char| c.is_ascii_digit());
    let l = match num.strip_prefix(". ").or_else(|| num.strip_prefix(") ")) {
        Some(rest) if num.len() < l.len() => rest,
        _ => l,
    };
    ["- ", "* ", "+ ", "• "]
        .iter()
        .find_map(|m| l.strip_prefix(m))
        .unwrap_or(l)
}

#[cfg(test)]
#[path = "search_lines_tests.rs"]
mod tests;
