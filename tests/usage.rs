//! Usage counts (docs/zendo/usage.md): zendo counts the actions of the keymap by pane, action and
//! trigger, at the place where they run, and never reads the counts.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use zendo::app::App;
use zendo::keys::Pane;
use zendo::usage::{Trigger, Usage};

const NOTE: &str = "docs/notes/2026-11-14-a.md";

/// An app that counts into a data folder of this test; what an earlier run left there is removed.
fn counting(name: &str) -> (PathBuf, PathBuf, App) {
    let root = repo(name, &[(NOTE, "# A\n\nBefore the table there is text.\n\nMore text here.\n")]);
    let data = std::env::temp_dir().join(format!("zendo-usage-data-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    let mut a = app(&root);
    a.keys = zendo::keys::Keymap::defaults();
    a.usage = Usage::start(&data, std::process::id());
    (root, data, a)
}

fn alt(a: &mut App, c: char) {
    a.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT));
}

fn mouse(a: &mut App, kind: MouseEventKind, x: usize, y: usize, modifiers: KeyModifiers) {
    a.on_mouse(MouseEvent { kind, column: x as u16, row: y as u16, modifiers });
}

fn clean(dirs: &[&PathBuf]) {
    for d in dirs {
        let _ = std::fs::remove_dir_all(d);
    }
}

// USAGE-1, USAGE-7: an action counts under the keymap pane of its binding, its name and its trigger.
#[tokio::test]
async fn an_action_counts_under_its_pane_name_and_trigger() {
    let (root, data, mut a) = counting("count");
    let screen = draw(&mut a, 140, 30);
    alt(&mut a, 'a');
    key(&mut a, KeyCode::Esc);
    draw(&mut a, 140, 30);
    alt(&mut a, 'a');
    key(&mut a, KeyCode::Esc);
    let screen = if a.key_pane() == Pane::Read { draw(&mut a, 140, 30) } else { screen };
    assert_eq!(a.usage.get(Pane::Read, "add_to_chat", Trigger::Key), 2, "pane {:?}", a.key_pane());
    // The mouse: alt and a drag put the selection into the chat (PANES-44).
    let y = screen.iter().position(|r| r.contains("Before the table")).expect("the text");
    let x = screen[y].chars().take_while(|c| *c != 'B').count();
    mouse(&mut a, MouseEventKind::Down(MouseButton::Left), x, y, KeyModifiers::NONE);
    mouse(&mut a, MouseEventKind::Drag(MouseButton::Left), x + 9, y, KeyModifiers::NONE);
    mouse(&mut a, MouseEventKind::Up(MouseButton::Left), x + 9, y, KeyModifiers::ALT);
    assert_eq!(a.usage.get(Pane::Read, "add_to_chat", Trigger::Mouse), 1);
    assert_eq!(a.usage.get(Pane::Read, "add_to_chat", Trigger::Key), 2, "the mouse is another trigger");
    // A global key in a pane counts under `global`.
    let before = a.state.tree;
    alt(&mut a, 'b');
    assert_ne!(a.state.tree, before, "the file tree opens or closes");
    assert_eq!(a.usage.get(Pane::Global, "toggle_tree", Trigger::Key), 1);
    clean(&[&root, &data]);
}

// USAGE-8: typed text and the keys of a box are not actions; the action that opens the box is one.
#[tokio::test]
async fn typed_text_is_not_counted() {
    let (root, data, mut a) = counting("text");
    draw(&mut a, 140, 30);
    key(&mut a, KeyCode::Char('i'));
    assert_eq!(a.mode, zendo::app::Mode::Write);
    typ(&mut a, "some words");
    a.usage.write();
    let text = std::fs::read_to_string(a.usage.file().unwrap()).unwrap();
    let t: toml::Table = toml::from_str(&text).unwrap();
    assert_eq!(t["read"]["write"]["key"].as_integer(), Some(1), "{text}");
    assert!(!t.contains_key("write"), "typing counts nothing: {text}");
    assert!(!text.contains("some") && !text.contains("2026-11-14-a"), "no text and no file name: {text}");
    clean(&[&root, &data]);
}

// USAGE-1: the copy of a mouse selection, and a click on a row of the file tree.
#[tokio::test]
async fn a_mouse_selection_counts_as_a_copy() {
    let (root, data, mut a) = counting("mouse");
    let screen = draw(&mut a, 140, 30);
    let y = screen.iter().position(|r| r.contains("Before the table")).expect("the text");
    let x = screen[y].chars().take_while(|c| *c != 'B').count();
    mouse(&mut a, MouseEventKind::Down(MouseButton::Left), x, y, KeyModifiers::NONE);
    mouse(&mut a, MouseEventKind::Drag(MouseButton::Left), x + 9, y, KeyModifiers::NONE);
    mouse(&mut a, MouseEventKind::Up(MouseButton::Left), x + 9, y, KeyModifiers::NONE);
    assert_eq!(a.usage.get(Pane::Read, "copy", Trigger::Mouse), 1);
    // A click with no drag selects nothing and copies nothing.
    mouse(&mut a, MouseEventKind::Down(MouseButton::Left), x, y, KeyModifiers::NONE);
    mouse(&mut a, MouseEventKind::Up(MouseButton::Left), x, y, KeyModifiers::NONE);
    assert_eq!(a.usage.get(Pane::Read, "copy", Trigger::Mouse), 1);
    clean(&[&root, &data]);
}

// USAGE-5: a write that fails shows no message, and the next action works.
#[tokio::test]
async fn a_failed_write_changes_nothing_on_the_screen() {
    let (root, data, mut a) = counting("fail");
    // A file where the data folder must be: zendo cannot write there.
    std::fs::write(&data, "").unwrap();
    let before = draw(&mut a, 120, 40);
    alt(&mut a, 'b');
    alt(&mut a, 'b');
    a.usage.tick(Instant::now() + Duration::from_secs(16));
    assert_eq!(a.flash.as_ref().map(|f| f.0.clone()), None, "no message");
    assert_eq!(draw(&mut a, 120, 40), before, "the screen is as before");
    alt(&mut a, 'b');
    assert_eq!(a.usage.get(Pane::Global, "toggle_tree", Trigger::Key), 3, "the next action works and counts");
    let _ = std::fs::remove_file(&data);
    clean(&[&root]);
}

// USAGE-6: the hints are the same with no counts and with large counts on the disk.
#[tokio::test]
async fn the_hints_do_not_depend_on_the_counts() {
    let (root, data, mut a) = counting("hints");
    let (root2, data2, mut b) = counting("hints-full");
    for _ in 0..500 {
        b.usage.count(Pane::Read, "add_to_chat", Trigger::Key);
        b.usage.count(Pane::Global, "help", Trigger::Key);
    }
    b.usage.write();
    assert!(b.usage.file().unwrap().exists());
    let (sa, sb) = (draw(&mut a, 120, 40), draw(&mut b, 120, 40));
    assert_eq!(sa.last(), sb.last(), "the status bar");
    key(&mut a, KeyCode::F(1));
    key(&mut b, KeyCode::F(1));
    let name = |s: Vec<String>, root: &PathBuf| s.join("\n").replace(root.file_name().unwrap().to_str().unwrap(), "repo");
    assert_eq!(name(draw(&mut a, 120, 40), &root), name(draw(&mut b, 120, 40), &root2), "the help");
    clean(&[&root, &data, &root2, &data2]);
}
