//! Note to spec (docs/zendo/note-to-spec.md): `[ → spec ]` and `s` give the agent the task to turn
//! a note into specifications.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use zendo::app::{App, Item};

const NOTE: &str = "docs/notes/2026-11-17-tea.md";
const DESIGN: &str = "docs/design.md";
const BUTTON: &str = "[ → spec ]";

/// The questions of the user in the chat of the active agent session.
fn asked(a: &App) -> Vec<String> {
    a.threads.get(&a.key()).map_or(Vec::new(), |t| t.items.iter().filter_map(|i| if let Item::User(s) = i { Some(s.clone()) } else { None }).collect())
}

/// The column and the row of `text` on the screen.
fn find(screen: &[String], text: &str) -> Option<(u16, u16)> {
    screen.iter().enumerate().find_map(|(y, l)| l.find(text).map(|i| (l[..i].chars().count() as u16, y as u16)))
}

// SPEC-1, SPEC-3: a click on `[ → spec ]` of a note sends the task to the chat; a document
// outside the notes folder has no button, and `s` sends nothing for it.
#[tokio::test]
async fn s_and_the_button_send_the_note_to_spec() {
    let root = repo("note-to-spec", &[(NOTE, "# Tea\n\nGreen tea is calm.\n"), (DESIGN, "# Design\n\nThe design.\n")]);
    let mut a = app(&root);
    a.open_file(root.join(DESIGN));
    let screen = draw(&mut a, 120, 40);
    assert!(find(&screen, BUTTON).is_none(), "no button on a document:\n{}", screen.join("\n"));
    key(&mut a, KeyCode::Char('s'));
    assert!(asked(&a).is_empty(), "no task for a document: {:?}", asked(&a));
    a.open_file(root.join(NOTE));
    let screen = draw(&mut a, 120, 40);
    let (x, y) = find(&screen, BUTTON).unwrap_or_else(|| panic!("the button on a note:\n{}", screen.join("\n")));
    a.on_mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column: x + 3, row: y, modifiers: KeyModifiers::NONE });
    a.on_mouse(MouseEvent { kind: MouseEventKind::Up(MouseButton::Left), column: x + 3, row: y, modifiers: KeyModifiers::NONE });
    let q = asked(&a);
    assert_eq!(q.len(), 1, "one task: {q:?}");
    assert!(q[0].contains(NOTE) && q[0].contains("specifications"), "{q:?}");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}
