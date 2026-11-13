use super::*;

fn ev(code: KeyCode, m: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, m)
}

#[test]
fn keys_parse_and_match_events() {
    let k = Keymap::defaults();
    assert_eq!(k.action(Pane::Read, &ev(KeyCode::Char('J'), KeyModifiers::SHIFT)), Some("select_down"));
    assert_eq!(k.action(Pane::Read, &ev(KeyCode::Char('J'), KeyModifiers::NONE)), Some("select_down"));
    assert_eq!(k.action(Pane::Read, &ev(KeyCode::Down, KeyModifiers::SHIFT)), Some("select_down"));
    assert_eq!(k.action(Pane::Read, &ev(KeyCode::Char('+'), KeyModifiers::SHIFT)), Some("add_to_chat"));
    // ctrl-b is bold in write; ctrl-shift-b is the tree in both modes.
    assert_eq!(k.action(Pane::Write, &ev(KeyCode::Char('b'), KeyModifiers::CONTROL)), Some("bold"));
    let ctrl_shift = KeyModifiers::CONTROL | KeyModifiers::SHIFT;
    assert_eq!(k.action(Pane::Write, &ev(KeyCode::Char('B'), ctrl_shift)), Some("toggle_tree"));
    assert_eq!(k.action(Pane::Read, &ev(KeyCode::Char('b'), ctrl_shift)), Some("toggle_tree"));
    // ctrl-shift-hjkl select in both modes.
    assert_eq!(k.action(Pane::Read, &ev(KeyCode::Char('J'), ctrl_shift)), Some("select_down"));
    assert_eq!(k.action(Pane::Write, &ev(KeyCode::Char('h'), ctrl_shift)), Some("select_left"));
    assert_eq!(k.action(Pane::Write, &ev(KeyCode::Char('c'), KeyModifiers::CONTROL)), Some("copy"));
    assert_eq!(k.action(Pane::Tree, &ev(KeyCode::Char('c'), KeyModifiers::CONTROL)), Some("quit"));
    // ctrl-shift and an arrow selects a char, as ctrl-shift-h j k l; alt-shift and an arrow, a word.
    assert_eq!(k.action(Pane::Write, &ev(KeyCode::Left, KeyModifiers::CONTROL | KeyModifiers::SHIFT)), Some("select_left"));
    assert_eq!(k.action(Pane::Chat, &ev(KeyCode::Enter, KeyModifiers::ALT)), Some("newline"));
    assert_eq!(k.action(Pane::Write, &ev(KeyCode::Char('x'), KeyModifiers::NONE)), None, "a letter in write mode is text");
    for s in ["ctrl-b", "alt-i", "shift-down", "ctrl-shift-left", "f5", "space", "J", "+", "pageup"] {
        assert!(Key::parse(s).is_ok(), "{s}");
    }
    assert_eq!(Key::parse("shift-j").unwrap(), Key::parse("J").unwrap());
    assert!(Key::parse("ctrl-nope").is_err());
}

#[test]
fn the_config_changes_keys_and_reports_problems() {
    let cfg: KeyConfig = toml::from_str(
        r#"
            [read]
            add_to_chat = "m"
            select_down = ["ctrl-j", "nonsense-key"]
            fly = "f"
            [nowhere]
            x = "x"
            "#,
    )
    .unwrap();
    let (k, problems) = Keymap::new(None, &cfg);
    assert_eq!(k.action(Pane::Read, &ev(KeyCode::Char('m'), KeyModifiers::NONE)), Some("add_to_chat"));
    assert_eq!(k.action(Pane::Read, &ev(KeyCode::Char('+'), KeyModifiers::NONE)), None, "the custom keys replace the defaults");
    assert_eq!(k.action(Pane::Read, &ev(KeyCode::Char('j'), KeyModifiers::CONTROL)), Some("select_down"));
    assert_eq!(problems.len(), 3, "{problems:?}");
    assert_eq!(k.label(Pane::Read, "down"), "j ↓");
}

