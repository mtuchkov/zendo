//! The keys of the search box, the find panel, the file name filter of the tree, the theme list and
//! the question of a file come from the keymap; only `esc` stays fixed (docs/zendo/panes/keymap.md,
//! PANES-48).

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use zendo::app::{App, Mode};
use zendo::keys::{KeyConfig, Keymap};

const KEYS: &str = r#"
[list]
down = ["ctrl-d"]
open = ["ctrl-o"]
[theme]
down = ["ctrl-d"]
keep = ["ctrl-o"]
[question]
complete = ["ctrl-t"]
"#;

fn remapped(name: &str) -> App {
    let root = repo(name, &[("docs/notes/2026-11-18-a.md", "# A\n\nOne apple.\n"), ("docs/notes/2026-11-18-b.md", "# B\n\nTwo apples.\n"), ("docs/zendo/x.md", "# X\n")]);
    let mut a = app(&root);
    let cfg: KeyConfig = toml::from_str(KEYS).unwrap();
    let (keys, problems) = Keymap::new(None, &cfg);
    assert!(problems.is_empty(), "{problems:?}");
    a.keys = keys;
    a
}

fn press(a: &mut App, code: KeyCode, m: KeyModifiers) {
    a.on_key(KeyEvent::new(code, m));
}

/// PANES-48: in the theme list, the keys of the keymap move and keep; the old keys do not; `esc`
/// still cancels.
#[test]
fn the_theme_list_takes_its_keys_from_the_keymap() {
    let mut a = remapped("box-theme");
    a.mode = Mode::Theme;
    a.theme_before = a.theme;
    let first = a.theme;
    press(&mut a, KeyCode::Char('j'), KeyModifiers::NONE);
    assert_eq!(a.theme, first, "j is not the key any more");
    press(&mut a, KeyCode::Char('d'), KeyModifiers::CONTROL);
    assert_ne!(a.theme, first);
    press(&mut a, KeyCode::Esc, KeyModifiers::NONE);
    assert_eq!((a.mode, a.theme), (Mode::Read, first), "esc is fixed");
    a.mode = Mode::Theme;
    press(&mut a, KeyCode::Char('d'), KeyModifiers::CONTROL);
    press(&mut a, KeyCode::Char('o'), KeyModifiers::CONTROL);
    assert_eq!(a.mode, Mode::Read);
    assert_ne!(a.theme, first, "ctrl-o keeps the theme");
}

/// PANES-48: in the find panel, the keys of the keymap move and open; a plain char types. The
/// search of the notes is the Notes mode of the panel (FILES-69).
#[test]
fn the_search_box_takes_its_keys_from_the_keymap() {
    let mut a = remapped("box-search");
    a.open_find();
    press(&mut a, KeyCode::BackTab, KeyModifiers::SHIFT);
    for c in "apple".chars() {
        press(&mut a, KeyCode::Char(c), KeyModifiers::NONE);
    }
    a.settle_find();
    assert_eq!(a.files.find().items.len(), 2, "a plain char types");
    press(&mut a, KeyCode::Down, KeyModifiers::NONE);
    assert_eq!(a.files.find().sel, 0, "down is not the key any more");
    press(&mut a, KeyCode::Char('d'), KeyModifiers::CONTROL);
    assert_eq!(a.files.find().sel, 1);
    press(&mut a, KeyCode::Enter, KeyModifiers::NONE);
    assert_eq!(a.mode, Mode::Find, "enter is not the key any more");
    press(&mut a, KeyCode::Char('o'), KeyModifiers::CONTROL);
    assert_eq!(a.mode, Mode::Read);
}

/// PANES-48: in the question of a file, the key of the keymap completes the folder.
#[test]
fn the_question_of_a_file_takes_its_keys_from_the_keymap() {
    let mut a = remapped("box-question");
    a.files.ask_file(Some("docs/notes/2026-11-18-a.md".into()), 'm');
    let text = |a: &App| a.files.prompt().and_then(|p| p.field().map(|f| f.text().to_string())).unwrap_or_default();
    assert_eq!(text(&a), "docs/notes");
    for _ in 0..5 {
        press(&mut a, KeyCode::Backspace, KeyModifiers::NONE);
    }
    for c in "zen".chars() {
        press(&mut a, KeyCode::Char(c), KeyModifiers::NONE);
    }
    press(&mut a, KeyCode::Tab, KeyModifiers::NONE);
    assert_eq!(text(&a), "docs/zen", "tab is not the key any more");
    press(&mut a, KeyCode::Char('t'), KeyModifiers::CONTROL);
    assert_eq!(text(&a), "docs/zendo/");
}
