use super::*;

// CLAUDE-20: a selection chip says the first 15 chars of the selection.
#[test]
fn a_selection_chip_says_its_first_chars() {
    assert_eq!(selection_label("IF a legal\n hold covers an event"), "IF a legal hold…");
    assert_eq!(selection_label("short"), "short");
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