// NOTE-17, NOTE-18, NOTE-25, PANES: the ctrl-shift keys work in read mode and in write mode.
#[test]
fn ctrl_shift_keys_work_in_both_modes() {
    let k = Keymap::defaults();
    let cs = KeyModifiers::CONTROL | KeyModifiers::SHIFT;
    for pane in [Pane::Read, Pane::Write] {
        assert_eq!(k.action(pane, &ev(KeyCode::Left, cs)), Some("select_left"), "{pane:?}");
        assert_eq!(k.action(pane, &ev(KeyCode::Down, cs)), Some("select_down"), "{pane:?}");
        assert_eq!(k.action(pane, &ev(KeyCode::Char('L'), cs)), Some("select_right"), "{pane:?}");
        assert_eq!(k.action(pane, &ev(KeyCode::Char('S'), cs)), Some("say"), "{pane:?}");
        assert_eq!(k.action(pane, &ev(KeyCode::Char('A'), cs)), Some("add_to_chat"), "{pane:?}");
        assert_eq!(k.action(pane, &ev(KeyCode::Char('C'), cs)), Some("chat"), "{pane:?}");
        assert_eq!(k.action(pane, &ev(KeyCode::Char('+'), cs)), None, "the terminal zooms: {pane:?}");
        assert_eq!(k.action(pane, &ev(KeyCode::Char('='), cs)), None, "the terminal zooms: {pane:?}");
    }
    assert_eq!(k.action(Pane::Write, &ev(KeyCode::Left, KeyModifiers::ALT | KeyModifiers::SHIFT)), Some("select_word_left"));
    assert_eq!(k.first(Pane::Read, "add_to_chat"), "alt-a", "the key of every terminal comes first");
    assert_eq!(k.action(Pane::Write, &ev(KeyCode::Char('k'), KeyModifiers::CONTROL)), None, "ctrl-k does not open the chat");
}

// PANES-17: the default keys come from the keymap file that zendo ships: every action is in it, and
// nothing else.
#[test]
fn the_default_keymap_file_names_every_action() {
    let file: KeyConfig = toml::from_str(DEFAULT_KEYMAP).expect("the default keymap parses");
    for a in ACTIONS {
        let keys = file.get(a.pane.name()).and_then(|m| m.get(a.name)).unwrap_or_else(|| panic!("{}.{} is not in the keymap file", a.pane.name(), a.name));
        assert!(!keys.list().is_empty());
    }
    let (_, problems) = Keymap::new(Some(&file), &KeyConfig::new());
    assert!(problems.is_empty(), "{problems:?}");
}

// PANES-23: a keymap file of the user changes the defaults, and `[keys]` of the config changes the file.
#[test]
fn a_user_keymap_file_then_the_config_change_keys() {
    let file: KeyConfig = toml::from_str("[read]\ndown = [\"n\"]\nup = [\"p\"]\n").unwrap();
    let cfg: KeyConfig = toml::from_str("[read]\nup = \"u\"\n").unwrap();
    let (k, problems) = Keymap::new(Some(&file), &cfg);
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(k.action(Pane::Read, &ev(KeyCode::Char('n'), KeyModifiers::NONE)), Some("down"));
    assert_eq!(k.action(Pane::Read, &ev(KeyCode::Char('u'), KeyModifiers::NONE)), Some("up"));
    assert_eq!(k.action(Pane::Read, &ev(KeyCode::Char('p'), KeyModifiers::NONE)), None, "the config wins over the file");
    assert_eq!(k.action(Pane::Read, &ev(KeyCode::Char('l'), KeyModifiers::NONE)), Some("right"), "the rest keeps the defaults");
}

// NOTE-17: alt-shift and h or l (or an arrow) selects a word, in read mode, write mode and the chat.
#[test]
fn alt_shift_hl_select_words_everywhere() {
    let k = Keymap::defaults();
    let a = KeyModifiers::ALT | KeyModifiers::SHIFT;
    for pane in [Pane::Read, Pane::Write, Pane::Chat] {
        assert_eq!(k.action(pane, &ev(KeyCode::Char('H'), a)), Some("select_word_left"), "{pane:?}");
        assert_eq!(k.action(pane, &ev(KeyCode::Char('L'), a)), Some("select_word_right"), "{pane:?}");
        assert_eq!(k.action(pane, &ev(KeyCode::Right, a)), Some("select_word_right"), "{pane:?}");
    }
    for pane in [Pane::Read, Pane::Write] {
        assert_eq!(k.action(pane, &ev(KeyCode::Char('J'), a)), Some("select_down"), "{pane:?}");
        assert_eq!(k.action(pane, &ev(KeyCode::Char('K'), a)), Some("select_up"), "{pane:?}");
    }
}

// PANES-24: two actions of one pane with one key are a problem; the default keymap has none.
#[test]
fn a_key_of_two_actions_is_a_problem() {
    let cfg: KeyConfig = toml::from_str("[read]\ncopy = \"m\"\nask = \"m\"\n").unwrap();
    let (_, problems) = Keymap::new(None, &cfg);
    assert!(problems.iter().any(|p| p.contains("ask") && p.contains("copy") && p.contains("also the key of")), "{problems:?}");
    let file: KeyConfig = toml::from_str("[write]\ncut = [\"ctrl-s\"]\nbold = [\"ctrl-s\"]\n").unwrap();
    let (_, problems) = Keymap::new(Some(&file), &KeyConfig::new());
    assert!(problems.iter().any(|p| p.contains("^s is also the key of")), "{problems:?}");
    assert!(Keymap::new(None, &KeyConfig::new()).1.is_empty(), "the defaults have no conflict");
}

