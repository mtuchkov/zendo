//! The update suggestion (docs/zendo/cli/upgrade.md): a newer zendo shows as a message and an
//! upgrade button in the top line, not as a key.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use zendo::app::App;

fn with_newer(name: &str) -> (std::path::PathBuf, App) {
    let root = repo(name, &[("docs/notes/2026-11-17-a.md", "# A\n\nText.\n")]);
    let mut a = app(&root);
    let (tx, rx) = tokio::sync::oneshot::channel();
    tx.send(Some("abc1234".to_string())).unwrap();
    a.update.watch(rx);
    a.tick();
    (root, a)
}

/// The first cell of `text` on the screen: column and row.
fn find(rows: &[String], text: &str) -> Option<(u16, u16)> {
    rows.iter().enumerate().find_map(|(y, row)| row.find(text).map(|i| (row[..i].chars().count() as u16, y as u16)))
}

fn click(a: &mut App, (column, row): (u16, u16)) {
    a.on_mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column, row, modifiers: KeyModifiers::NONE });
    a.on_mouse(MouseEvent { kind: MouseEventKind::Up(MouseButton::Left), column, row, modifiers: KeyModifiers::NONE });
}

// CMD-11: a newer version shows as a message and a button; no message names a key or a commit.
#[tokio::test]
async fn a_newer_version_shows_a_message_and_a_button() {
    let (root, mut a) = with_newer("upgrade-button-shows");
    let flash = a.flash.as_ref().map(|(s, _)| s.clone()).unwrap_or_default();
    assert_eq!(flash, "a new version of zendo is available");
    let rows = draw(&mut a, 160, 30);
    let top = &rows[0];
    assert!(top.contains("new version available"), "{top}");
    assert!(top.contains("[upgrade]"), "{top}");
    assert!(!top.contains("abc1234") && !top.contains("to upgrade"), "{top}");
    let _ = std::fs::remove_dir_all(&root);
}

// CMD-11: a click on the button upgrades zendo and starts it again.
#[tokio::test]
async fn the_upgrade_button_upgrades() {
    let (root, mut a) = with_newer("upgrade-button-click");
    let rows = draw(&mut a, 160, 30);
    let at = find(&rows, "[upgrade]").expect("the upgrade button");
    click(&mut a, at);
    assert!(a.update.upgrade_now() && a.quit);
    let _ = std::fs::remove_dir_all(&root);
}
