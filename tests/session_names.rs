//! The names of the agent sessions (docs/zendo/session-names/session-names.md), with the mock agent,
//! which names each conversation `mock`, `stub` or `fake`, keeps the name at the first check and
//! then renames it `renamed`, and the mock command line `zendo-mock-cli`.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};
use zendo::app::{App, Focus, Item, session_key};
use zendo::session::Command;

/// Ask in the chat of agent session `n` and wait until the turn ends.
async fn ask(a: &mut App, n: usize, q: &str) {
    if a.state.session != n || !a.panel {
        a.switch_session(n);
    }
    a.focus = Focus::Claude;
    typ(a, q);
    key(a, KeyCode::Enter);
    let key = session_key(n);
    until(a, |a| a.threads.get(&key).is_some_and(|t| t.busy.is_none() && t.items.iter().any(|i| matches!(i, Item::Agent(_))))).await;
}

/// One question in agent session `n`, and the name that comes after its answer.
async fn named(a: &mut App, n: usize) -> String {
    ask(a, n, "first question").await;
    until(a, |a| a.session_name(n).is_some()).await;
    a.session_name(n).unwrap()
}

/// The rows of the buttons of the agent sessions on the screen: from the row of `F6` to the header
/// of the chat.
fn buttons(a: &mut App, w: u16) -> String {
    let screen = draw(a, w, 30);
    let first = screen.iter().position(|r| r.contains(" F6")).unwrap_or_else(|| panic!("no buttons: {screen:#?}"));
    let header = screen.iter().position(|r| r.contains("CLAUDE") || r.contains("AGENT")).unwrap_or_else(|| panic!("no header: {screen:#?}"));
    assert!(first < header, "the buttons are above the header: {screen:#?}");
    screen[first..header].join("\n")
}

fn done(root: &std::path::Path, a: &mut App) {
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(root);
}

// NAMES-1, NAMES-5, NAMES-6, NAMES-9, CLAUDE-32: the first agent message names the session while
// the turn still runs; the name stands after the key in the row of buttons above the header of the
// chat; the terminal has none. The mock agent writes one message and then works for seconds on a
// "slow" question.
#[tokio::test(flavor = "multi_thread")]
async fn the_first_agent_message_names_the_session() {
    let root = repo("names-first", &[("README.txt", "no notes\n")]);
    let mut a = app(&root);
    a.switch_session(0);
    a.focus = Focus::Claude;
    assert!(!buttons(&mut a, 220).contains("F6 mock"));
    typ(&mut a, "a slow question");
    key(&mut a, KeyCode::Enter);
    until(&mut a, |a| a.session_name(0).is_some()).await;
    assert_eq!(a.session_name(0).as_deref(), Some("mock"));
    let t = &a.threads[&session_key(0)];
    assert!(t.busy.is_some(), "the turn still runs");
    assert!(t.items.iter().any(|i| matches!(i, Item::Agent(_))), "after the first agent message");
    let screen = draw(&mut a, 220, 30);
    let row = screen.iter().position(|r| r.contains("F6 mock")).unwrap_or_else(|| panic!("{screen:#?}"));
    assert!(screen[row].contains("F7") && screen[row].contains("F10"), "one row: {}", screen[row]);
    assert!(screen[row + 1].contains("CLAUDE") || screen[row + 1].contains("AGENT"), "above the header: {screen:#?}");
    assert!(screen.iter().any(|r| r.trim_end().ends_with("F5")), "the F5 button has no name: {screen:#?}");
    let users: Vec<&Item> = a.threads[&session_key(0)].items.iter().filter(|i| matches!(i, Item::User(_))).collect();
    assert_eq!(users.len(), 1, "the question for the name is not in the thread: {users:?}");
    done(&root, &mut a);
}

