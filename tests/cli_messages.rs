//! The messages of a running Claude Code session in the chat of its agent session
//! (docs/zendo/cli-messages.md), with the mock command line `zendo-mock-cli`: on `enter` it writes
//! the inputs of the "prompt sent" and "stop" hooks to the file of the session.

use crate::common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};
use zendo::app::{App, Focus, Item, session_key};
use zendo::session::Command;

fn mock() -> Command {
    Command { program: env!("CARGO_BIN_EXE_zendo-mock-cli").into(), args: vec!["--resume".into()], claude: true }
}

/// The chat on agent session 1, with its Claude Code session running and on the full screen.
fn running(name: &str) -> (std::path::PathBuf, App) {
    let root = repo(name, &[("README.txt", "no notes\n")]);
    let mut a = app(&root);
    a.cli.command = Some(mock());
    a.thread(&session_key(0)).session = Some("s1".into());
    a.switch_session(0);
    a.focus = Focus::Claude;
    ctrl(&mut a, 'o');
    wait(&mut a, 10, |a| a.cli.text(0).contains("tick 1"));
    // The mock writes the input of the "session start" hook before `tick 1`: one more tick reads it
    // now. Read later, with the first turn, it would open the transcript after the steps of the turn.
    a.tick();
    (root, a)
}

/// Tick until `done` holds, at most `secs` seconds.
fn wait(a: &mut App, secs: u64, done: impl Fn(&App) -> bool) {
    let end = Instant::now() + Duration::from_secs(secs);
    while !done(a) {
        assert!(Instant::now() < end, "timeout: {:?}", a.threads.get(&session_key(0)).map(|t| &t.items));
        a.tick();
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn leave(a: &mut App) {
    a.on_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::ALT));
}

fn items(a: &App) -> Vec<Item> {
    a.threads.get(&session_key(0)).map(|t| t.items.clone()).unwrap_or_default()
}

fn users(a: &App, text: &str) -> usize {
    items(a).iter().filter(|i| matches!(i, Item::User(s) if s == text)).count()
}

fn answered(a: &App, text: &str) -> bool {
    items(a).iter().any(|i| matches!(i, Item::Agent(s) if s == text))
}

fn done(root: &std::path::Path, a: &mut App) {
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(root);
}

// CLIMSG-1, CLIMSG-2, CLIMSG-3: a prompt typed in Claude Code and its answer show in the chat after
// alt-q.
#[tokio::test(flavor = "multi_thread")]
async fn claude_code_messages_show_in_the_chat() {
    let (root, mut a) = running("climsg-show");
    typ(&mut a, "hello");
    key(&mut a, KeyCode::Enter);
    leave(&mut a);
    wait(&mut a, 5, |a| answered(a, "answer: hello"));
    assert_eq!(users(&a, "hello"), 1, "the prompt shows: {:?}", items(&a));
    let screen = draw(&mut a, 160, 30).join("\n");
    assert!(screen.contains("answer: hello"), "{screen}");
    done(&root, &mut a);
}

// CLIMSG-4: a question from the chat shows once, also when the hook reports it.
#[tokio::test(flavor = "multi_thread")]
async fn a_question_from_the_chat_shows_once() {
    let (root, mut a) = running("climsg-once");
    leave(&mut a);
    a.focus = Focus::Claude;
    typ(&mut a, "abc");
    key(&mut a, KeyCode::Enter);
    wait(&mut a, 5, |a| answered(a, "answer: abc"));
    assert_eq!(users(&a, "abc"), 1, "{:?}", items(&a));
    done(&root, &mut a);
}

// CLIMSG-7: when the Claude Code session ends, the history that the agent loads replaces its
// messages.
#[tokio::test(flavor = "multi_thread")]
async fn the_end_of_the_session_replaces_the_messages() {
    let (root, mut a) = running("climsg-end");
    typ(&mut a, "hello");
    key(&mut a, KeyCode::Enter);
    leave(&mut a);
    wait(&mut a, 5, |a| answered(a, "answer: hello"));
    a.cli.write(0, b"q");
    wait(&mut a, 5, |a| !a.cli.running(0));
    assert!(!answered(&a, "answer: hello"), "the messages go: {:?}", items(&a));
    done(&root, &mut a);
}

// CLIMSG-8, CLIMSG-9: the text between the tool calls and the tool calls of a turn show in the
// chat, in their order, and the final answer shows once.
#[tokio::test(flavor = "multi_thread")]
async fn the_steps_of_a_turn_show_in_the_chat() {
    let (root, mut a) = running("climsg-steps");
    typ(&mut a, "hello");
    key(&mut a, KeyCode::Enter);
    leave(&mut a);
    wait(&mut a, 5, |a| answered(a, "answer: hello"));
    let seen: Vec<String> = items(&a)
        .iter()
        .filter_map(|i| match i {
            Item::User(s) => Some(format!("user {s}")),
            Item::Agent(s) => Some(format!("agent {s}")),
            Item::Tools { uses, .. } => Some(format!("tools {}", uses.iter().map(|u| u.title.as_str()).collect::<Vec<_>>().join(", "))),
            _ => None,
        })
        .collect();
    assert_eq!(seen, ["user hello", "agent step: hello", "tools Bash: run hello", "agent answer: hello"]);
    done(&root, &mut a);
}

// CLIMSG-10: after `/clear` in Claude Code, the steps of the next turn still show.
#[tokio::test(flavor = "multi_thread")]
async fn the_steps_show_after_a_clear() {
    let (root, mut a) = running("climsg-clear");
    a.cli.write(0, &[0x0c]);
    typ(&mut a, "next");
    key(&mut a, KeyCode::Enter);
    leave(&mut a);
    wait(&mut a, 5, |a| answered(a, "answer: next"));
    assert!(answered(&a, "step: next"), "{:?}", items(&a));
    done(&root, &mut a);
}
