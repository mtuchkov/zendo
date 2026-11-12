//! The queue of the chat: a question while the agent answers waits, and goes when the turn ends.

mod common;
use common::*;
use ratatui::crossterm::event::KeyCode;
use zendo::app::{App, CHAT_KEY, Focus, Item};

fn users(a: &App) -> usize {
    a.threads.get(CHAT_KEY).map_or(0, |t| t.items.iter().filter(|i| matches!(i, Item::User(_))).count())
}

/// A chat with no note open: the mock agent answers and proposes no edit.
fn chat(name: &str) -> (std::path::PathBuf, App) {
    let root = repo(name, &[("README.txt", "no notes\n")]);
    let mut a = app(&root);
    a.open_panel();
    a.focus = Focus::Claude;
    (root, a)
}

// CLAUDE-33: a question while the agent answers goes to the queue, shows there, and goes to the agent
// when the turn ends.
#[tokio::test]
async fn a_question_while_the_agent_answers_waits_in_the_queue() {
    let (root, mut a) = chat("queue-waits");
    typ(&mut a, "first");
    key(&mut a, KeyCode::Enter);
    typ(&mut a, "second");
    key(&mut a, KeyCode::Enter);
    assert_eq!(users(&a), 1, "the second question waits");
    assert_eq!(a.threads[CHAT_KEY].queue.len(), 1);
    assert_eq!(a.input.text(), "", "the input is free for the next one");
    let screen = draw(&mut a, 160, 30).join("\n");
    assert!(screen.contains("queued") && screen.contains("second"), "the queue shows:\n{screen}");
    until(&mut a, |a| users(a) == 2).await;
    until(&mut a, |a| a.threads[CHAT_KEY].busy.is_none()).await;
    assert!(a.threads[CHAT_KEY].queue.is_empty());
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-34: esc stops the answer and puts the queued questions back into the input.
#[tokio::test]
async fn esc_stops_and_puts_the_queue_back_in_the_input() {
    let (root, mut a) = chat("queue-esc");
    typ(&mut a, "first");
    key(&mut a, KeyCode::Enter);
    typ(&mut a, "second");
    key(&mut a, KeyCode::Enter);
    key(&mut a, KeyCode::Esc);
    assert_eq!(a.input.text(), "second", "the queued question is back in the input");
    assert!(a.threads[CHAT_KEY].queue.is_empty());
    until(&mut a, |a| a.threads[CHAT_KEY].busy.is_none()).await;
    assert_eq!(users(&a), 1, "nothing more went");
    let _ = std::fs::remove_dir_all(&root);
}
