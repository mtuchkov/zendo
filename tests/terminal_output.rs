//! The result of the last command of a terminal (docs/zendo/terminal/terminal-output.md): `alt-o` in the input
//! of a chat, the OSC 133 marks of bash, and `/bin/sh` without marks.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use zendo::app::{App, Focus, Item, session_key};
use zendo::session::shell;

/// An app with `program` for the terminals, a home directory with `bashrc`, and the chat open.
fn app_with(name: &str, program: &str, bashrc: &str) -> (PathBuf, App) {
    let root = repo(name, &[("docs/notes/2026-11-12-a.md", "# A\n")]);
    let home = root.join(".home");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join(".bashrc"), bashrc).unwrap();
    let mut a = app(&root);
    a.cli.shell = Some(program.into());
    a.cli.home = Some(home);
    a.open_panel();
    (root, a)
}

fn alt(a: &mut App, c: char) {
    a.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT));
}

/// Tick until `done` holds, at most `secs` seconds.
fn wait(a: &mut App, secs: u64, done: impl Fn(&App) -> bool) {
    let end = Instant::now() + Duration::from_secs(secs);
    while !done(a) {
        assert!(Instant::now() < end, "timeout:\n{}", (5..10).map(|n| a.cli.text(n)).collect::<Vec<_>>().join("\n----\n"));
        a.tick();
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Run `command` in terminal `i` when its prompt is ready, and wait for its result.
fn run(a: &mut App, i: usize, command: &str) {
    a.cli.put(i, command, true);
    wait(a, 10, |a| a.cli.result(shell(i)).is_some_and(|o| o.done && o.command.contains(command)));
}

/// Back in the input of the chat of agent session 1.
fn to_input(a: &mut App) {
    alt(a, 'q');
    a.focus = Focus::Claude;
    a.transcript.on = false;
}

fn input(a: &App) -> String {
    a.input.lines.join("\n")
}

fn done(root: &Path, a: &mut App) {
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(root);
}

// TERM-23, TERM-24, TERM-25, TERM-26: alt-o puts the command, the exit code and the output into the
// input, without the prompt, and does not send it.
#[tokio::test(flavor = "multi_thread")]
async fn alt_o_puts_the_result_into_the_input_and_does_not_send_it() {
    let (root, mut a) = app_with("out-alt-o", "bash", "PS1='PROMPT> '\n");
    key(&mut a, KeyCode::F(5));
    run(&mut a, 0, "echo zendo-out; false");
    to_input(&mut a);
    alt(&mut a, 'o');
    let text = input(&a);
    assert!(text.contains("echo zendo-out; false") && text.contains("exit 1") && text.contains("zendo-out"), "{text}");
    assert!(!text.contains("PROMPT>"), "no prompt: {text}");
    let sent = a.threads.get(&session_key(0)).is_some_and(|t| t.items.iter().any(|i| matches!(i, Item::User(_))));
    assert!(!sent, "the agent got no message");
    done(&root, &mut a);
}

// TERM-27: a shell without marks gives the output, and the exit code is unknown.
#[tokio::test(flavor = "multi_thread")]
async fn a_shell_without_marks_gives_the_output_and_no_exit_code() {
    let (root, mut a) = app_with("out-sh", "/bin/sh", "");
    key(&mut a, KeyCode::F(5));
    run(&mut a, 0, "echo plain-out");
    to_input(&mut a);
    alt(&mut a, 'o');
    let text = input(&a);
    assert!(text.contains("plain-out") && text.contains("exit code unknown"), "{text}");
    done(&root, &mut a);
}

/// The F5 terminal ran `echo first` and a `[>]` terminal `echo second`; the user viewed `[>]`,
/// then `F5`, and is back in the input.
fn two_results(name: &str) -> (PathBuf, App) {
    let (root, mut a) = app_with(name, "/bin/sh", "");
    key(&mut a, KeyCode::F(5));
    a.cli.put(0, "sleep 1; echo first", true);
    wait(&mut a, 10, |a| a.cli.busy(0));
    a.put_into_terminal("echo second", true);
    assert_eq!(a.cli.pane, Some(shell(1)), "the [>] terminal");
    wait(&mut a, 10, |a| a.cli.result(shell(1)).is_some_and(|o| o.done && o.text.contains("second")));
    a.focus = Focus::Note;
    key(&mut a, KeyCode::F(5));
    assert_eq!(a.cli.pane, Some(shell(0)), "the F5 terminal");
    wait(&mut a, 10, |a| a.cli.result(shell(0)).is_some_and(|o| o.done && o.text.contains("first")));
    to_input(&mut a);
    (root, a)
}

// TERM-24: alt-o takes the terminal that the user viewed last.
#[tokio::test(flavor = "multi_thread")]
async fn alt_o_takes_the_terminal_viewed_last() {
    let (root, mut a) = two_results("out-last");
    alt(&mut a, 'o');
    let text = input(&a);
    assert!(text.contains("first") && !text.contains("second"), "{text}");
    done(&root, &mut a);
}

// TERM-28, TERM-29: alt-o again goes to the next terminal, and then from the start again; the
// terminal without a result does not count.
#[tokio::test(flavor = "multi_thread")]
async fn alt_o_again_goes_through_the_terminals() {
    let (root, mut a) = two_results("out-again");
    a.cli.open_shell(2, &root).unwrap();
    for want in ["first", "second", "first"] {
        alt(&mut a, 'o');
        let text = input(&a);
        assert!(text.contains(want) && text.matches("echo ").count() == 1, "{want}: {text}");
    }
    done(&root, &mut a);
}

// TERM-30: the result of a command that runs is the output so far, marked as not complete.
#[tokio::test(flavor = "multi_thread")]
async fn a_command_that_runs_gives_output_that_is_not_complete() {
    let (root, mut a) = app_with("out-runs", "/bin/sh", "");
    key(&mut a, KeyCode::F(5));
    a.cli.put(0, "echo part; sleep 30", true);
    wait(&mut a, 10, |a| a.cli.result(shell(0)).is_some_and(|o| o.text.contains("part")));
    to_input(&mut a);
    alt(&mut a, 'o');
    let text = input(&a);
    assert!(text.contains("part") && text.contains("not complete"), "{text}");
    done(&root, &mut a);
}

// TERM-31: without a result, alt-o puts nothing and tells why.
#[tokio::test(flavor = "multi_thread")]
async fn no_result_puts_nothing() {
    let (root, mut a) = app_with("out-none", "/bin/sh", "");
    a.focus = Focus::Claude;
    alt(&mut a, 'o');
    assert_eq!(input(&a), "");
    assert!(a.flash.as_ref().is_some_and(|(s, _)| s.contains("no terminal")), "{:?}", a.flash);
    done(&root, &mut a);
}

// TERM-32: of a long output, the last 500 lines, and how many lines were dropped.
#[tokio::test(flavor = "multi_thread")]
async fn a_long_output_keeps_the_last_500_lines() {
    let (root, mut a) = app_with("out-long", "bash", "");
    key(&mut a, KeyCode::F(5));
    run(&mut a, 0, "seq 1 600");
    to_input(&mut a);
    alt(&mut a, 'o');
    let text = input(&a);
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.contains(&"101") && lines.contains(&"600") && !lines.contains(&"100"), "{text}");
    assert!(text.contains("100 earlier lines dropped"), "{text}");
    done(&root, &mut a);
}

// TERM-34, TERM-35: bash reads the .bashrc of the user, the marks do not show, and the file of the
// user stays the same.
#[tokio::test(flavor = "multi_thread")]
async fn the_marks_do_not_change_the_files_of_the_user() {
    let rc = "export ZENDO_RC=from-rc\n";
    let (root, mut a) = app_with("out-rc", "bash", rc);
    key(&mut a, KeyCode::F(5));
    run(&mut a, 0, "echo rc=$ZENDO_RC");
    let o = a.cli.result(shell(0)).unwrap();
    assert_eq!((o.text.as_str(), o.exit), ("rc=from-rc", Some(0)), "the .bashrc of the user, and the marks");
    let screen = a.cli.text(shell(0));
    assert!(!screen.contains("133") && !screen.contains('\u{7}'), "no marks on the screen:\n{screen}");
    assert_eq!(std::fs::read_to_string(root.join(".home/.bashrc")).unwrap(), rc);
    done(&root, &mut a);
}
