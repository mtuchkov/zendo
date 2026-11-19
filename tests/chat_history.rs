//! The history of the messages of the user: per zendo session, the common history after `ctrl-n`,
//! and `up` and `down` in the chat input (docs/zendo/chat-history.md).

mod common;
use common::*;
use ratatui::crossterm::event::KeyCode;
use zendo::app::{App, Focus};

const A: &str = "docs/notes/2026-11-18-a.md";

/// Send a message from the chat input and wait until the agent is idle again.
async fn send(a: &mut App, q: &str) {
    a.focus = Focus::Claude;
    typ(a, q);
    key(a, KeyCode::Enter);
    until(a, |a| a.threads.values().all(|t| t.busy.is_none())).await;
}

fn input(a: &App) -> String {
    a.composer().input().text()
}

// HISTORY-1, HISTORY-5, HISTORY-6: `up` brings back the messages of the session, newest first;
// `down` goes forward and then brings back the draft.
#[tokio::test]
async fn up_and_down_go_through_the_messages_of_the_session() {
    let root = repo("history-arrows", &[(A, "# A\n")]);
    let mut a = app(&root);
    a.open_panel();
    send(&mut a, "first question").await;
    send(&mut a, "second question").await;
    typ(&mut a, "draft");
    key(&mut a, KeyCode::Up);
    assert_eq!(input(&a), "second question");
    key(&mut a, KeyCode::Up);
    assert_eq!(input(&a), "first question");
    key(&mut a, KeyCode::Up);
    assert_eq!(input(&a), "first question", "nothing before the oldest");
    key(&mut a, KeyCode::Down);
    assert_eq!(input(&a), "second question");
    key(&mut a, KeyCode::Down);
    assert_eq!(input(&a), "draft");
    let _ = std::fs::remove_dir_all(&root);
}

// HISTORY-2, HISTORY-4: `ctrl-n` merges the messages of a session into the common history, by
// time. A thread that started before the merge does not see them; its next new thread does.
#[tokio::test]
async fn ctrl_n_merges_the_session_into_the_common_history_by_time() {
    let root = repo("history-merge", &[(A, "# A\n")]);
    let mut a = app(&root);
    a.switch_session(0);
    send(&mut a, "a one").await;
    a.switch_session(1);
    send(&mut a, "b two").await;
    a.switch_session(0);
    send(&mut a, "a three").await;
    ctrl(&mut a, 'n');
    a.switch_session(1);
    key(&mut a, KeyCode::Up);
    assert_eq!(input(&a), "b two");
    key(&mut a, KeyCode::Up);
    assert_eq!(input(&a), "b two", "the thread of session 2 started before the merge");
    key(&mut a, KeyCode::Down);
    ctrl(&mut a, 'n');
    let mut seen = Vec::new();
    for _ in 0..3 {
        key(&mut a, KeyCode::Up);
        seen.push(input(&a));
    }
    assert_eq!(seen, ["a three", "b two", "a one"], "the new thread sees the common history, by time");
    let _ = std::fs::remove_dir_all(&root);
}

// HISTORY-3: the histories stay in `.zendo/db.json` and come back after a restart.
#[tokio::test]
async fn the_history_comes_back_after_a_restart() {
    let root = repo("history-restart", &[(A, "# A\n")]);
    let mut a = app(&root);
    a.switch_session(2);
    send(&mut a, "kept for later").await;
    drop(a);
    assert!(std::fs::read_to_string(root.join(".zendo/db.json")).unwrap().contains("kept for later"));
    let mut b = app(&root);
    b.open_panel();
    b.focus = Focus::Claude;
    key(&mut b, KeyCode::Up);
    assert_eq!(input(&b), "kept for later");
    let _ = std::fs::remove_dir_all(&root);
}

// HISTORY-7, HISTORY-5: a pasted text chip goes into the history; `up` brings it back as a chip
// with its text, and the text goes again as context.
#[tokio::test]
async fn up_brings_a_text_chip_back_as_a_chip() {
    let root = repo("history-chips", &[(A, "# A\n")]);
    let mut a = app(&root);
    a.open_panel();
    a.focus = Focus::Claude;
    typ(&mut a, "why ");
    a.on_paste("fn main() {\n    panic!(\"no\");\n}\n".into());
    let label = a.composer().chips()[0].label.clone();
    send(&mut a, "now").await;
    assert!(a.composer().chips().is_empty());
    key(&mut a, KeyCode::Up);
    assert_eq!(input(&a), format!("why {label} now"));
    let chips = a.composer().chips();
    assert_eq!(chips.len(), 1, "the chip is a chip again");
    assert!(chips[0].context.as_deref().is_some_and(|c| c.contains("    panic!(\"no\");")), "{chips:?}");
    let _ = std::fs::remove_dir_all(&root);
}

// HISTORY-2: `/clear` starts a new conversation, as `ctrl-n` does: the history of the session goes
// into the common history.
#[tokio::test]
async fn clear_merges_the_session_into_the_common_history() {
    let root = repo("history-clear", &[(A, "# A\n")]);
    let mut a = app(&root);
    a.switch_session(0);
    send(&mut a, "before clear").await;
    send(&mut a, "/clear").await;
    a.switch_session(1);
    key(&mut a, KeyCode::Up);
    assert_eq!(input(&a), "/clear");
    key(&mut a, KeyCode::Up);
    assert_eq!(input(&a), "before clear");
    let _ = std::fs::remove_dir_all(&root);
}
