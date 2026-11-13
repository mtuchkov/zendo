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
    assert_eq!(k.action(Pane::Tree, &ev(KeyCode::Char('q'), KeyModifiers::CONTROL)), Some("quit"));
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

// PANES-32: ctrl-c is copy, not quit: no pane quits on ctrl-c.
#[test]
fn ctrl_c_does_not_quit() {
    let k = Keymap::defaults();
    for pane in Pane::ALL {
        assert_ne!(k.action(pane, &ev(KeyCode::Char('c'), KeyModifiers::CONTROL)), Some("quit"), "{pane:?}");
    }
    assert!(!k.label(Pane::Global, "quit").contains("^c"), "{}", k.label(Pane::Global, "quit"));
}

// PANES-30: the keymap file shows the actions of each pane in groups, in the order of the help: a
// `# <group>` line, then the actions of that group.
#[test]
fn the_keymap_file_groups_the_actions_as_the_help_does() {
    let mut pane = "";
    let mut group: Option<Group> = None;
    let mut in_file = Vec::new();
    for line in DEFAULT_KEYMAP.lines().map(str::trim) {
        if let Some(p) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            (pane, group) = (p, None);
        } else if let Some(g) = line.strip_prefix("# ").and_then(|l| Group::ALL.into_iter().find(|g| g.name() == l)) {
            group = Some(g);
        } else if let Some((name, _)) = line.split_once(" = ").filter(|_| !line.starts_with('#')) {
            in_file.push((pane.to_string(), group.map(Group::name), name.to_string()));
        }
    }
    let actions: Vec<_> = ACTIONS.iter().map(|a| (a.pane.name().to_string(), Some(a.group.name()), a.name.to_string())).collect();
    assert_eq!(in_file, actions);
}

// PANES-30: the actions of one group of a pane come one after the other, so that each group has one heading.
#[test]
fn each_group_of_a_pane_has_one_place() {
    for pane in Pane::ALL {
        let mut groups: Vec<Group> = ACTIONS.iter().filter(|a| a.pane == pane).map(|a| a.group).collect();
        groups.dedup();
        let mut once = groups.clone();
        once.sort();
        once.dedup();
        assert_eq!(groups.len(), once.len(), "{pane:?}: {groups:?}");
    }
}
