//! Flows of the chat: one thread for every file, the file that goes with a question, slash
//! commands, and the keys of the chat input.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use zendo::app::{App, CHAT_KEY, Focus, Item};

const A: &str = "docs/notes/2026-11-12-a.md";
const B: &str = "docs/notes/2026-11-12-b.md";

fn files() -> Vec<(&'static str, &'static str)> {
    vec![(A, "# A\n\n- one\n"), (B, "# B\n\n- two\n")]
}

/// The text of the agent after the last question.
fn last_answer(a: &App) -> String {
    let items = &a.threads[CHAT_KEY].items;
    let from = items.iter().rposition(|i| matches!(i, Item::User(_))).unwrap_or(0);
    items[from..].iter().filter_map(|i| if let Item::Agent(s) = i { Some(s.as_str()) } else { None }).collect()
}

/// Ask in the chat and wait until the turn ends; discard the proposal of the mock agent.
async fn ask(a: &mut App, q: &str) -> String {
    a.focus = Focus::Claude;
    typ(a, q);
    key(a, KeyCode::Enter);
    until(a, |a| a.threads.get(CHAT_KEY).is_some_and(|t| t.pending().is_some() || (t.busy.is_none() && !t.items.is_empty()))).await;
    if a.threads[CHAT_KEY].pending().is_some() {
        key(a, KeyCode::Char('x'));
    }
    until(a, |a| a.threads[CHAT_KEY].busy.is_none()).await;
    last_answer(a)
}

// CLAUDE-5, CLAUDE-26: one thread for all files: the file on screen goes with a question when it did not go with one
// of the last 20 questions.
#[tokio::test]
async fn one_thread_for_every_file_and_the_file_goes_once() {
    let root = repo("chat-one-thread", &files());
    let mut a = app(&root);
    a.open_file(root.join(A));
    a.open_panel();
    let first = ask(&mut a, "what is in it").await;
    assert!(first.contains("file link") && first.contains("2026-11-12-a.md"), "the file goes with the first question: {first}");
    let again = ask(&mut a, "and now").await;
    assert!(!again.contains("file link"), "not again for the same file: {again}");
    a.open_file(root.join(B));
    a.open_panel();
    let other = ask(&mut a, "and this one").await;
    assert!(other.contains("2026-11-12-b.md"), "the other file goes with its first question: {other}");
    let users = a.threads[CHAT_KEY].items.iter().filter(|i| matches!(i, Item::User(_))).count();
    assert_eq!(users, 3, "one thread has all the questions");
    assert_eq!(a.threads.keys().filter(|k| !k.starts_with("(recap)")).count(), 1, "{:?}", a.threads.keys().collect::<Vec<_>>());
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-15: the status bar names alt-enter for a new line: a terminal without the kitty keyboard protocol
// sends shift-enter as enter.
#[tokio::test]
async fn alt_enter_is_a_new_line_and_the_status_bar_says_so() {
    let root = repo("chat-alt-enter", &files());
    let mut a = app(&root);
    a.open_panel();
    a.focus = Focus::Claude;
    let screen = draw(&mut a, 160, 30);
    let bar = screen.iter().rev().find(|r| r.contains("CLAUDE")).unwrap();
    assert!(bar.contains("alt-enter new line"), "{bar}");
    typ(&mut a, "one");
    a.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));
    typ(&mut a, "two");
    assert_eq!(a.input.text(), "one\ntwo");
    key(&mut a, KeyCode::Left);
    key(&mut a, KeyCode::Left);
    typ(&mut a, "x");
    assert_eq!(a.input.text(), "one\ntxwo", "left moves back one char");
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-28: slash commands pass through: the list comes from the agent, `/` shows it, `tab` takes one, and
// the command goes to the agent as typed.
#[tokio::test]
async fn slash_commands_come_from_the_agent_and_go_to_it_as_typed() {
    let root = repo("chat-slash", &files());
    let mut a = app(&root);
    a.open_file(root.join(A));
    a.open_panel();
    until(&mut a, |a| !a.slash.commands.is_empty()).await;
    a.focus = Focus::Claude;
    typ(&mut a, "/rev");
    assert!(a.slash.open, "`/` and a part of a name show the commands");
    let screen = draw(&mut a, 160, 30);
    assert!(screen.iter().any(|r| r.contains("/review") && r.contains("review the note")), "{screen:#?}");
    key(&mut a, KeyCode::Tab);
    assert_eq!(a.input.text(), "/review ");
    assert!(!a.slash.open);
    typ(&mut a, "the risks");
    key(&mut a, KeyCode::Enter);
    until(&mut a, |a| a.threads[CHAT_KEY].busy.is_none() && last_answer(a).contains("ran the command")).await;
    assert_eq!(last_answer(&a), "ran the command: /review the risks", "no context before the command");
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-27, CLAUDE-6: the chat shows the last session when it opens, before the first question.
#[tokio::test]
async fn the_chat_shows_the_last_session_when_it_opens() {
    let root = repo("chat-resume", &files());
    let mut a = app(&root);
    a.open_panel();
    until(&mut a, |a| a.threads.get(CHAT_KEY).is_some_and(|t| t.session.is_some())).await;
    drop(a);
    let mut b = app(&root);
    b.open_panel();
    until(&mut b, |b| b.threads.get(CHAT_KEY).is_some_and(|t| t.items.iter().any(|i| matches!(i, Item::Agent(s) if s.contains("replayed"))))).await;
    let screen = draw(&mut b, 160, 30);
    let row = |s: &str| screen.iter().position(|r| r.contains(s)).unwrap_or_else(|| panic!("{s}: {screen:#?}"));
    assert!(row("(earlier question)") < row("(earlier answer"), "the question comes before its answer: {screen:#?}");
    let _ = std::fs::remove_dir_all(&root);
}
