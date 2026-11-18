//! Undo and redo in every input, with the loglets of documents in `.zendo/db.log.json`
//! (UNDO-1 to UNDO-10).

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use zendo::app::{App, Focus, Mode};
use zendo::keys::{Keymap, Pane};

const NOTE: &str = "docs/notes/2026-11-18-undo.md";

fn files() -> Vec<(&'static str, &'static str)> {
    vec![(NOTE, "# Undo\n\nThe first paragraph.\n"), ("docs/zendo/zendo.md", "# Zendo\n")]
}

/// Write mode in a new paragraph at the end of the note.
fn writing(root: &std::path::Path) -> App {
    let mut a = app(root);
    draw(&mut a, 120, 30);
    key(&mut a, KeyCode::Char('i'));
    draw(&mut a, 120, 30);
    assert_eq!(a.mode, Mode::Write);
    a.writer.editor_mut().move_to_end();
    key(&mut a, KeyCode::Enter);
    key(&mut a, KeyCode::Enter);
    a
}

fn text(a: &App) -> String {
    a.writer.editor().text()
}

fn log_file(root: &std::path::Path) -> std::path::PathBuf {
    root.join(".zendo/db.log.json")
}

// UNDO-1, UNDO-2: ctrl-z undoes the last change in write mode, ctrl-y does it again.
#[tokio::test]
async fn undo_and_redo_in_write_mode() {
    let root = repo("undo-write", &files());
    let mut a = writing(&root);
    typ(&mut a, "alpha beta");
    ctrl(&mut a, 'z');
    assert!(text(&a).ends_with("\n\nalpha "), "{}", text(&a));
    ctrl(&mut a, 'y');
    assert!(text(&a).ends_with("\n\nalpha beta"), "{}", text(&a));
    let _ = std::fs::remove_dir_all(&root);
}

// UNDO-3: a new change after undo trims the tail: the undone word does not come back.
#[tokio::test]
async fn a_new_change_after_undo_trims_the_tail() {
    let root = repo("undo-trim", &files());
    let mut a = writing(&root);
    typ(&mut a, "alpha beta");
    ctrl(&mut a, 'z');
    typ(&mut a, "x");
    ctrl(&mut a, 'y');
    assert!(text(&a).ends_with("alpha x"), "{}", text(&a));
    let _ = std::fs::remove_dir_all(&root);
}

// UNDO-4: the loglet of a note is in `.zendo/db.log.json`; undo works after a restart.
#[tokio::test]
async fn undo_works_after_a_restart() {
    let root = repo("undo-restart", &files());
    let mut a = writing(&root);
    typ(&mut a, "added");
    key(&mut a, KeyCode::Esc);
    assert!(std::fs::read_to_string(root.join(NOTE)).unwrap().contains("added"));
    let log = std::fs::read_to_string(log_file(&root)).expect("db.log.json");
    assert!(log.contains(NOTE), "{log}");
    drop(a);
    let mut b = writing(&root);
    // writing() pressed enter twice: take those back first, then the word.
    ctrl(&mut b, 'z');
    ctrl(&mut b, 'z');
    ctrl(&mut b, 'z');
    assert!(!text(&b).contains("added"), "{}", text(&b));
    let _ = std::fs::remove_dir_all(&root);
}

// UNDO-1, UNDO-5: the chat input and a text field undo their own changes, and keep no loglet on disk.
#[tokio::test]
async fn undo_in_the_chat_input_and_a_field() {
    let root = repo("undo-chat-field", &files());
    let mut a = app(&root);
    a.open_panel();
    a.focus = Focus::Claude;
    typ(&mut a, "hello world");
    ctrl(&mut a, 'z');
    assert_eq!(a.composer.input().text(), "hello ");
    ctrl(&mut a, 'y');
    assert_eq!(a.composer.input().text(), "hello world");
    a.focus = Focus::Note;
    a.on_key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::ALT));
    assert_eq!(a.mode, Mode::Find);
    typ(&mut a, "zen");
    ctrl(&mut a, 'z');
    assert_eq!(a.files.find().query.text(), "");
    ctrl(&mut a, 'y');
    assert_eq!(a.files.find().query.text(), "zen");
    assert!(!log_file(&root).exists(), "no document changed");
    let _ = std::fs::remove_dir_all(&root);
}

// UNDO-6: the agent writes the note; undo takes back the change of the user and keeps the agent's.
#[tokio::test]
async fn undo_keeps_the_change_of_the_agent() {
    let root = repo("undo-agent", &files());
    let mut a = writing(&root);
    typ(&mut a, "mine");
    a.save();
    let disk = std::fs::read_to_string(root.join(NOTE)).unwrap();
    std::fs::write(root.join(NOTE), format!("theirs\n\n{disk}")).unwrap();
    a.reload_after_agent();
    assert!(text(&a).starts_with("theirs") && text(&a).ends_with("mine"), "{}", text(&a));
    ctrl(&mut a, 'z');
    assert!(text(&a).starts_with("theirs") && !text(&a).contains("mine"), "{}", text(&a));
    let _ = std::fs::remove_dir_all(&root);
}

// UNDO-8: undo and redo put the cursor at the end of the change.
#[tokio::test]
async fn undo_moves_the_cursor_to_the_change() {
    let root = repo("undo-cursor", &files());
    let mut a = writing(&root);
    typ(&mut a, "alpha beta");
    key(&mut a, KeyCode::Home);
    ctrl(&mut a, 'z');
    let e = a.writer.editor();
    assert_eq!((e.col, e.lines[e.row].as_str()), (6, "alpha "));
    let _ = std::fs::remove_dir_all(&root);
}

// UNDO-10: the keys of undo and redo are in the keymap of every input and in the help.
#[test]
fn undo_keys_are_in_the_keymap() {
    let k = Keymap::defaults();
    let ev = |c| KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL);
    for pane in [Pane::Write, Pane::Chat, Pane::Field] {
        assert_eq!(k.local(pane, &ev('z')), Some("undo"), "{pane:?}");
        assert_eq!(k.local(pane, &ev('y')), Some("redo"), "{pane:?}");
    }
    let root = repo("undo-help", &files());
    let mut a = app(&root);
    draw(&mut a, 120, 30);
    key(&mut a, KeyCode::Char('e'));
    key(&mut a, KeyCode::F(1));
    let screen = draw(&mut a, 200, 70).join("\n");
    assert!(screen.contains("undo the last change"), "{screen}");
    let _ = std::fs::remove_dir_all(&root);
}
