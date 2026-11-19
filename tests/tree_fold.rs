//! The folders of the file tree: closed at the start, `[+]` and `[-]` in its head, and the open
//! document that the tree follows.

use crate::common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use zendo::app::{App, Focus};

fn files() -> Vec<(&'static str, &'static str)> {
    vec![("docs/notes/2026-11-17-a.md", "# A\n"), ("docs/adr/0001-x.md", "# X\n"), ("docs/guides/howto.md", "# How\n")]
}

fn click(a: &mut App, open: bool) {
    let (r, _) = *a.frame.tree_fold().iter().find(|(_, o)| *o == open).expect("the button is on the screen");
    a.on_mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column: r.x + 1, row: r.y, modifiers: KeyModifiers::NONE });
}

fn shown(screen: &[String], text: &str) -> bool {
    screen.iter().any(|l| l.contains(text))
}

/// The rows of the tree after a draw, with their indent; the note pane may show the same names.
fn rows(a: &mut App) -> Vec<String> {
    draw(a, 140, 30);
    a.files.tree().rows().iter().map(|r| format!("{}{}", "  ".repeat(r.depth), r.name)).collect()
}

// FILES-56, FILES-58: the tree opens with its folders closed, except the folders of the open
// document; the open document is selected.
#[test]
fn the_tree_opens_closed_at_the_open_document() {
    let root = repo("tree-fold-open", &files());
    let mut a = app(&root);
    a.open_file(root.join("docs/adr/0001-x.md"));
    a.toggle_tree();
    let screen = draw(&mut a, 140, 30);
    assert!(shown(&screen, "[-] adr/") && shown(&screen, "[+] guides/") && shown(&screen, "[+] notes/"), "{screen:#?}");
    assert_eq!(rows(&mut a), vec!["docs", "  adr", "    0001-x.md", "  guides", "  notes"], "other folders stay closed");
    assert_eq!(a.files.tree_file().as_deref(), Some(std::path::Path::new("docs/adr/0001-x.md")));
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-57: `[+]` opens every folder, `[-]` closes every folder; the keys do the same.
#[test]
fn the_head_buttons_open_and_close_every_folder() {
    let root = repo("tree-fold-all", &files());
    let mut a = app(&root);
    a.open_file(root.join("docs/adr/0001-x.md"));
    a.toggle_tree();
    let screen = draw(&mut a, 140, 30);
    assert!(shown(&screen, "[+] [-]"), "{screen:#?}");
    let all = vec!["docs", "  adr", "    0001-x.md", "  guides", "    howto.md", "  notes", "    2026-11-17-a.md"];
    click(&mut a, true);
    assert_eq!(rows(&mut a), all);
    click(&mut a, false);
    assert_eq!(rows(&mut a), vec!["docs"], "every folder closes, also the one of the open document");
    a.focus = Focus::Tree;
    key(&mut a, KeyCode::Char('*'));
    assert_eq!(rows(&mut a), all);
    key(&mut a, KeyCode::Char('_'));
    assert_eq!(rows(&mut a), vec!["docs"]);
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-58: while the tree shows, it follows each file that opens, also one that was open before.
#[test]
fn the_tree_follows_each_file_that_opens() {
    let root = repo("tree-fold-follow", &files());
    let mut a = app(&root);
    a.open_file(root.join("docs/adr/0001-x.md"));
    a.toggle_tree();
    draw(&mut a, 140, 30);
    a.open_file(root.join("docs/guides/howto.md"));
    assert!(rows(&mut a).contains(&"    howto.md".to_string()));
    assert_eq!(a.files.tree_file().as_deref(), Some(std::path::Path::new("docs/guides/howto.md")));
    click(&mut a, false);
    draw(&mut a, 140, 30);
    assert_eq!(rows(&mut a), vec!["docs"]);
    a.open_file(root.join("docs/guides/howto.md"));
    assert!(rows(&mut a).contains(&"    howto.md".to_string()), "the same file opens again");
    let _ = std::fs::remove_dir_all(&root);
}

