//! Full-text search across notes. `#tag` filters by tag.

use crate::docs::notes::Note;

/// A line break in a query: the find panel shows a new line of the selection as this char (FIND-19).
pub const NEW_LINE: char = '\u{23CE}';

#[derive(Clone, Debug)]
pub struct Snippet {
    pub line: usize,
    pub text: String,
    /// Byte ranges of matches in `text`.
    pub hits: Vec<(usize, usize)>,
}

#[derive(Clone, Debug)]
pub struct Group {
    pub note: usize,
    pub count: usize,
    pub snippets: Vec<Snippet>,
}

pub fn search(notes: &[Note], query: &str) -> Vec<Group> {
    let q = query.trim();
    if q.is_empty() {
        return Vec::new();
    }
    if let Some(tag) = q.strip_prefix('#').filter(|_| !q.contains(NEW_LINE)) {
        let tag = tag.to_lowercase();
        return notes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.tags().iter().any(|t| t.to_lowercase().starts_with(&tag)))
            .map(|(i, n)| Group {
                note: i,
                count: 1,
                snippets: vec![Snippet { line: 0, text: n.tags().iter().map(|t| format!("#{t}")).collect::<Vec<_>>().join(" "), hits: vec![] }],
            })
            .collect();
    }
    let mut groups = if q.contains(NEW_LINE) { super::search_lines::search(notes, q) } else { by_line(notes, &q.to_lowercase()) };
    groups.sort_by(|a, b| b.count.cmp(&a.count).then(notes[b.note].modified.cmp(&notes[a.note].modified)));
    groups
}

/// The notes with the query in a line.
fn by_line(notes: &[Note], needle: &str) -> Vec<Group> {
    let mut groups = Vec::new();
    // A note without the query in its text needs no look at its lines.
    for (i, note) in notes.iter().enumerate().filter(|(_, n)| has(&n.body, needle)) {
        let mut count = 0;
        let mut snippets = Vec::new();
        for (ln, line) in note.body.lines().enumerate() {
            let hits = find_all(line, needle);
            if hits.is_empty() {
                continue;
            }
            count += hits.len();
            let is_title = ln == 0 && line.starts_with("# ");
            if snippets.len() < 2 && !is_title {
                let text = line.trim().trim_start_matches(['#', '-', '*', ' ']).to_string();
                let hits = find_all(&text, needle);
                snippets.push(Snippet { line: ln, text, hits });
            }
        }
        if count > 0 {
            groups.push(Group { note: i, count, snippets });
        }
    }
    groups
}

/// Case-insensitive matches, as byte ranges into `hay`. ASCII text needs no lower-case copy.
pub(super) fn find_all(hay: &str, needle_lower: &str) -> Vec<(usize, usize)> {
    if needle_lower.is_empty() {
        return vec![];
    }
    if hay.is_ascii() && needle_lower.is_ascii() {
        return find_ascii(hay.as_bytes(), needle_lower.as_bytes());
    }
    if !lower(hay).contains(needle_lower) {
        return vec![];
    }
    find_mapped(hay, needle_lower)
}

/// The text has the query, without regard to letter case.
pub(super) fn has(hay: &str, needle_lower: &str) -> bool {
    if hay.is_ascii() && needle_lower.is_ascii() {
        return first_ascii(hay.as_bytes(), needle_lower.as_bytes()).is_some();
    }
    lower(hay).contains(needle_lower)
}

/// The text in lower case, char by char, as `find_mapped` reads it.
fn lower(hay: &str) -> String {
    hay.chars().flat_map(char::to_lowercase).collect()
}

fn find_ascii(hay: &[u8], needle: &[u8]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(at) = first_ascii(&hay[from..], needle) {
        let s = from + at;
        out.push((s, s + needle.len()));
        from = s + needle.len();
    }
    out
}

fn first_ascii(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w.eq_ignore_ascii_case(needle))
}

/// The matches in any text: a lower-case copy, with each char mapped back to its byte offset.
fn find_mapped(hay: &str, needle_lower: &str) -> Vec<(usize, usize)> {
    let mut lower = String::new();
    let mut map = Vec::new();
    for (b, c) in hay.char_indices() {
        for lc in c.to_lowercase() {
            map.push((lower.len(), b, b + c.len_utf8()));
            lower.push(lc);
        }
    }
    let orig = |lb: usize| -> usize { map.iter().find(|(l, _, _)| *l >= lb).map_or(hay.len(), |(_, s, _)| *s) };
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(pos) = lower[from..].find(needle_lower) {
        let s = from + pos;
        let e = s + needle_lower.len();
        let oe = map.iter().rev().find(|(l, _, _)| *l < e).map_or(hay.len(), |(_, _, e)| *e);
        out.push((orig(s), oe));
        from = e;
    }
    out
}

#[cfg(test)]
#[path = "search_tests.rs"]
mod tests;