// PANES-25, NOTE-17: without the kitty keyboard protocol ctrl-shift-h j k l come as ctrl-h j k l, and select.
#[test]
fn without_the_kitty_protocol_ctrl_hjkl_select() {
    let mut k = Keymap::defaults();
    k.legacy = true;
    for pane in [Pane::Read, Pane::Write] {
        for (c, act) in [('h', "select_left"), ('j', "select_down"), ('k', "select_up"), ('l', "select_right")] {
            assert_eq!(k.action(pane, &ev(KeyCode::Char(c), KeyModifiers::CONTROL)), Some(act), "{pane:?} ^{c}");
        }
    }
    assert_eq!(k.action(Pane::Write, &ev(KeyCode::Char('b'), KeyModifiers::CONTROL)), Some("bold"), "a key of its own wins");
    assert_eq!(k.action(Pane::Write, &ev(KeyCode::Char('s'), KeyModifiers::CONTROL)), Some("say"), "ctrl-shift-s, as no save key is in the way");
}

fn alt(c: char) -> KeyEvent {
    ev(KeyCode::Char(c), KeyModifiers::ALT)
}

// PANES-28: on macOS the keys of keymap-macos.toml come over the default keys.
#[test]
fn macos_keys_come_over_the_default_keys() {
    let (mac, problems) = Keymap::for_platform(true, None, &KeyConfig::new());
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(mac.action(Pane::Write, &alt('b')), Some("word_left"), "Option-Left comes as alt-b");
    assert_eq!(mac.action(Pane::Chat, &alt('f')), Some("word_right"), "Option-Right comes as alt-f");
    assert_eq!(mac.action(Pane::Read, &alt('b')), None, "Option-Left does not open the tree");
    assert_eq!(mac.action(Pane::Read, &alt('e')), Some("toggle_tree"));
    assert_eq!(mac.action(Pane::Write, &ev(KeyCode::Char('e'), KeyModifiers::CONTROL)), Some("end"));
    assert_eq!(mac.action(Pane::Read, &alt('1')), Some("session_1"));
    assert_eq!(mac.action(Pane::Read, &ev(KeyCode::F(6), KeyModifiers::NONE)), Some("session_1"), "the F-keys stay");
    assert_eq!(mac.action(Pane::Tree, &ev(KeyCode::Backspace, KeyModifiers::NONE)), Some("delete"));
    assert_eq!(mac.action(Pane::Read, &ev(KeyCode::Char('j'), KeyModifiers::NONE)), Some("down"), "the rest keeps the defaults");
    let (other, _) = Keymap::for_platform(false, None, &KeyConfig::new());
    assert_eq!(other.action(Pane::Read, &alt('b')), Some("toggle_tree"));
    assert_eq!(other.action(Pane::Write, &ev(KeyCode::Char('e'), KeyModifiers::CONTROL)), Some("toggle_tree"));
}

// PANES-28: the macOS keymap names only known actions and keys, and gives no key two actions.
#[test]
fn the_macos_keymap_has_one_action_for_each_key() {
    let overlay: KeyConfig = toml::from_str(MACOS_KEYMAP).expect("the macOS keymap parses");
    assert!(Keymap::new(Some(&overlay), &KeyConfig::new()).1.is_empty());
    let merged: KeyConfig = toml::from_str(&platform_keymap(true)).expect("the merged keymap parses");
    let (_, problems) = Keymap::for_platform(false, Some(&merged), &KeyConfig::new());
    assert!(problems.is_empty(), "{problems:?}");
}

// PANES-29: on macOS `zendo keymap` prints one file with the macOS keys, each changed line marked.
#[test]
fn the_platform_keymap_marks_the_macos_keys() {
    assert_eq!(platform_keymap(false), DEFAULT_KEYMAP);
    let mac = platform_keymap(true);
    assert!(mac.contains("toggle_tree = [\"alt-e\", \"ctrl-shift-b\"]  # file tree (macOS)"), "{mac}");
    assert!(mac.contains("down = [\"j\", \"down\"]  # cursor down\n"), "an unchanged line stays");
    assert_eq!(mac.matches("(macOS)").count(), toml::from_str::<KeyConfig>(MACOS_KEYMAP).unwrap().values().map(|a| a.len()).sum::<usize>());
}
