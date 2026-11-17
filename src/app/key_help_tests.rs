//! The help of the keys without an app (ADR 0008).
use super::*;

// ADR 0008, PANES-31: the help keeps the pane that it shows, the keymap file and its problems.
#[test]
fn the_help_keeps_its_pane_and_the_keymap() {
    let mut h = KeyHelp::new(vec!["unknown action".into()], Some("~/.zendo/keymap.toml".into()));
    assert_eq!(h.pane(), Pane::default());
    h.show(Pane::Write);
    assert_eq!(h.pane(), Pane::Write);
    assert_eq!(h.problems(), ["unknown action".to_string()]);
    assert_eq!(h.file().map(String::as_str), Some("~/.zendo/keymap.toml"));
    assert!(KeyHelp::default().file().is_none());
}
