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
