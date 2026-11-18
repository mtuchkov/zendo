use super::*;

fn key(s: &str) -> Key {
    Key::parse(s).expect("a key")
}

/// WIZARD-10, REKEY-3: a key of another action of the pane clashes.
#[test]
fn a_key_of_another_action_of_the_pane_clashes() {
    let map = Keymap::defaults();
    let owner = map.owner(Pane::Read, key("j")).expect("j has an action in read");
    assert_eq!(clash(&map, Pane::Read, "ask", key("j")), Some(Clash::Taken(Pane::Read, owner)));
}

/// WIZARD-10, REKEY-3: a global key clashes for an action of a pane, and a key of any pane for a
/// global action.
#[test]
fn global_keys_clash_in_every_pane() {
    let map = Keymap::defaults();
    assert_eq!(clash(&map, Pane::Read, "ask", key("tab")), Some(Clash::Taken(Pane::Global, "next_pane")));
    let k = key("alt-shift-h");
    let owner = map.owner(Pane::Read, k).expect("alt-shift-h has an action in read");
    assert_eq!(clash(&map, Pane::Global, "toggle_tree", k), Some(Clash::Taken(Pane::Read, owner)));
}

/// WIZARD-10: where the user types text, a key without ctrl or alt clashes.
#[test]
fn a_plain_char_clashes_where_the_user_types() {
    let map = Keymap::defaults();
    assert_eq!(clash(&map, Pane::Write, "bold", key("x")), Some(Clash::Text));
    assert_eq!(clash(&map, Pane::Chat, "send", key("X")), Some(Clash::Text));
    assert_eq!(clash(&map, Pane::Global, "toggle_tree", key("x")), Some(Clash::Text));
    assert_eq!(clash(&map, Pane::Write, "bold", key("f11")), None);
}

/// REKEY-28: `esc` is no new key.
#[test]
fn esc_is_never_free() {
    let map = Keymap::defaults();
    assert_eq!(clash(&map, Pane::Read, "ask", key("esc")), Some(Clash::Esc));
}

/// REKEY-2: a free key and the key of the action itself do not clash.
#[test]
fn a_free_key_and_the_own_key_do_not_clash() {
    let map = Keymap::defaults();
    assert_eq!(clash(&map, Pane::Global, "toggle_tree", key("alt-t")), None);
    assert_eq!(clash(&map, Pane::Global, "toggle_tree", key("alt-b")), None);
}

/// REKEY-11, WIZARD-10: the rules suggest only free keys, first with the char of the pressed key.
#[test]
fn the_rules_suggest_free_keys_near_the_pressed_key() {
    let map = Keymap::defaults();
    let near = [key("alt-b"), key("tab")];
    let got = suggest(&map, Pane::Global, "toggle_tree", &near, 3);
    assert_eq!(got.len(), 3);
    assert!(got.iter().all(|k| clash(&map, Pane::Global, "toggle_tree", *k).is_none()), "{got:?}");
    assert_eq!(got[0], key("alt-B"), "the letter of the old key first: {got:?}");
    assert!(got.iter().all(|k| k.code != KeyCode::Tab), "no tab with a modifier: {got:?}");
}

/// REKEY-7: a suggested key goes through the same rules.
#[test]
fn checked_drops_the_keys_that_clash() {
    let map = Keymap::defaults();
    let got = checked(&map, Pane::Write, "bold", &["j", "tab", "alt-y", "nonsense", "alt-y"]);
    assert_eq!(got, vec![key("alt-y")]);
}

/// REKEY-31, PANES-18: the actions of the hints of the status bar are frequently used.
#[test]
fn the_hint_actions_are_frequent() {
    assert!(frequent(Pane::Global, "toggle_tree"));
    assert!(frequent(Pane::Read, "ask"));
    assert!(!frequent(Pane::Read, "peek_disk"));
}

/// REKEY-16: a key goes to the config as text that parses back to the same key.
#[test]
fn a_key_spec_parses_back() {
    for s in ["j", "J", "+", "alt-b", "ctrl-shift-b", "ctrl-b", "shift-down", "alt-shift-left", "f6", "enter", "space", "pageup", "backspace", "alt-]", "ctrl-+"] {
        let k = key(s);
        assert_eq!(Key::parse(&k.spec()), Ok(k), "{s} → {}", k.spec());
    }
}

/// REKEY-29: an empty list in a layer leaves the action without a key.
#[test]
fn an_empty_list_removes_the_keys() {
    let cfg: KeyConfig = toml::from_str("[read]\nask = []\n").expect("toml");
    let (map, problems) = Keymap::new(None, &cfg);
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(map.key(Pane::Read, "ask"), None);
}
