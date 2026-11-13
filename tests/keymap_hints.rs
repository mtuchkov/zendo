//! Key hints and messages take their keys from the keymap in effect (docs/zendo/panes.md: PANES-28,
//! PANES-29): with other keys in the config, no hint shows a default key.

mod common;
use common::*;
use std::time::Instant;
use zendo::app::{App, Focus, Mode, session_key};
use zendo::docs::tree::View;
use zendo::keys::{KeyConfig, Keymap, Pane};

/// Other keys than the defaults for the actions that the messages name.
const KEYS: &str = r#"
[global]
help = ["f12"]
[read]
review = ["ctrl-y"]
upgrade = ["ctrl-u"]
new = ["N"]
chat = ["C"]
[write]
review = ["ctrl-y"]
[chat]
send = ["ctrl-s"]
back = ["ctrl-g"]
[notes]
new = ["N"]
[tree]
view = ["V"]
"#;

fn remapped(a: &mut App) {
    let cfg: KeyConfig = toml::from_str(KEYS).unwrap();
    let (keys, problems) = Keymap::new(None, &cfg);
    assert!(problems.is_empty(), "{problems:?}");
    a.keys = keys;
}

fn with_notes(name: &str) -> (std::path::PathBuf, App) {
    let root = repo(name, &[("docs/notes/2026-11-13-a.md", "# A\n\nText.\n")]);
    let mut a = app(&root);
    remapped(&mut a);
    (root, a)
}

fn without_notes(name: &str) -> (std::path::PathBuf, App) {
    let root = repo(name, &[("README.txt", "no notes\n")]);
    let mut a = app(&root);
    remapped(&mut a);
    (root, a)
}

fn flash(a: &App) -> String {
    a.flash.as_ref().map(|(s, _)| s.clone()).unwrap_or_default()
}

fn screen(a: &mut App) -> String {
    draw(a, 160, 30).join("\n")
}

// PANES-28: the review asks for its own key again, not for ctrl-r.
#[test]
fn the_review_names_its_key() {
    let (root, mut a) = with_notes("hints-review");
    a.review_key();
    let f = flash(&a);
    assert!(f.starts_with("^y again"), "{f}");
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-28: a newer zendo names the upgrade key of the keymap, not U.
#[tokio::test]
async fn the_upgrade_message_names_its_key() {
    let (root, mut a) = with_notes("hints-upgrade");
    let (tx, rx) = tokio::sync::oneshot::channel();
    tx.send(Some("abc1234".to_string())).unwrap();
    a.update_rx = Some(rx);
    a.tick();
    let f = flash(&a);
    assert!(f.contains("^u upgrades now") && !f.contains("U upgrades"), "{f}");
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-28: while the chat answers, the input names the send key and the stop key of the keymap.
#[tokio::test]
async fn the_busy_chat_names_its_keys() {
    let (root, mut a) = with_notes("hints-busy");
    a.open_panel();
    a.focus = Focus::Claude;
    a.thread(&session_key(0)).busy = Some(Instant::now());
    let s = draw(&mut a, 400, 30).join("\n");
    assert!(s.contains("^s queues the next question · ^g stops"), "{s}");
    assert!(!s.contains("enter queues") && !s.contains("esc stops"), "{s}");
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-28: a question about a selection names the send key of the keymap.
#[tokio::test]
async fn the_question_about_a_selection_names_the_send_key() {
    let (root, mut a) = with_notes("hints-ask");
    a.open_panel();
    a.focus = Focus::Claude;
    a.ask_context = Some("Text.".into());
    let s = screen(&mut a);
    assert!(s.contains("or press ^s"), "{s}");
    assert!(!s.contains("press enter"), "{s}");
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-28: the screen without notes names the keys of the keymap.
#[test]
fn the_screen_without_notes_names_its_keys() {
    let (root, mut a) = without_notes("hints-empty");
    let s = screen(&mut a);
    assert!(s.contains("Press N to write the first one"), "{s}");
    assert!(s.contains("Press C to talk to Claude, f12 for keys."), "{s}");
    assert!(!s.contains("alt-?"), "{s}");
    a.mode = Mode::Notes;
    let s = screen(&mut a);
    assert!(s.contains("No notes yet. Press N."), "{s}");
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-28: the empty docs view of the tree names the view key of the keymap.
#[test]
fn the_empty_docs_view_names_the_view_key() {
    let (root, mut a) = without_notes("hints-tree");
    a.state.tree = true;
    a.tree.view = View::Docs;
    a.focus = Focus::Tree;
    let s = screen(&mut a);
    assert!(s.contains("Press V to show all."), "{s}");
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-29: without keys, no hint and no message names a key.
#[test]
fn an_action_without_a_key_has_no_hint() {
    let (root, mut a) = without_notes("hints-none");
    a.keys = Keymap::default();
    let s = screen(&mut a);
    assert!(!s.contains("Press  ") && !s.contains("Press C"), "{s}");
    assert!(!s.contains(" for keys"), "no help hint without a help key:\n{s}");
    let status = draw(&mut a, 160, 30).into_iter().nth(28).unwrap_or_default();
    assert!(!status.contains(" keys "), "the status bar shows no hint without a key: {status}");
    a.review_key();
    assert!(!flash(&a).contains(" again"), "{}", flash(&a));
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-28: the waiting session view and the chat of an open Claude Code session name the
// hand-off key of the keymap.
#[test]
fn the_handoff_hints_name_its_key() {
    let (root, mut a) = with_notes("hints-handoff");
    let cfg: KeyConfig = toml::from_str("[chat]\nhandoff = [\"ctrl-k\"]\n").unwrap();
    a.keys = Keymap::new(None, &cfg).0;
    a.hold_cli(0);
    assert!(a.hold_line().contains("^k again"), "{}", a.hold_line());
    assert_eq!(a.keys.first(Pane::Chat, "handoff"), "^k");
    let _ = std::fs::remove_dir_all(&root);
}
