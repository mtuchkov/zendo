//! The background processes of the agent and of the Claude Code sessions (docs/zendo/background.md).
//! The tests start real `sleep` processes in process groups of their own, as Claude Code does.
//! They run one at a time: zendo finds the processes under the process of the test, and the mock
//! agents of the tests of this file are all under it (`#[serial]`).

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use serial_test::serial;
use zendo::app::{App, Focus, session_key};
use zendo::session::Command;

/// Stop the `sleep` processes that an interrupted run of a test left behind.
fn clean(secs: &[&str]) {
    for p in zendo::procs::list().iter().filter(|p| secs.iter().any(|s| p.args == format!("sleep {s}"))) {
        zendo::procs::stop(p.pgid, true);
    }
}

async fn ask(a: &mut App, q: &str) {
    a.open_panel();
    a.focus = Focus::Claude;
    typ(a, q);
    key(a, KeyCode::Enter);
    until(a, |a| a.threads.get(&session_key(0)).is_some_and(|t| t.busy.is_none() && t.items.len() > 1)).await;
}

fn click(a: &mut App, column: u16, row: u16) {
    a.on_mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column, row, modifiers: KeyModifiers::NONE });
    a.on_mouse(MouseEvent { kind: MouseEventKind::Up(MouseButton::Left), column, row, modifiers: KeyModifiers::NONE });
}

fn screen(a: &mut App) -> String {
    draw(a, 400, 40).join("\n")
}

/// Scan until `done` holds, at most 5 s.
fn scan_until(a: &mut App, done: impl Fn(&App) -> bool) {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !done(a) && std::time::Instant::now() < end {
        a.scan_background_now();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

// BG-1, BG-2: a command that the agent leaves in the background shows in the background line of
// its chat after the turn, with its time.
#[tokio::test]
#[serial]
async fn a_background_process_shows_after_the_turn() {
    clean(&["6101"]);
    let root = repo("bg-after-turn", &[]);
    let mut a = app(&root);
    ask(&mut a, "background 6101").await;
    scan_until(&mut a, |a| a.bg.list.len() == 1);
    let s = screen(&mut a);
    assert!(s.contains("⏵ 1 running · sleep 6101 0 s ×"), "{s}");
    assert!(screen(&mut a).contains("sleep 6101"), "the next draw still shows it");
    clean(&["6101"]);
    let _ = std::fs::remove_dir_all(&root);
}

// BG-1, BG-2: a Claude Code session that starts a process in a group of its own shows it in the
// chat of its agent session.
#[tokio::test]
#[serial]
async fn a_claude_code_session_shows_its_background_process() {
    clean(&["6201"]);
    let root = repo("bg-cli", &[]);
    let mut a = app(&root);
    a.cli.command = Some(Command { program: env!("CARGO_BIN_EXE_zendo-mock-cli").into(), args: vec![] });
    a.thread(&session_key(0)).session = Some("s1".into());
    a.open_panel();
    a.focus = Focus::Claude;
    ctrl(&mut a, 'o');
    until(&mut a, |a| a.cli.running(0)).await;
    for c in "background 6201\r".chars() {
        a.cli.write(0, c.to_string().as_bytes());
    }
    a.cli.view = None;
    scan_until(&mut a, |a| a.bg.of(&session_key(0)).count() == 1);
    assert!(screen(&mut a).contains("sleep 6201"), "the chat of the session shows it");
    a.cli.stop_all();
    clean(&["6201"]);
    let _ = std::fs::remove_dir_all(&root);
}

// BG-3: `×` stops one process and the turn goes on; the end shows in the chat (BG-7).
#[tokio::test]
#[serial]
async fn one_process_stops_and_the_turn_goes_on() {
    clean(&["6301", "6302"]);
    let root = repo("bg-stop-one", &[]);
    let mut a = app(&root);
    a.open_panel();
    a.focus = Focus::Claude;
    typ(&mut a, "background 6301 6302 slow");
    key(&mut a, KeyCode::Enter);
    until(&mut a, |a| a.threads.get(&session_key(0)).is_some_and(|t| t.items.iter().any(|i| matches!(i, zendo::app::Item::Agent(s) if s.contains("parser"))))).await;
    scan_until(&mut a, |a| a.bg.list.len() == 2);
    screen(&mut a);
    let first = a.bg.list[0].pgid;
    let stop = a.frame.bg_buttons().iter().find(|(_, g, x)| *g == first && *x).map(|(r, ..)| *r).expect("a × for the first process");
    click(&mut a, stop.x, stop.y);
    scan_until(&mut a, |a| a.bg.list.len() == 1);
    assert_eq!(a.bg.list.len(), 1, "only the first one ended");
    assert_ne!(a.bg.list[0].pgid, first);
    assert!(a.threads.get(&session_key(0)).unwrap().busy.is_some(), "the turn still runs");
    assert!(screen(&mut a).contains("sleep 6301 ended · 0 s"));
    clean(&["6301", "6302"]);
    let _ = std::fs::remove_dir_all(&root);
}

// BG-4: the quit asks while a background process runs, and `enter` stops it.
#[tokio::test]
#[serial]
async fn quit_asks_about_background_processes() {
    clean(&["6401"]);
    let root = repo("bg-quit", &[]);
    let mut a = app(&root);
    ask(&mut a, "background 6401").await;
    scan_until(&mut a, |a| a.bg.list.len() == 1);
    a.quit = true;
    assert!(!a.settle_quit(), "the first quit asks");
    assert!(a.flash.as_ref().is_some_and(|(m, _)| m.contains("1 background process runs: quit again or enter stops it, esc leaves it running")), "{:?}", a.flash);
    key(&mut a, KeyCode::Enter);
    assert!(a.settle_quit(), "enter quits");
    let pgid = a.bg.list[0].pgid;
    let end = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while zendo::procs::alive(&zendo::procs::list(), pgid) && std::time::Instant::now() < end {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    assert!(!zendo::procs::alive(&zendo::procs::list(), pgid), "the quit stopped it");
    let _ = std::fs::remove_dir_all(&root);
}
