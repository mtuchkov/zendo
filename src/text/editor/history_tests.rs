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

/// A document editor: its loglet takes only what zendo saved (UNDO-16).
fn document(text: &str) -> Editor {
    let mut e = Editor::new(text);
    e.as_document(text);
    e.move_to_end();
    e
}

// UNDO-16: a document records only the saved text, one operation for each save.
#[test]
fn a_document_records_what_it_saved() {
    let mut e = document("note");
    type_keys(&mut e, " one two");
    assert!(!e.undo(), "nothing saved yet");
    assert_eq!(e.text(), "note one two");
    e.flush("note one two");
    type_keys(&mut e, " three");
    e.flush("note one two three");
    e.undo();
    assert_eq!(e.text(), "note one two");
    e.undo();
    assert_eq!(e.text(), "note");
    e.redo();
    assert_eq!(e.text(), "note one two");
}

// UNDO-6, UNDO-13: a change of another writer is appended to the loglet as one external
// operation; the text that is not saved stays out of the loglet and moves over the change.
#[test]
fn an_external_change_is_appended() {
    let mut e = document("note");
    type_keys(&mut e, " mine");
    e.flush("note mine");
    type_keys(&mut e, " unsaved");
    assert!(e.external("theirs note mine"));
    assert_eq!(e.text(), "theirs note mine unsaved");
    assert_eq!(e.col, 24, "the cursor stays after the typed text");
    assert!(e.external_next());
    e.undo();
    assert_eq!(e.text(), "note mine", "the unsaved text was never in the loglet");
    e.undo();
    assert_eq!(e.text(), "note");
}

// UNDO-6: the file of the last save is no external change.
#[test]
fn the_saved_file_is_no_external_change() {
    let mut e = document("note");
    type_keys(&mut e, " mine  ");
    e.flush("note mine");
    assert!(!e.external("note mine"));
    assert!(!e.external_next());
}

// UNDO-14: keep the external change: the next undo takes back the change of the user before it.
#[test]
fn keep_the_external_change() {
    let mut e = document("note");
    type_keys(&mut e, " mine");
    e.flush("note mine");
    e.external("theirs note mine");
    e.keep_external();
    assert!(!e.external_next());
    e.undo();
    assert_eq!(e.text(), "theirs note");
}

// UNDO-4, UNDO-6: a stored loglet resumes on the file; a change since the last save is one
// external operation.
#[test]
fn resume_on_a_changed_file() {
    let mut e = document("note");
    type_keys(&mut e, " mine");
    e.flush("note mine");
    let (log, head) = e.history();
    let mut same = Editor::new("note mine");
    same.resume(log.clone(), &head);
    assert!(!same.external_next());
    let mut back = Editor::new("theirs note mine");
    back.resume(log, &head);
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
    e.resume(log, "note");
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


// UNDO-16: a new note has no file: undo of its first save empties it, and redo brings it back.
#[test]
fn undo_of_the_first_save_of_a_new_note() {
    let mut e = Editor::new("# \n\n");
    e.as_document("");
    e.move_to_end();
    type_keys(&mut e, "body");
    e.flush("# \n\nbody".trim_end());
    e.undo();
    assert_eq!(e.text(), "");
    e.flush("");
    assert!(e.redo(), "the empty file is no new change");
    assert_eq!(e.text(), "# \n\nbody");
}
