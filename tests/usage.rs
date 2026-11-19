//! Usage counts (docs/zendo/usage.md): each action of the keymap becomes an event of the telemetry
//! log with its pane, its action and its trigger, at the place where it runs; zendo never reads the
//! log.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use zendo::app::App;
use zendo::keys::Pane;
use zendo::responsiveness::Responsiveness;
use zendo::telemetry::{Telemetry, Trigger};

const NOTE: &str = "docs/notes/2026-11-14-a.md";

/// An app that records into a data folder of this test; what an earlier run left there is removed.
fn counting(name: &str) -> (PathBuf, PathBuf, App) {
    let root = repo(name, &[(NOTE, "# A\n\nBefore the table there is text.\n\nMore text here.\n")]);
    let data = std::env::temp_dir().join(format!("zendo-usage-data-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data);
    let mut a = app(&root);
    a.keys = zendo::keys::Keymap::defaults();
    (a.telemetry, a.responsiveness) = (Telemetry::start(&data, std::process::id()), Responsiveness::on());
    (root, data, a)
}

/// The events of `pane` and `action` with `trigger` that wait in memory.
fn count(a: &App, pane: Pane, action: &str, trigger: Trigger) -> usize {
    a.telemetry.waiting().iter().filter(|e| e.pane == pane && e.action == action && e.trigger == trigger).count()
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
    assert_eq!(count(&a, Pane::Read, "add_to_chat", Trigger::Key), 2, "pane {:?}", a.key_pane());
    // The mouse: alt and a drag put the selection into the chat (PANES-44).
    let y = screen.iter().position(|r| r.contains("Before the table")).expect("the text");
    let x = screen[y].chars().take_while(|c| *c != 'B').count();
    mouse(&mut a, MouseEventKind::Down(MouseButton::Left), x, y, KeyModifiers::NONE);
    mouse(&mut a, MouseEventKind::Drag(MouseButton::Left), x + 9, y, KeyModifiers::NONE);
    mouse(&mut a, MouseEventKind::Up(MouseButton::Left), x + 9, y, KeyModifiers::ALT);
    assert_eq!(count(&a, Pane::Read, "add_to_chat", Trigger::Mouse), 1);
    assert_eq!(count(&a, Pane::Read, "add_to_chat", Trigger::Key), 2, "the mouse is another trigger");
    // A global key in a pane counts under `global`.
    let before = a.state.tree;
    alt(&mut a, 'b');
    assert_ne!(a.state.tree, before, "the file tree opens or closes");
    assert_eq!(count(&a, Pane::Global, "toggle_tree", Trigger::Key), 1);
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
    a.end_telemetry();
    let events = zendo::telemetry::read(&data);
    let text = format!("{events:?}");
    let of = |pane: &str| events.iter().filter(|e| e["pane"] == pane).count();
    assert_eq!((of("read"), events.iter().filter(|e| e["action"] == "write").count()), (1, 1), "{text}");
    assert_eq!(of("write"), 0, "typing counts nothing: {text}");
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
    assert_eq!(count(&a, Pane::Read, "copy", Trigger::Mouse), 1);
    // A click with no drag selects nothing and copies nothing.
    mouse(&mut a, MouseEventKind::Down(MouseButton::Left), x, y, KeyModifiers::NONE);
    mouse(&mut a, MouseEventKind::Up(MouseButton::Left), x, y, KeyModifiers::NONE);
    assert_eq!(count(&a, Pane::Read, "copy", Trigger::Mouse), 1);
    clean(&[&root, &data]);
}

// TELEM-11: a write that fails shows no message, and the next action works.
#[tokio::test]
async fn a_failed_write_changes_nothing_on_the_screen() {
    let (root, data, mut a) = counting("fail");
    // A file where the data folder must be: zendo cannot write there.
    std::fs::write(&data, "").unwrap();
    let before = draw(&mut a, 120, 40);
    alt(&mut a, 'b');
    alt(&mut a, 'b');
    a.telemetry.tick(Instant::now() + Duration::from_secs(16));
    assert_eq!(a.flash.as_ref().map(|f| f.0.clone()), None, "no message");
    assert_eq!(draw(&mut a, 120, 40), before, "the screen is as before");
    alt(&mut a, 'b');
    assert_eq!(count(&a, Pane::Global, "toggle_tree", Trigger::Key), 1, "the next action works and counts");
    a.end_telemetry();
    assert_eq!(a.flash.as_ref().map(|f| f.0.clone()), None, "no message at the exit");
    let _ = std::fs::remove_file(&data);
    clean(&[&root]);
}

// USAGE-6: the hints are the same with no log and with a large log on the disk.
#[tokio::test]
async fn the_hints_do_not_depend_on_the_counts() {
    let (root, data, mut a) = counting("hints");
    let (root2, data2, mut b) = counting("hints-full");
    draw(&mut b, 120, 40);
    for _ in 0..250 {
        alt(&mut b, 'b');
    }
    b.end_telemetry();
    assert_eq!(zendo::telemetry::read(&data2).len(), 250, "the log of b");
    let (sa, sb) = (draw(&mut a, 120, 40), draw(&mut b, 120, 40));
    assert_eq!(sa.last(), sb.last(), "the status bar");
    key(&mut a, KeyCode::F(1));
    key(&mut b, KeyCode::F(1));
    let name = |s: Vec<String>, root: &PathBuf| s.join("\n").replace(root.file_name().unwrap().to_str().unwrap(), "repo");
    assert_eq!(name(draw(&mut a, 120, 40), &root), name(draw(&mut b, 120, 40), &root2), "the help");
    clean(&[&root, &data, &root2, &data2]);
}
