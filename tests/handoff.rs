//! `ctrl-o` while the chat answers (docs/zendo/handoff.md): the waiting session view, the stop, and
//! the start of Claude Code, with the mock agent and the mock command line `zendo-mock-cli`.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};
use zendo::app::{App, Focus, session_key};
use zendo::session::Command;

fn mock() -> Command {
    Command { program: env!("CARGO_BIN_EXE_zendo-mock-cli").into(), args: vec!["--resume".into()] }
}

/// The chat on agent session 1, with a question that the mock agent answers. The agent events do
/// not go to the app until a test calls `until`: the turn runs until then.
async fn asking(name: &str) -> (std::path::PathBuf, App) {
    let root = repo(name, &[("README.txt", "no notes\n")]);
    let mut a = app(&root);
    a.cli.command = Some(mock());
    a.switch_session(0);
    a.focus = Focus::Claude;
    typ(&mut a, "hello");
    key(&mut a, KeyCode::Enter);
    until(&mut a, |a| a.threads.get(&session_key(0)).is_some_and(|t| t.session.is_some())).await;
    assert!(answering(&a), "the turn runs");
    (root, a)
}

fn answering(a: &App) -> bool {
    a.threads.get(&session_key(0)).is_some_and(|t| t.busy.is_some())
}

/// The arguments that the mock command line printed first, also when the screen wrapped them.
fn args(a: &App) -> String {
    let text: String = a.cli.text(0).lines().collect();
    text.split_once("mock cli:").map(|(_, rest)| rest.split("tick ").next().unwrap_or(rest).to_string()).unwrap_or_default()
}

fn screen(a: &mut App) -> String {
    draw(a, 160, 30).join("\n")
}

fn done(root: &std::path::Path, a: &mut App) {
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(root);
}

// HANDOFF-1, HANDOFF-2, HANDOFF-3, HANDOFF-7: ctrl-o while the chat answers shows the session view
// at once, and Claude Code starts when the turn ends, without a first prompt.
#[tokio::test(flavor = "multi_thread")]
async fn ctrl_o_during_a_turn_waits_for_its_end() {
    let (root, mut a) = asking("handoff-wait").await;
    ctrl(&mut a, 'o');
    assert!(a.hold.is_some(), "the session view waits");
    assert!(!a.cli.running(0), "Claude Code does not start while the turn runs");
    let s = screen(&mut a);
    assert!(s.contains("Claude Code starts when the turn ends"), "the bottom line says why:\n{s}");
    assert!(s.contains("again: stop the turn"), "the bottom line shows the stop:\n{s}");
    until(&mut a, |a| a.cli.running(0)).await;
    assert_eq!(a.cli.view, Some(0), "Claude Code shows in the session view");
    assert!(a.hold.is_none());
    assert!(!answering(&a), "Claude Code started after the turn ended");
    until(&mut a, |a| !args(a).is_empty()).await;
    assert!(!args(&a).contains("Continue"), "no first prompt: {}", args(&a));
    done(&root, &mut a);
}

// HANDOFF-4, HANDOFF-5, HANDOFF-8: ctrl-o twice stops the turn; Claude Code starts with the prompt
// to continue after the turn ends.
#[tokio::test(flavor = "multi_thread")]
async fn ctrl_o_again_stops_the_turn_and_continues_in_claude_code() {
    let (root, mut a) = asking("handoff-stop").await;
    ctrl(&mut a, 'o');
    ctrl(&mut a, 'o');
    assert!(a.hold.as_ref().is_some_and(|h| h.stopped.is_some()), "the turn stops");
    assert!(!a.cli.running(0), "Claude Code waits for the end of the turn");
    let s = screen(&mut a);
    assert!(s.contains("stopping the turn"), "the bottom line shows the stop:\n{s}");
    until(&mut a, |a| a.cli.running(0)).await;
    until(&mut a, |a| !args(a).is_empty()).await;
    assert!(args(&a).contains("The last turn was stopped. Continue it."), "the prompt to continue: {}", args(&a));
    done(&root, &mut a);
}

// HANDOFF-6: if the stopped turn does not end within 5 s, Claude Code starts anyway, and zendo says so.
#[tokio::test(flavor = "multi_thread")]
async fn claude_code_starts_if_the_turn_does_not_stop() {
    let (root, mut a) = asking("handoff-late").await;
    ctrl(&mut a, 'o');
    ctrl(&mut a, 'o');
    // No agent event goes to the app: for zendo, the turn does not end.
    let from = Instant::now();
    while !a.cli.running(0) {
        assert!(from.elapsed() < Duration::from_secs(8), "Claude Code did not start");
        a.tick();
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(from.elapsed() >= Duration::from_millis(4500), "zendo waited for the turn");
    assert!(a.flash.as_ref().is_some_and(|(s, _)| s.contains("did not stop")), "{:?}", a.flash);
    let s = screen(&mut a);
    assert!(s.contains("did not stop"), "the bottom line says so:\n{s}");
    let end = Instant::now() + Duration::from_secs(5);
    while args(&a).is_empty() && Instant::now() < end {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(args(&a).contains("Continue it."), "the prompt to continue: {}", args(&a));
    done(&root, &mut a);
}

// HANDOFF-9, HANDOFF-10: the waiting view ignores other keys; alt-q leaves it, and then Claude
// Code does not start when the turn ends.
#[tokio::test(flavor = "multi_thread")]
async fn keys_in_the_waiting_view() {
    let (root, mut a) = asking("handoff-keys").await;
    ctrl(&mut a, 'o');
    typ(&mut a, "xyz");
    a.on_paste("pasted".into());
    assert_eq!(a.input.text(), "", "no key and no paste goes to the chat input");
    assert!(a.hold.is_some());
    a.on_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::ALT));
    assert!(a.hold.is_none(), "alt-q leaves the waiting view");
    until(&mut a, |a| !answering(a)).await;
    a.tick();
    assert!(!a.cli.running(0), "Claude Code does not start after the user left");
    done(&root, &mut a);
}

// HANDOFF-11: the queued questions of the chat go to the input, not to the agent, when the turn
// ends while the session view waits.
#[tokio::test(flavor = "multi_thread")]
async fn queued_questions_go_to_the_input() {
    let (root, mut a) = asking("handoff-queue").await;
    typ(&mut a, "second");
    key(&mut a, KeyCode::Enter);
    assert_eq!(a.threads[&session_key(0)].queue.len(), 1, "the second question waits in the queue");
    ctrl(&mut a, 'o');
    until(&mut a, |a| a.cli.running(0)).await;
    assert!(a.threads[&session_key(0)].queue.is_empty());
    assert!(!answering(&a), "the queued question did not go to the agent");
    assert_eq!(a.input.text(), "second", "the queued question is in the chat input");
    done(&root, &mut a);
}
