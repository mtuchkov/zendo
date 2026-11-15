//! The lists without an app (ADR 0008).
use super::*;
use chrono::Local;

fn note(name: &str, body: &str) -> Note {
    Note { path: name.into(), title: name.into(), created: None, modified: Local::now(), body: body.into() }
}

// ADR 0008: the selection of the notes list stays in the list.
#[test]
fn the_selected_note_stays_in_the_list() {
    let mut l = Lists::default();
    l.note_down(0);
    assert_eq!(l.note(), 0, "an empty list");
    l.note_down(3);
    l.note_down(3);
    l.note_down(3);
    assert_eq!(l.note(), 2);
    l.note_up();
    l.note_bottom(5);
    assert_eq!(l.note(), 4);
    l.keep_note_in(2);
    assert_eq!(l.note(), 1, "a note is gone");
    l.select_note(0);
    l.note_up();
    assert_eq!(l.note(), 0);
}

// ADR 0008: a search selects its first match; `?` asks the agent and matches nothing.
#[test]
fn a_search_selects_its_first_match() {
    let notes = [note("a.md", "# A\ncoffee here\n"), note("b.md", "# B\nno match\n"), note("c.md", "# C\nmore coffee\n")];
    let mut l = Lists::default();
    l.edit_query(|q| q.push_str("coffee"));
    l.search(&notes);
    assert_eq!((l.query(), l.results().len(), l.result()), ("coffee", 2, 0));
    let first = l.picked().map(|g| g.note);
    l.result_down();
    l.result_down();
    assert_eq!(l.result(), 1, "the last match is the limit");
    assert_ne!(l.picked().map(|g| g.note), first);
    l.result_up();
    assert_eq!((l.result(), l.picked().map(|g| g.note)), (0, first));
    l.edit_query(|q| *q = "?coffee".into());
    l.search(&notes);
    assert!(l.results().is_empty() && l.picked().is_none());
    l.result_down();
    assert_eq!(l.result(), 0);
}
