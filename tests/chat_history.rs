//! The history of the messages of the user: per zendo session, the common history after `ctrl-n`,
//! and `up` and `down` in the chat input (docs/zendo/chat-history.md).

mod common;
use common::*;
use ratatui::crossterm::event::KeyCode;
use zendo::app::{App, Focus, Item, session_key};

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

/// The questions that zendo sent to the agent, each with its session ID, from the agent log. The
/// questions of the name model have no `Question:` line, and can come at any time.
fn prompts(root: &std::path::Path) -> Vec<(String, String)> {
    let text = std::fs::read_to_string(root.join(".zendo/agent.log")).unwrap_or_default();
    let sent = text.lines().filter_map(|l| l.split_once(" Stdin ")).filter_map(|(_, json)| serde_json::from_str::<serde_json::Value>(json).ok());
    sent.filter(|m| m["method"] == "session/prompt")
        .map(|m| {
            let blocks = m["params"]["prompt"].as_array().cloned().unwrap_or_default();
            let text: Vec<String> = blocks.iter().filter_map(|b| b["text"].as_str().map(str::to_string)).collect();
            (m["params"]["sessionId"].as_str().unwrap_or("").to_string(), text.join("\n"))
        })
        .filter(|(_, text)| text.contains("Question: "))
        .collect()
}

/// The session ID of the thread of the active agent session.
fn session_id(a: &App) -> Option<String> {
    a.threads.get(&a.key()).and_then(|t| t.session.clone())
}

// HISTORY-8, HISTORY-9: `/new <text>` starts a new thread and sends the text to it with the context
// of the chat input; `/new` alone starts a new thread and sends nothing. zendo does `/new` itself.
#[tokio::test]
async fn new_starts_a_thread_and_sends_its_text() {
    let root = repo("history-new", &[(A, "# A\n")]);
    let mut a = app(&root);
    a.open_panel();
    send(&mut a, "first question").await;
    let old = session_id(&a).expect("a session after the first question");
    a.composer_mut().set_context("CONTEXT OF THE TASK".into());
    send(&mut a, "/new second question").await;
    let new = session_id(&a).expect("a session for the new thread");
    assert_ne!(old, new, "a new thread");
    let p = prompts(&root);
    let last = p.last().expect("a prompt");
    assert_eq!(last.0, new);
    assert!(last.1.ends_with("Question: second question") && last.1.contains("CONTEXT OF THE TASK"), "{}", last.1);
    assert!(!p.iter().any(|(_, t)| t.contains("/new")), "zendo does `/new` itself: {p:?}");
    let before = prompts(&root).len();
    send(&mut a, "/new").await;
    assert_eq!(prompts(&root).len(), before, "`/new` alone sends nothing");
    assert_ne!(session_id(&a), Some(new), "`/new` alone starts a new thread");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// HISTORY-10: `/new` while the session answers waits for the end of the turn, then starts the new
// thread; the turn that runs keeps its thread.
#[tokio::test]
async fn new_waits_for_the_turn_end() {
    let root = repo("history-new-queue", &[(A, "# A\n")]);
    let mut a = app(&root);
    answering(&mut a);
    typ(&mut a, "/new after the turn");
    key(&mut a, KeyCode::Enter);
    let kept = a.threads.get(&a.key()).is_some_and(|t| t.busy.is_some() && t.queue.len() == 1 && t.items.iter().any(|i| matches!(i, Item::User(q) if q == "first question")));
    assert!(kept, "the turn keeps its thread, and `/new` waits");
    until(&mut a, |a| a.threads.values().all(|t| t.busy.is_none() && t.queue.is_empty())).await;
    let p = prompts(&root);
    let (first, last) = (&p[0], p.last().unwrap());
    assert!(last.1.ends_with("Question: after the turn"), "{}", last.1);
    assert_ne!(last.0, first.0, "the queued text goes to the new thread");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

/// The chat answers a question. The events of the agent reach the app only in `until`, so the
/// turn runs for the app until the test calls it: no slow turn of the mock agent is needed.
fn answering(a: &mut App) {
    a.open_panel();
    a.focus = Focus::Claude;
    typ(a, "first question");
    key(a, KeyCode::Enter);
    assert!(a.threads.get(&a.key()).is_some_and(|t| t.busy.is_some()), "the turn runs");
}

// HISTORY-10: a question in the queue behind `/new` is the first question of the new thread: it
// goes with the prompt of zendo, as the first question of the old thread did.
#[tokio::test]
async fn a_question_behind_a_queued_new_is_the_first_of_the_new_thread() {
    let root = repo("history-new-then-question", &[(A, "# A\n")]);
    let mut a = app(&root);
    a.open_file(root.join(A));
    answering(&mut a);
    typ(&mut a, "/new");
    key(&mut a, KeyCode::Enter);
    typ(&mut a, "what next?");
    key(&mut a, KeyCode::Enter);
    until(&mut a, |a| a.threads.values().all(|t| t.busy.is_none() && t.queue.is_empty())).await;
    let p = prompts(&root);
    let (first, last) = (&p[0], p.last().unwrap());
    assert!(last.1.ends_with("Question: what next?"), "{}", last.1);
    assert_ne!(last.0, first.0, "the question goes to the new thread");
    let prime = first.1.lines().next().expect("the prompt of zendo");
    assert!(last.1.starts_with(prime), "the new thread gets the prompt of zendo:\n{}", last.1);
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// HISTORY-10, CLAUDE-34: `/new` in the queue shows as the user typed it, and `esc` puts it back
// into the chat input with `/new`.
#[tokio::test]
async fn a_queued_new_goes_back_into_the_input_as_new() {
    let root = repo("history-new-back", &[(A, "# A\n")]);
    let mut a = app(&root);
    answering(&mut a);
    typ(&mut a, "/new after the turn");
    key(&mut a, KeyCode::Enter);
    let queued: Vec<String> = a.threads.get(&a.key()).map_or(Vec::new(), |t| t.queue.iter().map(|o| o.question.clone()).collect());
    assert_eq!(queued, ["/new after the turn"], "the queue shows the command");
    key(&mut a, KeyCode::Esc);
    assert_eq!(input(&a), "/new after the turn");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-28, HISTORY-11, SESSIONS-4: while Claude Code runs in the session, zendo cannot start a new
// thread: `/new` stays in the chat input, does not go to Claude Code, and zendo says why.
#[tokio::test]
async fn new_does_not_go_to_claude_code() {
    let root = repo("history-new-cli", &[(A, "# A\n")]);
    let mut a = app(&root);
    a.cli.command = Some(zendo::session::Command { program: env!("CARGO_BIN_EXE_zendo-mock-cli").into(), args: vec![], claude: true });
    a.thread(&session_key(0)).session = Some("s1".into());
    a.open_panel();
    a.focus = Focus::Claude;
    ctrl(&mut a, 'o');
    until(&mut a, |a| a.cli.running(0)).await;
    a.cli.view = None;
    a.focus = Focus::Claude;
    typ(&mut a, "/new a task");
    key(&mut a, KeyCode::Enter);
    assert_eq!(input(&a), "/new a task", "the command stays in the chat input");
    let said = a.threads.get(&session_key(0)).is_some_and(|t| t.items.iter().any(|i| matches!(i, Item::Info(s) if s.contains("open in Claude Code"))));
    assert!(said, "zendo says why");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}
