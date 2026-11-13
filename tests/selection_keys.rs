//! The keys of `[selection]`: while text is selected where the user writes, `c` copies and `x`
//! cuts (PANES-37, PANES-38, PANES-39).

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use zendo::app::{Focus, Mode};
use zendo::keys::{KeyConfig, Keymap, Pane};

const NOTE: &str = "docs/notes/2026-11-13-sel.md";

fn files() -> Vec<(&'static str, &'static str)> {
    vec![(NOTE, "# Sel\n\nThe first paragraph.\n"), ("docs/zendo/zendo.md", "# Zendo\n")]
}

fn shift_right(a: &mut zendo::app::App, n: usize) {
    for _ in 0..n {
        a.on_key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));
    }
}

fn writing(name: &str) -> (std::path::PathBuf, zendo::app::App) {
    let root = repo(name, &files());
    let mut a = app(&root);
    draw(&mut a, 120, 30);
    a.read_cursor = a.read_text.iter().position(|r| r.starts_with("The first")).unwrap();
    a.read_col = 0;
    key(&mut a, KeyCode::Char('i'));
    draw(&mut a, 120, 30);
    assert_eq!(a.mode, Mode::Write);
    (root, a)
}

// PANES-37: in write mode with a selection, `c` copies and `x` cuts.
#[test]
fn c_copies_and_x_cuts_the_selection_in_write_mode() {
    let (root, mut a) = writing("sel-write");
    let line = a.editor.row;
    shift_right(&mut a, 3);
    key(&mut a, KeyCode::Char('c'));
    assert_eq!(a.clip.as_deref(), Some("The"));
    assert_eq!(a.editor.lines[line], "The first paragraph.", "copy keeps the text");
    assert!(a.editor.selection().is_none(), "copy ends the selection");
    key(&mut a, KeyCode::Right);
    shift_right(&mut a, 6);
    key(&mut a, KeyCode::Char('x'));
    assert_eq!(a.clip.as_deref(), Some("first "));
    assert_eq!(a.editor.lines[line], "The paragraph.");
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-38: without a selection, `c` and `x` are text.
#[test]
fn without_a_selection_c_and_x_are_text() {
    let (root, mut a) = writing("sel-text");
    let line = a.editor.row;
    typ(&mut a, "cx");
    assert_eq!(a.editor.lines[line], "cxThe first paragraph.");
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-37: in the chat input with a selection, `x` cuts.
#[tokio::test]
async fn x_cuts_the_selection_in_the_chat_input() {
    let root = repo("sel-chat", &files());
    let mut a = app(&root);
    a.open_panel();
    a.focus = Focus::Claude;
    typ(&mut a, "hello world");
    for _ in 0..5 {
        a.on_key(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT));
    }
    key(&mut a, KeyCode::Char('x'));
    assert_eq!((a.input.text().as_str(), a.clip.as_deref()), ("hello ", Some("world")));
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-39: the keys come from `[selection]` of the keymap, and the help shows them in a section of
// their own.
#[test]
fn the_selection_keys_are_in_the_keymap_and_the_help() {
    let k = Keymap::defaults();
    let ev = |c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE);
    assert_eq!((k.local(Pane::Selected, &ev('c')), k.local(Pane::Selected, &ev('x'))), (Some("copy"), Some("cut")));
    let cfg: KeyConfig = toml::from_str("[selection]\ncopy = \"y\"\n").unwrap();
    let (k, problems) = Keymap::new(None, &cfg);
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!((k.local(Pane::Selected, &ev('y')), k.local(Pane::Selected, &ev('c'))), (Some("copy"), None));
    let (root, mut a) = writing("sel-help");
    key(&mut a, KeyCode::F(1));
    let screen = draw(&mut a, 200, 70).join("\n");
    assert!(screen.contains("SELECTION") && screen.contains("cut the selection"), "{screen}");
    let _ = std::fs::remove_dir_all(&root);
}
