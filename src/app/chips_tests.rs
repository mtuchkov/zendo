use super::*;

// CLAUDE-20, CLAUDE-35: a text chip says the first 15 chars of the text and, always, its lines.
#[test]
fn a_text_chip_says_its_first_chars_and_its_lines() {
    assert_eq!(text_label("IF a legal hold covers an event"), "IF a legal hold… · 1 line");
    assert_eq!(text_label("short"), "short · 1 line");
    assert_eq!(text_label("IF a legal\nhold covers\nan event"), "IF a legal hold… · 3 lines");
    // Blank lines at the ends do not count.
    assert_eq!(text_label("\n  fn main() {}\n\n"), "fn main() {} · 1 line");
}

// CLAUDE-36: a paste of more than one line, or of a long line, is a chip; a short line is text.
#[test]
fn a_long_paste_is_a_chip_and_a_short_one_is_text() {
    assert!(paste_is_chip("a\nb"));
    assert!(paste_is_chip(&"x".repeat(PASTE_CHIP_CHARS + 1)));
    assert!(!paste_is_chip("hello world"));
    assert!(!paste_is_chip("hello\n"), "one line and a new line at the end");
}

// CLAUDE-21: a file chip says the last 4 chars of its folder and the first 10 chars of its name.
#[test]
fn a_file_chip_says_its_folder_tail_and_name_head() {
    assert_eq!(file_label(Path::new("/r/docs/guides/templates/specification.md")), "ates/specificat");
    assert_eq!(file_label(Path::new("/r/a/x.png")), "a/x.png");
}

#[test]
fn chips_are_found_by_their_marks() {
    assert_eq!(chip_ranges("⟦ab⟧ x ⟦c⟧"), vec![(0, 4), (7, 10)]);
    assert!(chip_ranges("[selection] plain").is_empty());
}
