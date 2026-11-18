//! The lists without an app (ADR 0008).
use super::*;

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
