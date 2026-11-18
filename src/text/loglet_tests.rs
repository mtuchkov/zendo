use super::*;
use crate::text::ot::Op;

/// A loglet and its text, changed by a list of texts one after the other.
fn edited(texts: &[&str]) -> (Loglet, String) {
    let mut log = Loglet::default();
    for w in texts.windows(2) {
        log.record(Op::diff(w[0], w[1]));
    }
    (log, texts.last().unwrap().to_string())
}

fn undo(log: &mut Loglet, text: &str) -> String {
    log.undo().unwrap().apply(text).unwrap()
}

fn redo(log: &mut Loglet, text: &str) -> String {
    log.redo().unwrap().apply(text).unwrap()
}

// UNDO-2: undo moves the head back, redo moves it forward.
#[test]
fn undo_and_redo_move_the_head() {
    let (mut log, t) = edited(&["", "a", "a b", "a b c"]);
    let t = undo(&mut log, &t);
    assert_eq!(t, "a b");
    let t = undo(&mut log, &t);
    assert_eq!(t, "a");
    assert_eq!(log.head(), 1);
    let t = redo(&mut log, &t);
    assert_eq!(t, "a b");
    assert_eq!(redo(&mut log, &t), "a b c");
}

// UNDO-9: nothing to undo or to redo gives no operation.
#[test]
fn nothing_to_undo_or_redo() {
    let (mut log, _) = edited(&["", "a"]);
    assert!(log.redo().is_none());
    assert!(log.undo().is_some());
    assert!(log.undo().is_none());
}

// UNDO-3: a new operation after an undo trims the tail.
#[test]
fn a_new_operation_trims_the_tail() {
    let (mut log, t) = edited(&["", "a", "a b"]);
    let t = undo(&mut log, &t);
    log.record(Op::diff(&t, "a x"));
    assert_eq!(log.len(), 2);
    assert!(log.redo().is_none(), "the undone operation is gone");
}

// UNDO-2: a loglet keeps a limited number of operations; the oldest go.
#[test]
fn a_loglet_drops_the_oldest_operations() {
    let mut log = Loglet::default();
    let mut t = String::new();
    for _ in 0..MAX_OPS + 5 {
        let next = format!("{t}x");
        log.record(Op::diff(&t, &next));
        t = next;
    }
    assert_eq!(log.len(), MAX_OPS);
    assert_eq!(log.head(), MAX_OPS);
}

// UNDO-7: the last operation can take the place of a larger one.
#[test]
fn the_last_operation_merges() {
    let (mut log, _) = edited(&["", "a"]);
    log.replace_last(Op::diff("", "ab"));
    assert_eq!(log.len(), 1);
    assert_eq!(undo(&mut log, "ab"), "");
}

// UNDO-4: a loglet goes to JSON and comes back.
#[test]
fn a_loglet_goes_to_json_and_back() {
    let (mut log, t) = edited(&["", "a", "a b"]);
    let t = undo(&mut log, &t);
    let back: Loglet = serde_json::from_str(&serde_json::to_string(&log).unwrap()).unwrap();
    assert_eq!(back, log);
    assert_eq!(redo(&mut back.clone(), &t), "a b");
}

// UNDO-6: a change of another writer is one operation of the loglet, marked as external.
#[test]
fn an_external_change_is_an_operation() {
    let (mut log, t) = edited(&["note", "note mine"]);
    assert!(!log.external_next());
    let theirs = "theirs note mine";
    log.record_external(Op::diff(&t, theirs));
    assert!(log.external_next());
    assert_eq!(log.len(), 2);
    let t = undo(&mut log, theirs);
    assert_eq!(t, "note mine");
    assert!(!log.external_next(), "the change of the user is next");
    assert_eq!(redo(&mut log, &t), theirs);
}

// UNDO-3: an external change after an undo trims the tail too.
#[test]
fn an_external_change_trims_the_tail() {
    let (mut log, t) = edited(&["", "a", "a b"]);
    let t = undo(&mut log, &t);
    log.record_external(Op::diff(&t, "x a"));
    assert!(log.redo().is_none());
}

// UNDO-14: keep the external change: it leaves the loglet, and the operations before it move over
// it, so that the next undo keeps it.
#[test]
fn keep_an_external_change() {
    let (mut log, t) = edited(&["note", "note mine"]);
    let theirs = format!("theirs {t}");
    log.record_external(Op::diff(&t, &theirs));
    log.keep_external();
    assert_eq!((log.len(), log.head(), log.external_next()), (1, 1, false));
    assert_eq!(undo(&mut log, &theirs), "theirs note");
}
