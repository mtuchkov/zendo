use super::*;
use crate::text::loglet::Loglet;
use crate::text::ot::Op;

/// Type each char and commit after it, as the app does after each key.
fn type_keys(e: &mut Editor, s: &str) {
    for c in s.chars() {
        if c == '\n' { e.enter(true) } else { e.type_char(c) }
        e.commit();
    }
}

fn backspaces(e: &mut Editor, n: usize) {
    for _ in 0..n {
        e.backspace();
        e.commit();
    }
}

// UNDO-1, UNDO-2: undo takes the last change back, redo does it again.
#[test]
fn undo_and_redo_the_last_change() {
    let mut e = Editor::new("alpha");
    e.move_to_end();
    type_keys(&mut e, " beta");
    assert!(e.undo());
    assert_eq!(e.text(), "alpha");
    assert!(e.redo());
    assert_eq!(e.text(), "alpha beta");
    assert!(!e.redo(), "UNDO-9: nothing more to redo");
}

// UNDO-7: a typed word with the space after it is one operation; the next word is another.
#[test]
fn a_word_is_one_operation() {
    let mut e = Editor::new("");
    type_keys(&mut e, "one two three");
    e.undo();
    assert_eq!(e.text(), "one two ");
    e.undo();
    assert_eq!(e.text(), "one ");
    e.undo();
    assert_eq!(e.text(), "");
    assert!(!e.undo(), "UNDO-9: nothing more to undo");
}

// UNDO-7: enter is an operation of its own; a run of backspace is one.
#[test]
fn enter_and_a_run_of_backspace() {
    let mut e = Editor::new("");
    type_keys(&mut e, "ab\ncd");
    backspaces(&mut e, 3);
    assert_eq!(e.text(), "ab");
    e.undo();
    assert_eq!(e.text(), "ab\ncd");
    e.undo();
    assert_eq!(e.text(), "ab\n");
    e.undo();
    assert_eq!(e.text(), "ab");
}

// UNDO-7: a move of the cursor ends the group.
#[test]
fn a_move_ends_the_group() {
    let mut e = Editor::new("");
    type_keys(&mut e, "ab");
    e.left();
    e.commit();
    e.right();
    e.commit();
    type_keys(&mut e, "c");
    e.undo();
    assert_eq!(e.text(), "ab");
}

// UNDO-3: a new change after undo trims the tail: redo has nothing to do.
#[test]
fn a_new_change_trims_the_tail() {
    let mut e = Editor::new("");
    type_keys(&mut e, "one two");
    e.undo();
    type_keys(&mut e, "x");
    assert!(!e.redo());
    assert_eq!(e.text(), "one x");
}

// UNDO-8: undo puts the cursor at the end of the change and clears the selection.
#[test]
fn undo_moves_the_cursor_and_clears_the_selection() {
    let mut e = Editor::new("first\nsecond");
    e.move_to_end();
    type_keys(&mut e, " line");
    e.row = 0;
    e.col = 2;
    e.anchor = Some((0, 0));
    e.undo();
    assert_eq!((e.row, e.col, e.anchor), (1, 6, None));
}

// UNDO-2: a change that commits late is one more operation: a paste with no commit after it.
#[test]
fn a_change_without_a_commit_goes_in_before_undo() {
    let mut e = Editor::new("a");
    e.move_to_end();
    e.type_str(" pasted");
    e.undo();
    assert_eq!(e.text(), "a");
}

// UNDO-6, UNDO-13: a change of another writer is one external operation, merged with the text
// that is not saved yet; the cursor moves with it.
#[test]
fn an_external_change_merges_with_the_unsaved_text() {
    let mut e = Editor::new("note");
    e.move_to_end();
    type_keys(&mut e, " mine");
    assert!(e.external("theirs note"));
    assert_eq!(e.text(), "theirs note mine");
    assert_eq!(e.col, 16, "the cursor stays after the typed text");
    assert!(e.external_next());
    e.undo();
    assert_eq!(e.text(), "note mine", "undo of the external change");
    e.undo();
    assert_eq!(e.text(), "note");
}

// UNDO-6: the file as zendo saved it is no external change.
#[test]
fn the_saved_text_is_no_external_change() {
    let mut e = Editor::new("note");
    e.move_to_end();
    type_keys(&mut e, " mine  ");
    e.saved_as("note mine");
    assert!(!e.external("note mine"));
    assert!(!e.external_next());
}

// UNDO-14: keep the external change: the next undo takes back the change of the user before it.
#[test]
fn keep_the_external_change() {
    let mut e = Editor::new("note");
    e.move_to_end();
    type_keys(&mut e, " mine");
    e.saved_as("note mine");
    e.external("theirs note mine");
    e.keep_external();
    assert!(!e.external_next());
    e.undo();
    assert_eq!(e.text(), "theirs note");
}

// UNDO-4, UNDO-6: a stored loglet resumes on a text that changed while the document was closed:
// the change is one external operation; the trim of the save is none.
#[test]
fn resume_on_a_changed_text() {
    let mut e = Editor::new("note");
    e.move_to_end();
    type_keys(&mut e, " mine  ");
    e.saved_as("note mine");
    let (log, head, disk) = e.history();
    let mut same = Editor::new("note mine");
    same.resume(log.clone(), &head, &disk);
    assert!(!same.external_next(), "only the trim of the save");
    let mut back = Editor::new("theirs note mine");
    back.resume(log, &head, &disk);
    assert!(back.external_next());
    back.undo();
    assert_eq!(back.text(), "note mine");
    back.undo();
    assert_eq!(back.text(), "note");
}

// UNDO-11: a loglet that does not fit the text goes; the text stays.
#[test]
fn a_loglet_that_does_not_fit_goes() {
    let mut log = Loglet::default();
    log.record(Op::diff("something else", "something else entirely"));
    let mut e = Editor::new("note");
    e.resume(log, "note", "note");
    assert!(!e.undo());
    assert_eq!(e.text(), "note");
    assert!(e.history().0.is_empty());
}

// UNDO-5: a taken text starts a new loglet; a cleared text can come back.
#[test]
fn take_starts_a_new_loglet_and_clear_keeps_it() {
    let mut e = Editor::new("");
    type_keys(&mut e, "question");
    e.take();
    assert!(!e.undo());
    type_keys(&mut e, "draft");
    e.clear_text();
    e.undo();
    assert_eq!(e.text(), "draft");
}

