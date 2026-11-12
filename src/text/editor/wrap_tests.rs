use super::*;

fn typed(text: &str, width: usize) -> Editor {
    let mut e = Editor::new(text);
    e.move_to_end();
    while e.hard_wrap(width) {}
    e
}

// NOTE-29: a line longer than the text column breaks at a word; read mode joins it again.
#[test]
fn a_long_line_breaks_at_a_word() {
    let e = typed("one two three four five six", 14);
    assert_eq!(e.lines, vec!["one two three", "four five six"]);
    assert_eq!((e.row, e.col), (1, 13), "the cursor stays after the last char");
}

#[test]
fn list_items_and_quotes_go_on_under_their_marker() {
    assert_eq!(typed("- an item with words", 12).lines, vec!["- an item", "  with words"]);
    assert_eq!(typed("> a quote with words", 12).lines, vec!["> a quote", "> with words"]);
}

#[test]
fn headings_tables_and_code_keep_their_length() {
    assert_eq!(typed("# a long heading of words", 10).lines.len(), 1);
    assert_eq!(typed("| a | long | table | row |", 10).lines.len(), 1);
    assert_eq!(typed("```\nlet a_long_line = of code;", 10).lines.len(), 2);
}

#[test]
fn a_line_does_not_start_with_a_block_marker() {
    let e = typed("see the list - item", 15);
    assert!(!e.lines[1].starts_with("- "), "{:?}", e.lines);
}
