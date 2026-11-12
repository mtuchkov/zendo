//! Full-text search across notes. `#tag` filters by tag.

use crate::docs::notes::Note;

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
    if let Some(tag) = q.strip_prefix('#') {
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
    let needle = q.to_lowercase();
    let mut groups = Vec::new();
    for (i, note) in notes.iter().enumerate() {
        let mut count = 0;
        let mut snippets = Vec::new();
        for (ln, line) in note.body.lines().enumerate() {
            let hits = find_all(line, &needle);
            if hits.is_empty() {
                continue;
            }
            count += hits.len();
            let is_title = ln == 0 && line.starts_with("# ");
            if snippets.len() < 2 && !is_title {
                let text = line.trim().trim_start_matches(['#', '-', '*', ' ']).to_string();
                let hits = find_all(&text, &needle);
                snippets.push(Snippet { line: ln, text, hits });
            }
        }
        if count > 0 {
            groups.push(Group { note: i, count, snippets });
        }
    }
    groups.sort_by(|a, b| b.count.cmp(&a.count).then(notes[b.note].modified.cmp(&notes[a.note].modified)));
    groups
}

/// Case-insensitive matches, as byte ranges into `hay`.
fn find_all(hay: &str, needle_lower: &str) -> Vec<(usize, usize)> {
    if needle_lower.is_empty() {
        return vec![];
    }
    // Map each char of the lowercased text back to the original byte offset.
    let mut lower = String::new();
    let mut map = Vec::new();
    for (b, c) in hay.char_indices() {
        for lc in c.to_lowercase() {
            map.push((lower.len(), b, b + c.len_utf8()));
            lower.push(lc);
        }
    }
    let orig = |lb: usize, start: bool| -> usize {
        map.iter()
            .find(|(l, _, _)| *l >= lb)
            .map(|(_, s, e)| if start { *s } else { *e })
            .unwrap_or(hay.len())
    };
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(pos) = lower[from..].find(needle_lower) {
        let s = from + pos;
        let e = s + needle_lower.len();
        let os = orig(s, true);
        let oe = map.iter().rev().find(|(l, _, _)| *l < e).map(|(_, _, e)| *e).unwrap_or(hay.len());
        out.push((os, oe));
        from = e;
    }
    out
}

#[cfg(test)]
#[path = "search_tests.rs"]
mod tests;
