//! A selection that starts in a table with box borders keeps to the columns (PANES-34): the keys of
//! read mode and the mouse.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, MouseButton, MouseEvent, MouseEventKind, KeyModifiers};

const NOTE: &str = "docs/notes/2026-11-13-table.md";

fn files() -> Vec<(&'static str, &'static str)> {
    vec![
        (NOTE, "# Table\n\nBefore the table.\n\n| Field | What to enter |\n|---|---|\n| Name | keep it |\n| Branch | main |\n| Path | the folder |\n\nAfter.\n"),
        ("docs/zendo/zendo.md", "# Zendo\n"),
    ]
}

fn mouse(a: &mut zendo::app::App, kind: MouseEventKind, x: usize, y: usize) {
    a.on_mouse(MouseEvent { kind, column: x as u16, row: y as u16, modifiers: KeyModifiers::NONE });
}

// PANES-34: `v`, a move down, `y`: only the cells of the column.
#[test]
fn the_keys_of_read_mode_select_in_a_column() {
    let root = repo("table-keys", &files());
    let mut a = app(&root);
    draw(&mut a, 120, 30);
    a.read_cursor = a.read_text.iter().position(|r| r.contains("keep it")).expect("the table row");
    a.read_col = a.read_text[a.read_cursor].chars().position(|c| c == 'k').unwrap();
    a.read_want = a.read_col;
    key(&mut a, KeyCode::Char('v'));
    key(&mut a, KeyCode::Char('j'));
    key(&mut a, KeyCode::Char('j'));
    draw(&mut a, 120, 30);
    key(&mut a, KeyCode::Char('y'));
    let clip = a.clip.clone().unwrap_or_default();
    assert_eq!(clip, "keep it\nmain\nt");
    assert!(!clip.contains("Branch") && !clip.contains('│'), "{clip:?}");
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-34: a drag that starts in a cell copies only the cells of its column.
#[test]
fn a_drag_in_a_table_copies_the_column() {
    let root = repo("table-mouse", &files());
    let mut a = app(&root);
    let screen = draw(&mut a, 120, 30);
    let y = screen.iter().position(|r| r.contains("keep it")).expect("the table row");
    let x = screen[y].chars().take_while(|c| *c != 'k').count();
    mouse(&mut a, MouseEventKind::Down(MouseButton::Left), x, y);
    mouse(&mut a, MouseEventKind::Drag(MouseButton::Left), x + 3, y + 2);
    mouse(&mut a, MouseEventKind::Up(MouseButton::Left), x + 3, y + 2);
    draw(&mut a, 120, 30);
    assert_eq!(a.clip.as_deref(), Some("keep it\nmain\nthe"));
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-34: a drag that starts before the table goes line by line, the borders too.
#[test]
fn a_drag_from_before_a_table_goes_line_by_line() {
    let root = repo("table-before", &files());
    let mut a = app(&root);
    let screen = draw(&mut a, 120, 30);
    let y = screen.iter().position(|r| r.contains("Before the table")).expect("the text");
    let x = screen[y].chars().take_while(|c| *c != 'B').count();
    mouse(&mut a, MouseEventKind::Down(MouseButton::Left), x, y);
    mouse(&mut a, MouseEventKind::Drag(MouseButton::Left), x + 6, y + 3);
    mouse(&mut a, MouseEventKind::Up(MouseButton::Left), x + 6, y + 3);
    draw(&mut a, 120, 30);
    let clip = a.clip.clone().unwrap_or_default();
    assert!(clip.starts_with("Before the table.") && clip.contains('┌') && clip.contains("│ Field"), "{clip:?}");
    let _ = std::fs::remove_dir_all(&root);
}