// NAMES-14, NAMES-3: after 200 more characters zendo asks whether the name still fits; the mock
// agent keeps it once, which doubles the thresholds, and renames it at the next check. The unit
// test `due_after_the_thresholds_doubled_by_each_kept_name` checks the doubled thresholds.
#[tokio::test(flavor = "multi_thread")]
async fn a_changed_topic_renames_the_session() {
    let root = repo("names-rename", &[("README.txt", "no notes\n")]);
    let mut a = app(&root);
    assert_eq!(named(&mut a, 0).await, "mock");
    let id = a.threads[&session_key(0)].session.clone().unwrap();
    ask(&mut a, 0, &"more about the topic ".repeat(10)).await;
    until(&mut a, |a| a.names_asked.get(&id).is_some_and(|c| c.keeps == 1) && a.work.free()).await;
    assert_eq!(a.session_name(0).as_deref(), Some("mock"), "the first check keeps the name");
    ask(&mut a, 0, &"a new topic ".repeat(30)).await;
    until(&mut a, |a| a.session_name(0).as_deref() == Some("renamed")).await;
    assert!(buttons(&mut a, 220).contains("F6 renamed"));
    done(&root, &mut a);
}

// NAMES-2: two agent sessions never get the same name.
#[tokio::test(flavor = "multi_thread")]
async fn two_sessions_get_two_names() {
    let root = repo("names-two", &[("README.txt", "no notes\n")]);
    let mut a = app(&root);
    assert_eq!(named(&mut a, 0).await, "mock");
    assert_eq!(named(&mut a, 1).await, "stub");
    let row = buttons(&mut a, 220);
    assert!(row.contains("F6 mock") && row.contains("F7 stub"), "{row}");
    done(&root, &mut a);
}

// NAMES-3: the name stays with its Claude session after zendo starts again.
#[tokio::test(flavor = "multi_thread")]
async fn the_name_stays_after_a_restart() {
    let root = repo("names-restart", &[("README.txt", "no notes\n")]);
    let mut a = app(&root);
    named(&mut a, 0).await;
    drop(a);
    let mut b = app(&root);
    assert_eq!(b.session_name(0).as_deref(), Some("mock"));
    b.open_panel();
    assert!(buttons(&mut b, 220).contains("F6 mock"));
    done(&root, &mut b);
}

// NAMES-4: a new thread in the chat is a new Claude session: the name goes away.
#[tokio::test(flavor = "multi_thread")]
async fn a_new_thread_clears_the_name() {
    let root = repo("names-thread", &[("README.txt", "no notes\n")]);
    let mut a = app(&root);
    named(&mut a, 0).await;
    a.focus = Focus::Claude;
    ctrl(&mut a, 'n');
    assert_eq!(a.session_name(0), None);
    assert!(!buttons(&mut a, 220).contains("F6 mock"));
    done(&root, &mut a);
}

fn mock_cli() -> Command {
    Command { program: env!("CARGO_BIN_EXE_zendo-mock-cli").into(), args: vec!["--resume".into()] }
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

/// Agent session 1 with a name, and its Claude Code session running on the full screen.
async fn named_in_claude_code(name: &str) -> (std::path::PathBuf, App) {
    let root = repo(name, &[("README.txt", "no notes\n")]);
    let mut a = app(&root);
    named(&mut a, 0).await;
    a.cli.command = Some(mock_cli());
    a.focus = Focus::Claude;
    ctrl(&mut a, 'o');
    wait(&mut a, 10, |a| a.cli.text(0).contains("tick 1"));
    (root, a)
}

// NAMES-4: `/clear` in Claude Code starts a new Claude session: within 1 s the name goes away.
#[tokio::test(flavor = "multi_thread")]
async fn a_new_claude_session_clears_the_name() {
    let (root, mut a) = named_in_claude_code("names-clear").await;
    // ctrl-l: the mock command line reports the start of a session after `/clear`.
    a.on_key(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::CONTROL));
    wait(&mut a, 1, |a| a.session_name(0).is_none());
    a.on_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::ALT));
    assert!(!buttons(&mut a, 220).contains("F6 mock"));
    done(&root, &mut a);
}

