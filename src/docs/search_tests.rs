use super::*;
use chrono::Local;
use std::path::PathBuf;

fn note(body: &str) -> Note {
    Note { path: PathBuf::from("x.md"), title: "x".into(), created: None, modified: Local::now(), body: body.into() }
}

#[test]
fn finds_case_insensitive_with_ranges() {
    let notes = vec![note("# A\nThe Idempotency-Key header\nnot here"), note("nothing")];
    let g = search(&notes, "idempot");
    assert_eq!(g.len(), 1);
    let s = &g[0].snippets[0];
    assert_eq!(&s.text[s.hits[0].0..s.hits[0].1], "Idempot");
}

#[test]
fn tag_filter() {
    let notes = vec![note("#orders here"), note("#billing")];
    let g = search(&notes, "#ord");
    assert_eq!(g.len(), 1);
    assert_eq!(g[0].note, 0);
}

#[test]
fn unicode_ranges() {
    let r = find_all("Кэш и TTL", "кэш");
    assert_eq!(r, vec![(0, "Кэш".len())]);
}

// FIND-9: the fast matcher of ASCII text gives the same ranges as the matcher of any text.
#[test]
fn ascii_matches_are_the_matches_of_any_text() {
    let cases = [("Tea, TEA and tEa", "tea"), ("aaaa", "aa"), ("no match here", "tea"), ("x", "longer"), ("ab", "ab")];
    for (hay, needle) in cases {
        assert_eq!(find_all(hay, needle), find_mapped(hay, needle), "{hay:?} / {needle:?}");
    }
    assert_eq!(find_all("Tea, TEA and tEa", "tea"), vec![(0, 3), (5, 8), (13, 16)]);
}

// FIND-9: an ASCII query in a line with other letters still finds its place in bytes.
#[test]
fn an_ascii_query_in_a_line_with_other_letters() {
    let hay = "Кэш и TTL и ttl";
    let at = hay.find("TTL").unwrap();
    assert_eq!(find_all(hay, "ttl"), vec![(at, at + 3), (hay.len() - 3, hay.len())]);
}

// FIND-9: a note with the query only in the title line counts, and one without it has no group.
#[test]
fn a_note_without_the_query_has_no_group() {
    let notes = vec![note("# Tea\nnothing\n"), note("# X\nno match at all\n"), note("Кэш\nи ТЕА\n")];
    let g = search(&notes, "tea");
    assert_eq!(g.iter().map(|g| g.note).collect::<Vec<_>>(), vec![0]);
    assert_eq!(search(&notes, "теа").iter().map(|g| g.note).collect::<Vec<_>>(), vec![2]);
}
