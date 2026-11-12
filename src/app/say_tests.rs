use super::*;

const DOC: &str = "# T\n\nOne a.\nOne b.\n\nTwo.\n\n\nThree a.\nThree b.";

// NOTE-25: the mark takes the paragraph above and the paragraph below.
#[test]
fn paragraphs_around_a_line() {
    let lines: Vec<&str> = DOC.lines().collect();
    assert_eq!(paragraph(&lines, 3), Some((2, 3)));
    assert_eq!(paragraph(&lines, 4), None);
    assert_eq!(paragraph_before(&lines, 5), Some((2, 3)));
    assert_eq!(paragraph_after(&lines, 5), Some((8, 9)));
    assert_eq!(paragraph_after(&lines, 9), None);
}

// NOTE-25, NOTE-26: the context names the place, quotes the paragraphs and the selection.
#[test]
fn the_context_quotes_the_text_around_the_mark() {
    let lines: Vec<&str> = DOC.lines().collect();
    let c = say_context("a.md", &lines, (2, 2), None);
    assert!(c.contains("after line 4") && c.contains("> One a.") && c.contains("> Two."), "{c}");
    let c = say_context("a.md", &lines, (5, 5), Some("Two."));
    assert!(c.contains("> Two.") && c.contains("> One b.") && c.contains("> Three a.") && c.contains("replaces the selected text"), "{c}");
}

// NOTE-25: say here tells the agent the line and the column of the cursor.
#[test]
fn say_here_names_the_line_and_the_column_of_the_cursor() {
    assert_eq!(cursor_place("a.md", (2, 4)), "The cursor is at line 3, column 5 of a.md.");
}
