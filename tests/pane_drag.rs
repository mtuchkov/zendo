//! A drag of the edge between two panes changes the width of the pane (PANES-10).

mod common;
use common::*;
use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

fn mouse(a: &mut zendo::app::App, kind: MouseEventKind, x: u16) {
    a.on_mouse(MouseEvent { kind, column: x, row: 5, modifiers: KeyModifiers::NONE });
}

fn chat_width(a: &zendo::app::App) -> u16 {
    a.pane_rects.iter().find(|(p, _)| *p == zendo::app::Focus::Claude).map(|(_, r)| r.width).expect("the chat pane")
}

// PANES-10: a press on the edge, or one column left or right of it, grabs the edge; a drag to
// the left widens the chat.
#[test]
fn dragging_the_chat_edge_left_widens_the_chat() {
    for off in [-1i32, 0, 1] {
        let root = repo(&format!("pane-drag{off}"), &[("docs/notes/n.md", "# N\n"), ("docs/zendo/zendo.md", "# Zendo\n")]);
        let mut a = app(&root);
        a.panel = true;
        draw(&mut a, 120, 30);
        let (before, edge) = (chat_width(&a), a.pane_rects[1].1.x);
        let x = (edge as i32 + off) as u16;
        mouse(&mut a, MouseEventKind::Down(MouseButton::Left), x);
        assert!(a.drag.is_some(), "a press {off} columns off the edge grabs it");
        mouse(&mut a, MouseEventKind::Drag(MouseButton::Left), x - 10);
        mouse(&mut a, MouseEventKind::Up(MouseButton::Left), x - 10);
        draw(&mut a, 120, 30);
        assert_eq!(chat_width(&a), before + 10, "offset {off}");
        let _ = std::fs::remove_dir_all(&root);
    }
}
