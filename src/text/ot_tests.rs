use super::*;

fn ins(at: usize, s: &str, len: usize) -> Op {
    Op::diff(&"x".repeat(len), &format!("{}{s}{}", "x".repeat(at), "x".repeat(len - at)))
}

// UNDO-2: an operation from a diff makes the new text from the old one, and its inverse the old one.
#[test]
fn a_diff_applies_and_its_inverse_undoes_it() {
    for (a, b) in [("alpha", "alpha beta"), ("alpha beta", "alpha"), ("héllo wörld", "hello world"), ("", "new"), ("same", "same")] {
        let op = Op::diff(a, b);
        assert_eq!(op.apply(a).as_deref(), Some(b), "{a} -> {b}");
        assert_eq!(op.invert().apply(b).as_deref(), Some(a), "{b} -> {a}");
    }
    assert!(Op::diff("same", "same").is_noop());
}

// UNDO-11: an operation does not apply to a text that it does not fit.
#[test]
fn an_operation_does_not_apply_to_another_text() {
    let op = Op::diff("alpha", "alpha beta");
    assert_eq!(op.apply("alp"), None, "too short");
    assert_eq!(Op::diff("alpha", "al").apply("alXXa"), None, "the deleted text differs");
}

// UNDO-6: two changes at other places both stay, in either order.
#[test]
fn transform_keeps_both_changes() {
    let base = "one two three";
    let pairs = [("one 2 two three", "one two three!"), ("one two three", "ONE two three"), ("e", "one tw"), ("one X two three", "one Y two three"), ("one", "three")];
    for (x, y) in pairs {
        let (a, b) = (Op::diff(base, x), Op::diff(base, y));
        let (a2, b2) = Op::transform(&a, &b);
        let ab = b2.apply(&a.apply(base).unwrap()).unwrap();
        let ba = a2.apply(&b.apply(base).unwrap()).unwrap();
        assert_eq!(ab, ba, "{x} and {y}");
    }
}

// UNDO-6: an insert of the first operation at the same place goes first.
#[test]
fn transform_puts_the_first_insert_first() {
    let (a, b) = (ins(2, "A", 4), ins(2, "B", 4));
    let (a2, _) = Op::transform(&a, &b);
    assert_eq!(a2.apply(&b.apply("xxxx").unwrap()).as_deref(), Some("xxABxx"));
}

// UNDO-8: the end of the change is where the cursor goes after the operation.
#[test]
fn the_end_of_a_change() {
    assert_eq!(Op::diff("alpha", "alpha beta").end(), 10);
    assert_eq!(Op::diff("alpha beta", "alpha").end(), 5);
    assert_eq!(Op::diff("ab", "aXYb").end(), 3);
}

// UNDO-4: an operation goes to JSON and comes back.
#[test]
fn an_operation_goes_to_json_and_back() {
    let op = Op::diff("alpha beta", "alpha gamma");
    let json = serde_json::to_string(&op).unwrap();
    assert_eq!(serde_json::from_str::<Op>(&json).unwrap(), op);
}

// UNDO-13: the cursor moves with the change of another writer.
#[test]
fn a_position_moves_over_a_change() {
    let op = Op::diff("abcd", "abXcd");
    assert_eq!((op.map(1), op.map(3)), (1, 4));
    assert_eq!(Op::diff("abcdef", "af").map(3), 1, "in a deleted range: its start");
    assert_eq!(Op::diff("abcdef", "af").map(6), 2);
}
