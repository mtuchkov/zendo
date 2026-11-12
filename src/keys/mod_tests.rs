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
    assert_eq!(k.action(Pane::Read, &ev(KeyCode::Char('s'), KeyModifiers::CONTROL)), Some("add_to_chat"));
    // ctrl-b is bold in write; ctrl-shift-b is the tree in both modes; ctrl-b also in read.
    assert_eq!(k.action(Pane::Write, &ev(KeyCode::Char('b'), KeyModifiers::CONTROL)), Some("bold"));
    assert_eq!(k.action(Pane::Read, &ev(KeyCode::Char('b'), KeyModifiers::CONTROL)), Some("tree"));
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
    let (k, problems) = Keymap::new(&cfg);
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
        assert_eq!(k.action(pane, &ev(KeyCode::Char('A'), cs)), Some("ask"), "{pane:?}");
        assert_eq!(k.action(pane, &ev(KeyCode::Char('C'), cs)), Some("chat"), "{pane:?}");
        assert_eq!(k.action(pane, &ev(KeyCode::Char('='), cs)), Some("add_to_chat"), "{pane:?}");
        assert_eq!(k.action(pane, &ev(KeyCode::Char('+'), cs)), Some("add_to_chat"), "{pane:?}");
    }
    assert_eq!(k.action(Pane::Write, &ev(KeyCode::Left, KeyModifiers::ALT | KeyModifiers::SHIFT)), Some("select_word_left"));
    assert_eq!(k.first(Pane::Read, "add_to_chat"), "^⇧+");
}
