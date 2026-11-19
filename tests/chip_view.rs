//! The chips of the chat input before a question goes (docs/zendo/chat-chips.md).

mod common;
use common::*;
use ratatui::crossterm::event::KeyCode;
use zendo::app::{App, CHAT_KEY, Focus, Item};

const A: &str = "docs/notes/a.md";
const B: &str = "docs/notes/b.md";

fn files() -> Vec<(&'static str, &'static str)> {
    vec![(A, "# A\n\ntext\n"), (B, "# B\n\nmore\n")]
}

/// The text of the agent after the last question.
fn last_answer(a: &App) -> String {
    let items = &a.threads[CHAT_KEY].items;
    let from = items.iter().rposition(|i| matches!(i, Item::User(_))).unwrap_or(0);
    items[from..].iter().filter_map(|i| if let Item::Agent(s) = i { Some(s.as_str()) } else { None }).collect()
}

// CLAUDE-57: a question with the chip of a gone file goes, with its label and without the file.
#[tokio::test]
async fn a_red_chip_goes_without_its_file() {
    let root = repo("chip-red", &files());
    let mut a = app(&root);
    a.open_panel();
    a.focus = Focus::Claude;
    a.on_paste(root.join(B).display().to_string());
    std::fs::remove_file(root.join(B)).unwrap();
    typ(&mut a, "what is in it");
    key(&mut a, KeyCode::Enter);
    until(&mut a, |a| a.threads.get(CHAT_KEY).is_some_and(|t| t.busy.is_none() && !t.items.is_empty())).await;
    let answer = last_answer(&a);
    assert!(!answer.contains("file link"), "the gone file does not go: {answer}");
    let asked = a.threads[CHAT_KEY].items.iter().find_map(|i| if let Item::User(s) = i { Some(s.clone()) } else { None }).unwrap_or_default();
    assert!(asked.contains("⟦otes/b.md⟧"), "the label goes: {asked}");
    let _ = std::fs::remove_dir_all(&root);
}
