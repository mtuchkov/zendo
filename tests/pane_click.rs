//! A click in a pane moves the focus and the keys there, as `tab` does (PANES-46).

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use zendo::app::{App, Focus};
use zendo::keys::Pane;
use zendo::text::select::Region;

fn click(a: &mut App, x: u16, y: u16) {
    for kind in [MouseEventKind::Down(MouseButton::Left), MouseEventKind::Up(MouseButton::Left)] {
        a.on_mouse(MouseEvent { kind, column: x, row: y, modifiers: KeyModifiers::NONE });
    }
}

fn pane(a: &App, p: Focus) -> Rect {
    a.frame.pane_rects().iter().find(|(f, _)| *f == p).map(|(_, r)| *r).expect("the pane is on the screen")
}

fn region(a: &App, want: Region) -> Rect {
    a.frame.regions().iter().find(|(r, _)| *r == want).map(|(_, r)| *r).expect("the region is on the screen")
}

/// The note, the chat and the tree on the screen, with the focus on the note.
fn three_panes(name: &str) -> (std::path::PathBuf, App) {
    let root = repo(name, &[("docs/notes/n.md", "# N\n\nsome text\n"), ("docs/zendo/zendo.md", "# Zendo\n")]);
    let mut a = app(&root);
    a.open_file(root.join("docs/notes/n.md"));
    a.toggle_tree();
    a.threads.show();
    a.focus = Focus::Note;
    draw(&mut a, 160, 30);
    (root, a)
}

// PANES-46: a click in the transcript gives it the keys of the chat pane.
#[test]
fn a_click_in_the_transcript_gives_it_the_keys() {
    let (root, mut a) = three_panes("pane-click-transcript");
    let body = region(&a, Region::Panel);
    click(&mut a, body.x + 2, body.y + 1);
    assert_eq!(a.focus, Focus::Claude);
    assert_eq!(a.key_pane(), Pane::Transcript);
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-46: a click in the chat input gives it the keys, also when the transcript had them.
#[test]
fn a_click_in_the_chat_input_takes_the_keys_from_the_transcript() {
    let (root, mut a) = three_panes("pane-click-input");
    let body = region(&a, Region::Panel);
    click(&mut a, body.x + 2, body.y + 1);
    assert_eq!(a.key_pane(), Pane::Transcript);
    let chat = pane(&a, Focus::Claude);
    // The input box is under the transcript, above the bottom of the pane.
    click(&mut a, body.x + 4, chat.bottom() - 2);
    assert_eq!(a.focus, Focus::Claude);
    assert_eq!(a.key_pane(), Pane::Chat);
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-46: a click in the tree, below its rows, gives the tree the keys.
#[test]
fn a_click_in_the_tree_below_its_rows_gives_it_the_keys() {
    let (root, mut a) = three_panes("pane-click-tree");
    let tree = pane(&a, Focus::Tree);
    click(&mut a, tree.x + 2, tree.bottom() - 1);
    assert_eq!(a.focus, Focus::Tree);
    assert_eq!(a.key_pane(), Pane::Tree);
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-46: a click in the note gives the keys back to the note, from the transcript.
#[test]
fn a_click_in_the_note_takes_the_keys_from_the_chat() {
    let (root, mut a) = three_panes("pane-click-note");
    let body = region(&a, Region::Panel);
    click(&mut a, body.x + 2, body.y + 1);
    let note = region(&a, Region::Note);
    click(&mut a, note.x + 1, note.y);
    assert_eq!(a.focus, Focus::Note);
    assert_eq!(a.key_pane(), Pane::Read);
    // `tab` from the note comes into the chat at the input, as after a key.
    key(&mut a, KeyCode::Tab);
    draw(&mut a, 160, 30);
    assert_eq!(a.focus, Focus::Claude);
    assert_eq!(a.key_pane(), Pane::Chat);
    let _ = std::fs::remove_dir_all(&root);
}
