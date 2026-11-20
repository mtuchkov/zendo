//! `/new` (docs/zendo/chat-history.md).
use super::*;

// HISTORY-8, HISTORY-9: `/new` is the command only as the first word.
#[test]
fn new_is_a_command_only_as_the_first_word() {
    assert_eq!(strip_new("/new turn the note"), Some("turn the note"));
    assert_eq!(strip_new("/new"), Some(""));
    assert_eq!(strip_new("/new\nline two"), Some("line two"));
    assert_eq!(strip_new("/newer"), None);
    assert_eq!(strip_new("say /new"), None);
}
