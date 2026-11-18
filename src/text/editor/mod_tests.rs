use super::*;

#[test]
fn type_and_split() {
    let mut e = Editor::new("");
    e.insert_str("hello\nworld");
    assert_eq!(e.text(), "hello\nworld");
    assert_eq!((e.row, e.col), (1, 5));
    e.backspace();
    e.home();
    e.backspace();
    assert_eq!(e.text(), "helloworl");
}

#[test]
fn bullets_continue_and_end() {
    let mut e = Editor::new("- first");
    e.end();
    e.split_line(true);
    assert_eq!(e.text(), "- first\n- ");
    e.split_line(true);
    assert_eq!(e.text(), "- first\n");
}

#[test]
fn wrap_keeps_words() {
    let segs = wrap_line("one two three four", 9);
    let parts: Vec<String> = segs.iter().map(|(a, b)| "one two three four".chars().skip(*a).take(b - a).collect()).collect();
    assert_eq!(parts, vec!["one two ", "three ", "four"]);
}

#[test]
fn vertical_moves_through_wrapped_rows() {
    let mut e = Editor::new("one two three four\nfive");
    e.row = 0;
    e.col = 1;
    e.vertical(1, 9);
    assert_eq!((e.row, e.col), (0, 9));
    e.vertical(2, 9);
    assert_eq!(e.row, 1);
}

#[test]
fn utf8_safe() {
    let mut e = Editor::new("кэш");
    e.end();
    e.backspace();
    e.insert('ш');
    e.word_left();
    e.delete_word_back();
    assert_eq!(e.text(), "кэш");
}

// HISTORY-5: the first and the last visual row, also of a wrapped line: there `up` or `down`
// cannot move, so the chat input goes through the history.
#[test]
fn the_cursor_knows_the_first_and_the_last_visual_row() {
    let mut e = Editor::new("short\na line that wraps at ten");
    assert!(e.on_edge(-1, 10) && !e.on_edge(1, 10));
    e.row = 1;
    e.col = 2;
    assert!(!e.on_edge(-1, 10) && !e.on_edge(1, 10), "the first row of a wrapped line is not the last row");
    e.col = 24;
    assert!(e.on_edge(1, 10));
    assert!(Editor::new("").on_edge(-1, 10) && Editor::new("").on_edge(1, 10));
}
