use super::*;

// CLAUDE-37: the name is the last line with letters, in lowercase words, not longer than 24 chars.
#[test]
fn the_name_of_an_image_is_short_lowercase_words() {
    assert_eq!(image_name("Login Form").as_deref(), Some("login form"));
    assert_eq!(image_name("Here it is:\n`error dialog`.\n").as_deref(), Some("error dialog"));
    assert_eq!(image_name("terminal-window with very long words").as_deref(), Some("terminal-window with"));
    assert_eq!(image_name("abcdefghijklmnopqrstuvwxyz0").as_deref(), Some("abcdefghijklmnopqrstuvwx"));
    assert_eq!(image_name("\n...\n"), None);
}
