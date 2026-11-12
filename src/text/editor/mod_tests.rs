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
