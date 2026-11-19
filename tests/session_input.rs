//! The chat input of each agent session (ADR 0011): its own text and chips, kept across a switch
//! and a restart; a chip whose file is gone is red and goes without its file.

use crate::common;
use common::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use zendo::app::{App, Focus};

const A: &str = "docs/notes/a.md";
const B: &str = "docs/notes/b.md";

fn files() -> Vec<(&'static str, &'static str)> {
    vec![(A, "# A\n\ntext\n"), (B, "# B\n\nmore\n")]
}

/// The screen as text, and its cells for the colors.
fn draw_buf(a: &mut App, w: u16, h: u16) -> (Vec<String>, Buffer) {
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| a.draw(f)).unwrap();
    let buf = term.backend().buffer().clone();
    let rows = (0..h).map(|y| (0..w).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>()).collect();
    (rows, buf)
}

// CLAUDE-43, CLAUDE-44: each agent session has its own input; a switch shows the input of the
// other session and keeps the one before.
#[tokio::test]
async fn each_session_keeps_its_own_chat_input() {
    let root = repo("input-sessions", &files());
    let mut a = app(&root);
    a.open_panel();
    a.focus = Focus::Claude;
    typ(&mut a, "one ");
    a.on_paste("line a\nline b\n".into());
    assert_eq!(a.composer().chips().len(), 1);
    a.switch_session(1);
    a.focus = Focus::Claude;
    assert_eq!(a.composer().input().text(), "", "session 1 starts empty");
    assert!(a.composer().chips().is_empty());
    typ(&mut a, "two");
    a.switch_session(0);
    assert_eq!(a.composer().input().text(), "one ⟦line a line b · 2 lines⟧ ");
    assert_eq!(a.composer().chips().len(), 1, "the chip and its text stay with session 0");
    a.switch_session(1);
    assert_eq!(a.composer().input().text(), "two");
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-45, CLAUDE-46: the input of each session comes back after a restart; a chip whose file
// is gone shows in red.
#[tokio::test]
async fn the_input_of_a_session_comes_back_after_a_restart() {
    let root = repo("input-restart", &files());
    {
        let mut a = app(&root);
        a.open_panel();
        a.focus = Focus::Claude;
        typ(&mut a, "why ");
        a.on_paste(root.join(B).display().to_string());
        a.on_paste("x\ny\n".into());
        a.switch_session(1);
        a.focus = Focus::Claude;
        typ(&mut a, "later");
        a.quit = true;
        assert!(a.settle_quit());
    }
    std::fs::remove_file(root.join(B)).unwrap();
    let mut a = app(&root);
    assert_eq!(a.composer().input().text(), "later", "the active session comes back with its input");
    a.switch_session(0);
    let text = a.composer().input().text();
    assert!(text.starts_with("why ⟦otes/b.md⟧ ⟦x y · 2 lines⟧"), "{text}");
    assert!(a.composer().chips()[1].context.as_deref().is_some_and(|c| c.contains("> y")), "the chip keeps its text");
    a.focus = Focus::Claude;
    let (rows, buf) = draw_buf(&mut a, 160, 30);
    let y = rows.iter().position(|r| r.contains(" otes/b.md ")).expect("the chip of the gone file");
    let col = rows[y][..rows[y].find(" otes/b.md ").unwrap()].chars().count() + 1;
    let t = *a.theme();
    assert_eq!(buf[(col as u16, y as u16)].bg, t.del, "the chip of a gone file is red");
    let _ = std::fs::remove_dir_all(&root);
}
