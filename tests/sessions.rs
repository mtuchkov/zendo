//! Claude Code sessions in terminals that zendo owns (docs/zendo/sessions.md), with the mock command
//! line `zendo-mock-cli` in the place of `claude`.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};
use zendo::app::{App, Focus, Item, session_key};
use zendo::session::{Command, State};

fn mock() -> Command {
    Command { program: env!("CARGO_BIN_EXE_zendo-mock-cli").into(), args: vec!["--resume".into()] }
}

/// An app with the mock command line, and agent sessions 1 and 2 with session IDs.
fn app_with_cli(name: &str) -> (std::path::PathBuf, App) {
    let root = repo(name, &[("docs/notes/2026-11-12-a.md", "# A\n\n- one\n")]);
    let mut a = app(&root);
    a.cli.command = Some(mock());
    a.thread(&session_key(0)).session = Some("s1".into());
    a.thread(&session_key(1)).session = Some("s2".into());
    a.open_panel();
    (root, a)
}

/// Tick until `done` holds, at most `secs` seconds.
fn wait(a: &mut App, secs: u64, done: impl Fn(&App) -> bool) {
    let end = Instant::now() + Duration::from_secs(secs);
    while !done(a) {
        assert!(Instant::now() < end, "timeout");
        a.tick();
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The last counter on the screen of Claude Code session `n`.
fn counter(a: &App, n: usize) -> u64 {
    a.cli.text(n).lines().filter_map(|l| l.trim().strip_prefix("tick ")?.parse().ok()).max().unwrap_or(0)
}

fn leave(a: &mut App) {
    a.on_key(KeyEvent::new(KeyCode::Char(']'), KeyModifiers::CONTROL));
}

/// The chat, focused, on agent session `n`.
fn chat_on(a: &mut App, n: usize) {
    a.switch_session(n);
    a.focus = Focus::Claude;
}

// SESSIONS-1, SESSIONS-2, SESSIONS-3, SESSIONS-7, SESSIONS-8, SESSIONS-10: two Claude Code sessions
// keep running when the user leaves them, in terminals that zendo owns.
#[tokio::test(flavor = "multi_thread")]
async fn left_sessions_keep_running() {
    let (root, mut a) = app_with_cli("sessions-run");
    chat_on(&mut a, 0);
    ctrl(&mut a, 'o');
    assert_eq!(a.cli.view, Some(0), "ctrl-o shows the Claude Code session");
    wait(&mut a, 10, |a| counter(a, 0) >= 2);
    assert!(a.cli.text(0).contains("--resume s1"), "started with the session ID: {}", a.cli.text(0));
    leave(&mut a);
    assert_eq!(a.cli.view, None, "ctrl-] goes back to zendo");
    chat_on(&mut a, 1);
    ctrl(&mut a, 'o');
    assert_eq!(a.cli.view, Some(1));
    wait(&mut a, 10, |a| counter(a, 1) >= 2);
    leave(&mut a);
    let (c0, c1) = (counter(&a, 0), counter(&a, 1));
    std::thread::sleep(Duration::from_millis(600));
    a.tick();
    assert!(counter(&a, 0) > c0 && counter(&a, 1) > c1, "both ran while the user was away");
    assert!(a.cli.running(0) && a.cli.running(1));
    assert_ne!(a.cli.pid(0), a.cli.pid(1), "two processes");
    #[cfg(target_os = "linux")]
    for n in 0..2 {
        let stat = std::fs::read_to_string(format!("/proc/{}/stat", a.cli.pid(n).unwrap())).unwrap();
        let ppid: u32 = stat.rsplit(')').next().unwrap().split_whitespace().nth(1).unwrap().parse().unwrap();
        assert_eq!(ppid, std::process::id(), "zendo itself runs the process, no multiplexer");
    }
    chat_on(&mut a, 0);
    ctrl(&mut a, 'o');
    let last = counter(&a, 0);
    let screen = draw(&mut a, 80, 24).join("\n");
    assert!(screen.contains(&format!("tick {last}")), "the view shows the screen as it is now:\n{screen}");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// SESSIONS-4: while a Claude Code session runs, its chat sends no question.
#[tokio::test(flavor = "multi_thread")]
async fn chat_of_an_open_session_sends_no_question() {
    let (root, mut a) = app_with_cli("sessions-one-writer");
    chat_on(&mut a, 0);
    ctrl(&mut a, 'o');
    leave(&mut a);
    a.focus = Focus::Claude;
    typ(&mut a, "hello");
    key(&mut a, KeyCode::Enter);
    let t = &a.threads[&session_key(0)];
    assert!(t.busy.is_none(), "no prompt went to the agent");
    assert!(!t.items.iter().any(|i| matches!(i, Item::User(_))), "no question in the thread");
    assert!(t.items.iter().any(|i| matches!(i, Item::Info(s) if s.contains("Claude Code"))), "the chat says why");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// SESSIONS-5, SESSIONS-14: a new state shows on the session button within 1 s.
#[tokio::test(flavor = "multi_thread")]
async fn state_shows_within_a_second() {
    let (root, mut a) = app_with_cli("sessions-state");
    chat_on(&mut a, 0);
    ctrl(&mut a, 'o');
    wait(&mut a, 10, |a| counter(a, 0) >= 1);
    typ(&mut a, "w");
    leave(&mut a);
    let from = Instant::now();
    wait(&mut a, 5, |a| a.cli.state(0) == Some(State::Waits));
    assert!(from.elapsed() < Duration::from_secs(1), "took {:?}", from.elapsed());
    let screen = draw(&mut a, 160, 30).join("\n");
    assert!(screen.contains("F6!"), "the button of session 1 shows waits:\n{screen}");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// SESSIONS-6, SESSIONS-19: quit asks first while sessions run; quit again stops them.
#[tokio::test(flavor = "multi_thread")]
async fn quit_asks_while_sessions_run() {
    let (root, mut a) = app_with_cli("sessions-quit");
    chat_on(&mut a, 0);
    ctrl(&mut a, 'o');
    leave(&mut a);
    ctrl(&mut a, 'q');
    assert!(!a.settle_quit(), "the first quit asks");
    assert!(a.flash.as_ref().is_some_and(|(s, _)| s.contains("Claude Code")), "{:?}", a.flash);
    assert!(a.cli.running(0), "the session runs");
    ctrl(&mut a, 'q');
    assert!(a.settle_quit(), "the second quit quits");
    assert!(!a.cli.running(0), "the session stopped");
    let _ = std::fs::remove_dir_all(&root);
}

// SESSIONS-12, SESSIONS-13: when Claude Code exits, zendo goes back and opens the agent session again.
#[tokio::test(flavor = "multi_thread")]
async fn exit_goes_back_and_opens_the_agent_session_again() {
    let (root, mut a) = app_with_cli("sessions-exit");
    chat_on(&mut a, 0);
    ctrl(&mut a, 'o');
    wait(&mut a, 10, |a| counter(a, 0) >= 1);
    typ(&mut a, "q");
    wait(&mut a, 5, |a| a.cli.view.is_none());
    assert!(!a.cli.running(0));
    assert!(a.threads.get(&session_key(0)).is_none_or(|t| t.opened && t.items.is_empty()), "the thread opens again");
    let _ = std::fs::remove_dir_all(&root);
}

// SESSIONS-15: no Claude Code session while the chat answers.
#[tokio::test(flavor = "multi_thread")]
async fn no_session_while_the_chat_answers() {
    let (root, mut a) = app_with_cli("sessions-busy");
    chat_on(&mut a, 0);
    a.thread(&session_key(0)).busy = Some(Instant::now());
    ctrl(&mut a, 'o');
    assert_eq!(a.cli.view, None);
    assert!(!a.cli.running(0));
    assert!(a.flash.as_ref().is_some_and(|(s, _)| s.contains("answer")), "{:?}", a.flash);
    let _ = std::fs::remove_dir_all(&root);
}