// NAMES-6, NAMES-7: Claude Code continues the session: the name stays, and the bottom line of the
// session view shows it after the key.
#[tokio::test(flavor = "multi_thread")]
async fn the_bottom_line_shows_the_name() {
    let (root, mut a) = named_in_claude_code("names-bottom").await;
    for _ in 0..10 {
        a.tick();
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(a.session_name(0).as_deref(), Some("mock"));
    let screen = draw(&mut a, 160, 30);
    assert!(screen[29].contains("F6 mock"), "{}", screen[29]);
    done(&root, &mut a);
}

/// True when the buttons of the agent sessions stand on the right edge, one under the other, with
/// their keys alone, as the terminal buttons.
fn on_the_side(screen: &[String]) -> bool {
    let row = |k: &str| screen.iter().position(|r| r.trim_end().ends_with(&format!(" {k}")));
    matches!((row("F6"), row("F10")), (Some(a), Some(b)) if b == a + 8)
}

// NAMES-8, CLAUDE-32: in a narrow chat the names get shorter; when the row does not fit even then,
// the buttons stand on the right edge, one under the other, with their keys alone.
#[tokio::test(flavor = "multi_thread")]
async fn narrow_chats_shorten_the_names_and_then_put_the_buttons_on_the_side() {
    let root = repo("names-narrow", &[("README.txt", "no notes\n")]);
    let mut a = app(&root);
    let agent = a.cfg.agent.clone();
    for n in 0..5 {
        let id = format!("id-{n}");
        a.state.set_session(&agent, &session_key(n), id.clone());
        a.state.names.insert(id, format!("abcdefghijk{n}"));
    }
    a.switch_session(0);
    a.state.widths.insert("chat".into(), 100);
    let wide = buttons(&mut a, 220);
    assert!(!wide.contains('\n') && wide.contains("F6 abcdefghijk0") && wide.contains("F10 abcdefghijk4"), "one row: {wide}");
    a.state.widths.insert("chat".into(), 60);
    let narrow = buttons(&mut a, 160);
    assert!(narrow.contains("F6 abc") && !narrow.contains("abcdefghijk0") && narrow.contains("F10 abc"), "shorter: {narrow}");
    a.state.widths.insert("chat".into(), 40);
    let screen = draw(&mut a, 160, 30);
    assert!(on_the_side(&screen) && !screen.join("\n").contains("F6 abc"), "on the side: {screen:#?}");
    done(&root, &mut a);
}

// CLAUDE-32, TERM-14: while a terminal shows in the chat pane, the buttons stand on the right edge,
// one under the other, above the terminal buttons.
#[tokio::test(flavor = "multi_thread")]
async fn a_terminal_in_the_chat_pane_puts_the_buttons_on_the_side() {
    let root = repo("names-terminal", &[("README.txt", "no notes\n")]);
    let mut a = app(&root);
    a.cli.shell = Some("/bin/sh".into());
    a.open_panel();
    a.focus = Focus::Note;
    key(&mut a, KeyCode::F(5));
    assert!(a.cli.pane.is_some(), "the terminal is in the chat pane");
    let screen = draw(&mut a, 160, 30);
    assert!(on_the_side(&screen), "{screen:#?}");
    let f5 = screen.iter().position(|r| r.trim_end().ends_with(" F5")).unwrap();
    assert!(screen[f5 - 2].trim_end().ends_with("────"), "a line above the terminal buttons: {screen:#?}");
    done(&root, &mut a);
}

// NAMES-12, NAMES-13: the model of the config names the session; a model that the agent does not
// have leaves the default model. The mock agent names with its model `mock-light` "light".
#[tokio::test(flavor = "multi_thread")]
async fn a_light_model_names_the_session() {
    let root = repo("names-model", &[("README.txt", "no notes\n")]);
    let mut a = app(&root);
    a.name_model = Some("light".into());
    assert_eq!(named(&mut a, 0).await, "light");
    a.name_model = Some("no-such-model".into());
    assert_eq!(named(&mut a, 1).await, "mock");
    done(&root, &mut a);
}
