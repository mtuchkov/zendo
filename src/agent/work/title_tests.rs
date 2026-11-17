//! The title of a new note (docs/zendo/float-title.md) without an app.
use super::*;

// FLOAT-19, FLOAT-21: the title of the agent goes in where the note has none, replaces only the
// title of the agent, and leaves the title of the user.
#[test]
fn a_title_goes_in_only_where_the_user_has_none() {
    assert_eq!(with_title("# \n\nGreen tea.", "Tea", None).as_deref(), Some("# Tea\n\nGreen tea."), "the empty title line");
    assert_eq!(with_title("Green tea.\nBlack tea.", "Tea", None).as_deref(), Some("# Tea\n\nGreen tea.\nBlack tea."), "no title line");
    assert_eq!(with_title("# Mine\n\nGreen tea.", "Tea", None), None, "the title of the user");
    assert_eq!(with_title("# Tea\n\nGreen tea.", "Teas", Some("Tea")).as_deref(), Some("# Teas\n\nGreen tea."), "the title of the agent");
    assert_eq!(with_title("# Mine\n\nGreen tea.", "Teas", Some("Tea")), None, "the user changed the title of the agent");
    assert_eq!(with_title("# Tea\n\nGreen tea.", "Tea", Some("Tea")), None, "the same title");
    assert_eq!(with_title("\nGreen tea.", "Tea", Some("Old")).as_deref(), Some("# Tea\n\n\nGreen tea."), "the user took the title away");
}

// FLOAT-18: three lines with text, or 200 chars, are enough for a title; blank lines do not count.
#[test]
fn three_lines_or_200_chars_are_enough_for_a_title() {
    assert!(!ripe("One.\n\nTwo."));
    assert!(!ripe("# \n\nOne.\nTwo."), "the empty title line is no text");
    assert_eq!((title_of("# \n\nOne."), body_of("# \n\nOne.")), (None, "One."));
    assert!(ripe("One.\nTwo.\n\nThree."));
    assert!(ripe(&"word ".repeat(41)));
    assert_eq!(title_of("\n# Tea \n\nbody"), Some("Tea"));
    assert_eq!(body_of("# Tea\n\nGreen.\n"), "Green.");
    assert_eq!(body_of("Green.\nBlack.\n"), "Green.\nBlack.");
}
