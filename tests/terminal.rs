//! The terminals of zendo (docs/zendo/terminal/terminal.md): shell blocks in the chat, `F5`, `alt-F5`, and the
//! `F5` and `[>]` terminals, with `/bin/sh` as the shell.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};
use zendo::app::{App, Focus, Item, SESSIONS, session_key};
use zendo::session::shell;

/// An app with `/bin/sh` for the terminals and the chat open on agent session 1.
fn app_with_shell(name: &str) -> (std::path::PathBuf, App) {
    let root = repo(name, &[("docs/notes/2026-11-12-a.md", "# A\n")]);
    let mut a = app(&root);
    a.cli.shell = Some("/bin/sh".into());
    a.open_panel();
    (root, a)
}

/// An answer of the agent with one ` ```sh ` block, and the cursor of the transcript in that block.
fn answer(a: &mut App, command: &str) {
    a.thread(&session_key(0)).items.push(Item::Agent(format!("Run this:\n\n```sh\n{command}\n```\n")));
    a.focus = Focus::Claude;
    a.transcript.on = true;
    draw(a, 120, 40);
    a.transcript.row = a.transcript.blocks.last().expect("a shell block").0 + 1;
}

fn f5(a: &mut App, alt: bool) {
    a.on_key(KeyEvent::new(KeyCode::F(5), if alt { KeyModifiers::ALT } else { KeyModifiers::NONE }));
}

fn alt(a: &mut App, c: char) {
    a.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT));
}

fn alt_q(a: &mut App) {
    alt(a, 'q');
}

/// The size of the screen of the shell of slot `n`, as `stty size` gives it: `rows cols`.
fn stty_size(a: &mut App, n: usize) -> String {
    a.cli.write(n, b"echo size=$(stty size)=\r");
    let tag = |a: &App| a.cli.text(n).rsplit("size=").next().and_then(|t| t.split_once('=')).map(|(v, _)| v.to_string());
    wait(a, 10, |a| a.cli.text(n).matches("size=").count() >= 2 && tag(a).is_some_and(|v| !v.contains('$')));
    tag(a).unwrap()
}

