//! The change of a key in place (docs/zendo/panes/key-change.md): a click on a key of the status
//! bar or of the help makes it a button, a click on the button opens the key float, and the next
//! key goes to the keymap file.

use crate::common;
use common::rekey::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use zendo::keys::{Key, Pane};

fn key(s: &str) -> Key {
    Key::parse(s).unwrap()
}

/// REKEY-1, REKEY-30: a click makes a button, the focus moves it back, a click on the button opens
/// the float.
#[test]
fn a_click_makes_a_button_and_a_second_click_opens_the_float() {
    let (_root, mut a) = app_with("rekey-open");
    let buf = frame(&mut a);
    let at = find(&buf, "alt-b", Some(status_row(&buf))).expect("alt-b in the status bar");
    click(&mut a, at);
    frame(&mut a);
    assert!(a.rekey.button().is_some(), "the key is a button");
    assert!(a.rekey.float().is_none());
    press(&mut a, KeyCode::F(1), KeyModifiers::NONE);
    frame(&mut a);
    assert!(a.rekey.button().is_none(), "the help moved the focus: the button is text again");
    press(&mut a, KeyCode::Esc, KeyModifiers::NONE);
    let buf = open_float(&mut a);
    let f = a.rekey.float().expect("the float");
    assert_eq!((f.target.pane, f.target.action, f.target.key), (Pane::Global, "toggle_tree", key("alt-b")));
    let s = screen(&buf);
    assert!(s.contains("toggle_tree") && s.contains("ctrl-shift-b") || s.contains("^⇧b"), "{s}");
    // REKEY-26: the float says which key the hints show.
    assert!(s.contains("the hints and the help show the first key"), "{s}");
}

/// REKEY-2, REKEY-16: a free key replaces the old one in the keymap file, and the hints show it.
#[test]
fn a_free_key_goes_to_the_keymap_file_and_the_hints() {
    let (root, mut a) = app_with("rekey-free");
    open_float(&mut a);
    press(&mut a, KeyCode::Char('t'), KeyModifiers::ALT);
    assert!(a.rekey.float().is_none(), "the float closes");
    let text = std::fs::read_to_string(root.join("keymap.toml")).unwrap();
    assert!(text.contains("toggle_tree = [\"alt-t\", \"ctrl-shift-b\"]  # file tree"), "{text}");
    assert!(text.starts_with("# my keys\n"), "{text}");
    assert_eq!(a.keys.first(Pane::Global, "toggle_tree"), "alt-t");
    let buf = frame(&mut a);
    assert!(find(&buf, "alt-t", Some(status_row(&buf))).is_some(), "{}", screen(&buf));
    assert!(a.flash.as_ref().is_some_and(|(m, _)| m.contains("alt-b → alt-t")), "{:?}", a.flash);
}

/// REKEY-3, REKEY-11: a key of another action changes nothing; the float says why and suggests
/// free keys at once.
#[test]
fn a_clash_keeps_the_old_key() {
    let (root, mut a) = app_with("rekey-clash");
    let before = std::fs::read_to_string(root.join("keymap.toml")).unwrap();
    open_float(&mut a);
    press(&mut a, KeyCode::Tab, KeyModifiers::NONE);
    let f = a.rekey.float().expect("the float stays");
    assert!(f.message().is_some_and(|m| m.contains("next_pane")), "{:?}", f.message());
    assert!(!f.suggestions().is_empty(), "the rules suggest keys");
    assert_eq!(std::fs::read_to_string(root.join("keymap.toml")).unwrap(), before);
    let s = screen(&frame(&mut a));
    assert!(s.contains("next_pane"), "{s}");
}

/// REKEY-12: a suggested key is a key to press.
#[test]
fn a_suggested_key_is_taken() {
    let (_root, mut a) = app_with("rekey-suggest");
    open_float(&mut a);
    press(&mut a, KeyCode::Tab, KeyModifiers::NONE);
    let k = a.rekey.float().unwrap().suggestions()[0];
    a.on_key(KeyEvent::new(k.code, mods(k)));
    assert!(a.rekey.float().is_none());
    assert_eq!(a.keys.keys_of(Pane::Global, "toggle_tree")[0], k);
}


/// REKEY-14, REKEY-28: `esc` closes the float and changes nothing; `esc` is never a key to click.
#[test]
fn esc_does_not_change() {
    let (root, mut a) = app_with("rekey-esc");
    let before = std::fs::read_to_string(root.join("keymap.toml")).unwrap();
    open_float(&mut a);
    press(&mut a, KeyCode::Esc, KeyModifiers::NONE);
    assert!(a.rekey.float().is_none());
    assert_eq!(std::fs::read_to_string(root.join("keymap.toml")).unwrap(), before);
    a.mode = zendo::app::Mode::Theme;
    frame(&mut a);
    assert!(a.frame.key_targets().iter().all(|(_, t)| t.key != key("esc")), "no target for esc");
}

/// REKEY-25: `[add]` puts the new key first and keeps the others.
#[test]
fn add_puts_the_new_key_first() {
    let (_root, mut a) = app_with("rekey-add");
    let buf = open_float(&mut a);
    let at = find(&buf, "[add]", None).expect("[add]");
    click(&mut a, at);
    press(&mut a, KeyCode::Char('t'), KeyModifiers::ALT);
    assert_eq!(a.keys.keys_of(Pane::Global, "toggle_tree"), vec![key("alt-t"), key("alt-b"), key("ctrl-shift-b")]);
}

/// REKEY-29, REKEY-31: `[remove]` removes the key; the last key of a frequent action stays.
#[test]
fn remove_takes_the_key_away_but_not_the_last_of_a_frequent_action() {
    let (_root, mut a) = app_with("rekey-remove");
    let buf = open_float(&mut a);
    click(&mut a, find(&buf, "[remove]", None).expect("[remove]"));
    assert_eq!(a.keys.keys_of(Pane::Global, "toggle_tree"), vec![key("ctrl-shift-b")]);
    a.flash = None;
    let buf = frame(&mut a);
    let at = find(&buf, "^⇧b", Some(status_row(&buf))).unwrap_or_else(|| panic!("{}", screen(&buf)));
    click(&mut a, at);
    frame(&mut a);
    click(&mut a, at);
    let buf = frame(&mut a);
    click(&mut a, find(&buf, "[remove]", None).expect("[remove]"));
    assert_eq!(a.keys.keys_of(Pane::Global, "toggle_tree"), vec![key("ctrl-shift-b")], "the last key stays");
    assert!(a.rekey.float().unwrap().message().is_some_and(|m| m.contains("last key")));
}

/// REKEY-1: a key in the help opens the float as well.
#[test]
fn a_key_in_the_help_opens_the_float() {
    let (_root, mut a) = app_with("rekey-help");
    press(&mut a, KeyCode::F(1), KeyModifiers::NONE);
    let buf = frame(&mut a);
    let row = find(&buf, "file tree", None).expect("the row of the tree in the help").1;
    let at = find(&buf, "alt-b", Some(row)).expect("alt-b in the help");
    click(&mut a, at);
    frame(&mut a);
    click(&mut a, at);
    frame(&mut a);
    let f = a.rekey.float().expect("the float");
    assert_eq!((f.target.pane, f.target.action), (Pane::Global, "toggle_tree"));
}
