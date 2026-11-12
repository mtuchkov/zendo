use super::*;

fn at(text: &str, row: usize, col: usize) -> Editor {
    let mut e = Editor::new(text);
    e.row = row;
    e.col = col;
    e
}

#[test]
fn select_copy_cut_and_type_over() {
    let mut e = at("one two three", 0, 4);
    e.select_with(|e| e.word_right());
    assert_eq!(e.selected_text().as_deref(), Some("two "));
    e.type_char('2');
    assert_eq!(e.text(), "one 2three");
    let mut e = at("ab\ncd\nef", 0, 1);
    e.select_with(|e| e.vertical(2, 80));
    assert_eq!(e.selected_text().as_deref(), Some("b\ncd\ne"));
    assert!(e.delete_selection());
    assert_eq!(e.text(), "af");
    let mut e = at("x", 0, 0);
    e.type_str("pasted\ntext ");
    assert_eq!(e.text(), "pasted\ntext x");
}

#[test]
fn a_delete_key_deletes_the_selection_or_else_does_its_own_delete() {
    let mut e = at("one two", 0, 4);
    e.select_with(|e| e.word_right());
    e.delete_selection_or(Editor::backspace);
    assert_eq!(e.text(), "one ");
    e.delete_selection_or(Editor::backspace);
    assert_eq!(e.text(), "one");
}

#[test]
fn bold_italic_underline_toggle() {
    let mut e = at("make this bold", 0, 6);
    e.wrap_marks("**", "**");
    assert_eq!(e.text(), "make **this** bold");
    e.col = 8;
    e.wrap_marks("**", "**");
    assert_eq!(e.text(), "make this bold", "bold again takes it away");
    let mut e = at("a word here", 0, 2);
    e.anchor = Some((0, 11));
    e.wrap_marks("<u>", "</u>");
    assert_eq!(e.text(), "a <u>word here</u>");
    let mut e = at("an **x** y", 0, 6);
    e.wrap_marks("*", "*");
    assert_eq!(e.text(), "an ***x*** y", "italic inside bold");
    let mut e = at("an owner.", 0, 8);
    e.anchor = Some((0, 2));
    e.wrap_marks("**", "**");
    assert_eq!(e.text(), "an **owner**.", "the marks touch the word, not the space");
    let mut e = at("end ", 0, 4);
    e.wrap_marks("*", "*");
    assert_eq!((e.text().as_str(), e.col), ("end **", 5), "no word: both marks, the cursor between");
}