/// Tick until `done` holds, at most `secs` seconds.
fn wait(a: &mut App, secs: u64, done: impl Fn(&App) -> bool) {
    let end = Instant::now() + Duration::from_secs(secs);
    while !done(a) {
        assert!(Instant::now() < end, "timeout:\n{}", (0..10).map(|n| a.cli.text(n)).collect::<String>());
        a.tick();
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn done(root: &std::path::Path, a: &mut App) {
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(root);
}

// TERM-1, TERM-2, TERM-5, TERM-12: F5 puts the block into the shell of the F5 terminal and does not
// run it; enter runs it. zendo itself runs the shell.
#[tokio::test(flavor = "multi_thread")]
async fn f5_puts_the_block_into_the_shell_and_does_not_run_it() {
    let (root, mut a) = app_with_shell("term-f5");
    answer(&mut a, "touch ran");
    f5(&mut a, false);
    assert_eq!(a.cli.pane, Some(shell(0)), "the F5 terminal shows in the chat pane");
    wait(&mut a, 10, |a| a.cli.text(shell(0)).contains("touch ran"));
    std::thread::sleep(Duration::from_millis(300));
    assert!(!root.join("ran").exists(), "F5 does not run the command");
    #[cfg(target_os = "linux")]
    {
        let stat = std::fs::read_to_string(format!("/proc/{}/stat", a.cli.pid(shell(0)).unwrap())).unwrap();
        let ppid: u32 = stat.rsplit(')').next().unwrap().split_whitespace().nth(1).unwrap().parse().unwrap();
        assert_eq!(ppid, std::process::id(), "zendo itself runs the shell, no multiplexer");
    }
    key(&mut a, KeyCode::Enter);
    wait(&mut a, 10, |_| root.join("ran").exists());
    done(&root, &mut a);
}

// TERM-21: alt-F5 puts the block in and runs it.
#[tokio::test(flavor = "multi_thread")]
async fn alt_f5_runs_the_block() {
    let (root, mut a) = app_with_shell("term-alt-f5");
    answer(&mut a, "echo zendo-$((40+2))");
    f5(&mut a, true);
    wait(&mut a, 10, |a| a.cli.text(shell(0)).contains("zendo-42"));
    done(&root, &mut a);
}

// TERM-2: a shell block does not run by itself.
#[tokio::test(flavor = "multi_thread")]
async fn a_shell_block_does_not_run_by_itself() {
    let (root, mut a) = app_with_shell("term-no-run");
    answer(&mut a, "touch ran");
    key(&mut a, KeyCode::Tab);
    for _ in 0..10 {
        a.tick();
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(!root.join("ran").exists());
    assert!(!a.cli.running(shell(0)), "no terminal started");
    done(&root, &mut a);
}

// TERM-1, TERM-3, TERM-6, TERM-22: while the F5 terminal runs a command, a block goes to a new [>]
// terminal; alt-q leaves a terminal and its command runs on.
#[tokio::test(flavor = "multi_thread")]
async fn a_busy_f5_terminal_opens_a_new_one() {
    let (root, mut a) = app_with_shell("term-busy");
    answer(&mut a, "sleep 30");
    f5(&mut a, true);
    wait(&mut a, 10, |a| a.cli.busy(0));
    alt_q(&mut a);
    assert_eq!((a.cli.view, a.cli.pane), (None, None), "alt-q goes back to zendo");
    answer(&mut a, "echo two-$((1+1))");
    f5(&mut a, true);
    assert_eq!(a.cli.pane, Some(shell(1)), "a [>] terminal");
    wait(&mut a, 10, |a| a.cli.text(shell(1)).contains("two-2"));
    assert!(a.cli.busy(0), "sleep still runs");
    done(&root, &mut a);
}

// TERM-4, TERM-18: with five terminals that run a command, a block goes nowhere; quit asks first.
#[tokio::test(flavor = "multi_thread")]
async fn five_busy_terminals_take_nothing() {
    let (root, mut a) = app_with_shell("term-five");
    answer(&mut a, "sleep 30");
    for i in 0..5 {
        f5(&mut a, true);
        assert_eq!(a.cli.pane, Some(shell(i)));
        wait(&mut a, 10, |a| a.cli.busy(i));
        alt_q(&mut a);
    }
    answer(&mut a, "touch ran");
    f5(&mut a, true);
    assert_eq!(a.cli.pane, None, "no new terminal");
    assert!(a.flash.as_ref().is_some_and(|(s, _)| s.contains("terminals run")), "{:?}", a.flash);
    std::thread::sleep(Duration::from_millis(300));
    assert!(!root.join("ran").exists());
    ctrl(&mut a, 'q');
    assert!(!a.settle_quit(), "quit asks while terminals run commands");
    done(&root, &mut a);
}

// TERM-8, TERM-11, TERM-14, TERM-16: the play button on the block; F5 outside a block shows the F5
// terminal; the terminal buttons come under F10, after a line.
#[tokio::test(flavor = "multi_thread")]
async fn play_button_and_terminal_buttons() {
    let (root, mut a) = app_with_shell("term-buttons");
    answer(&mut a, "ls");
    let screen = draw(&mut a, 120, 40);
    assert!(screen.iter().any(|l| l.contains("▶")), "the play button:\n{}", screen.join("\n"));
    let col = |s: &str| screen.iter().position(|l| l.contains(s));
    let (f10, f5_row) = (col("F10").unwrap(), col(" F5 ").unwrap());
    assert!(f5_row > f10 + 2, "F5 comes under F10 with a gap");
    assert!(screen[f10 + 2].trim_end().ends_with("────"), "a line between them:\n{}", screen.join("\n"));
    a.focus = Focus::Note;
    a.transcript.on = false;
    f5(&mut a, false);
    assert_eq!(a.cli.pane, Some(SESSIONS), "F5 elsewhere shows the F5 terminal");
    a.cli.put(0, "sleep 30", true);
    wait(&mut a, 10, |a| a.cli.busy(0));
    alt_q(&mut a);
    let screen = draw(&mut a, 120, 40).join("\n");
    assert!(screen.contains("F5•"), "the busy mark:\n{screen}");
    done(&root, &mut a);
}

// TERM-37, TERM-40, PANES-33: a terminal shows in the chat pane, the note stays on the screen, the
// shell gets the size of the chat pane, and the badge of the pane is `SHELL`.
#[tokio::test(flavor = "multi_thread")]
async fn a_terminal_shows_in_the_chat_pane() {
    let (root, mut a) = app_with_shell("term-pane");
    answer(&mut a, "true");
    f5(&mut a, true);
    assert_eq!((a.cli.pane, a.cli.view), (Some(shell(0)), None), "the chat pane, not the full screen");
    let screen = draw(&mut a, 120, 40).join("\n");
    assert!(screen.contains("# A") || screen.contains(" A"), "the note is on the screen:\n{screen}");
    let bar = screen.lines().last().unwrap_or_default();
    assert!(bar.contains(" SHELL ") && !bar.contains(" CHAT "), "the badge of a shell: {bar}");
    let size = stty_size(&mut a, shell(0));
    let cols: u16 = size.split_whitespace().nth(1).unwrap().parse().unwrap();
    assert!(cols < 100, "the shell has the width of the chat pane, not of the screen: {size}");
    done(&root, &mut a);
}

// TERM-38, TERM-39, TERM-40, TERM-42: ctrl-o makes the terminal full screen; there ctrl-o goes to
// the shell; after the user leaves, the terminal shows in the chat pane again.
#[tokio::test(flavor = "multi_thread")]
async fn ctrl_o_makes_the_terminal_full_screen() {
    let (root, mut a) = app_with_shell("term-full");
    f5(&mut a, false);
    assert_eq!(a.cli.pane, Some(shell(0)));
    ctrl(&mut a, 'o');
    assert_eq!(a.cli.view, Some(shell(0)), "ctrl-o: the full screen");
    draw(&mut a, 120, 40);
    assert_eq!(stty_size(&mut a, shell(0)), "39 120", "the shell has the size of the screen");
    ctrl(&mut a, 'o');
    assert_eq!(a.cli.view, Some(shell(0)), "ctrl-o on the full screen goes to the shell");
    alt_q(&mut a);
    assert_eq!((a.cli.view, a.cli.pane), (None, None), "alt-q leaves the terminal");
    f5(&mut a, false);
    assert_eq!((a.cli.pane, a.cli.view), (Some(shell(0)), None), "the next time: the chat pane");
    let screen = draw(&mut a, 120, 40).join("\n");
    assert!(screen.contains(" A"), "the note is on the screen:\n{screen}");
    done(&root, &mut a);
}

// TERM-41, TERM-49: alt-] goes from the last agent session to F5, to each [>] terminal, then to
// agent session 1; alt-[ goes the same way back.
#[tokio::test(flavor = "multi_thread")]
async fn alt_bracket_goes_through_the_terminals() {
    let (root, mut a) = app_with_shell("term-alt-bracket");
    answer(&mut a, "sleep 30");
    f5(&mut a, true);
    wait(&mut a, 10, |a| a.cli.busy(0));
    answer(&mut a, "true");
    f5(&mut a, false);
    assert_eq!(a.cli.pane, Some(shell(1)), "a [>] terminal runs");
    a.switch_session(SESSIONS - 1);
    assert_eq!(a.cli.pane, None, "the chat shows agent session 5");
    alt(&mut a, ']');
    assert_eq!(a.cli.pane, Some(shell(0)), "after the last agent session: F5");
    alt(&mut a, ']');
    assert_eq!(a.cli.pane, Some(shell(1)), "then the [>] terminal");
    alt(&mut a, ']');
    assert_eq!((a.cli.pane, a.state.session), (None, 0), "then agent session 1");
    alt(&mut a, '[');
    assert_eq!(a.cli.pane, Some(shell(1)), "back: the [>] terminal");
    alt(&mut a, 'p');
    assert_eq!(a.cli.pane, Some(shell(0)), "alt-p is alt-[ too: F5");
    alt(&mut a, '[');
    assert_eq!((a.cli.pane, a.state.session), (None, SESSIONS - 1), "then agent session 5");
    done(&root, &mut a);
}
