//! The mouse in a terminal (docs/zendo/terminal/terminal-mouse.md): the wheel and the selection, in the chat
//! pane and on the full screen, with `/bin/sh` as the shell.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::time::{Duration, Instant};
use zendo::app::{App, Focus};
use zendo::session::shell;

const W: u16 = 160;
const H: u16 = 40;

fn mouse(a: &mut App, kind: MouseEventKind, column: u16, row: u16) {
    a.on_mouse(MouseEvent { kind, column, row, modifiers: KeyModifiers::NONE });
}

fn wait(a: &mut App, done: impl Fn(&App) -> bool) {
    let end = Instant::now() + Duration::from_secs(10);
    while !done(a) {
        assert!(Instant::now() < end, "timeout:\n{}", a.cli.text(shell(0)));
        a.tick();
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The F5 terminal, in the chat pane or on the full screen (`full`).
fn open(name: &str, full: bool) -> (std::path::PathBuf, App) {
    let root = repo(name, &[("docs/notes/2026-11-12-a.md", "# A\n")]);
    let mut a = app(&root);
    a.cli.shell = Some("/bin/sh".into());
    a.open_panel();
    a.on_key(KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE));
    draw(&mut a, W, H);
    if full {
        ctrl(&mut a, 'o');
        draw(&mut a, W, H);
    }
    (root, a)
}

/// The F5 terminal with 200 numbered lines printed.
fn terminal(name: &str, full: bool) -> (std::path::PathBuf, App) {
    let (root, mut a) = open(name, full);
    a.cli.write(shell(0), b"seq -f 'line-%g' 1 200\r");
    wait(&mut a, |a| a.cli.text(shell(0)).contains("line-200"));
    std::thread::sleep(Duration::from_millis(200));
    (root, a)
}

/// The F5 terminal with a program that prints `MARK` in its top left corner, sends `modes` to the
/// terminal, and then prints each byte that it gets (`cat -v`). The cell of `MARK` on the screen.
fn program(name: &str, full: bool, modes: &str) -> (std::path::PathBuf, App, (u16, u16)) {
    let (root, mut a) = open(name, full);
    let command = format!("printf '{modes}\\033[2J\\033[HMARK'; stty -icanon -echo; cat -v\r");
    a.cli.write(shell(0), command.as_bytes());
    wait(&mut a, |a| a.cli.text(shell(0)).starts_with("MARK"));
    std::thread::sleep(Duration::from_millis(400));
    let at = find(&draw(&mut a, W, H), "MARK");
    (root, a, at)
}

/// What the program got: the text on its screen after `MARK`, when it has `want` in it.
fn got(a: &mut App, want: &str) -> String {
    let end = Instant::now() + Duration::from_secs(3);
    let text = |a: &App| a.cli.text(shell(0)).replace('\n', "");
    while !text(a).contains(want) && Instant::now() < end {
        std::thread::sleep(Duration::from_millis(20));
    }
    text(a)
}

fn done(root: std::path::PathBuf, mut a: App) {
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(root);
}

/// The cell of the first char of `text` on the screen.
fn find(screen: &[String], text: &str) -> (u16, u16) {
    let (y, row) = screen.iter().enumerate().find(|(_, r)| r.contains(text)).unwrap_or_else(|| panic!("no {text}:\n{}", screen.join("\n")));
    let x = row.chars().collect::<Vec<_>>().windows(text.chars().count()).position(|w| w.iter().collect::<String>() == text).unwrap();
    (x as u16, y as u16)
}

fn wheel_shows_earlier_lines(name: &str, full: bool) {
    let (root, mut a) = terminal(name, full);
    let screen = draw(&mut a, W, H);
    let (x, y) = find(&screen, "line-200");
    assert!(!screen.iter().any(|r| r.contains("line-130")));
    for _ in 0..20 {
        mouse(&mut a, MouseEventKind::ScrollUp, x, y);
        std::thread::sleep(Duration::from_millis(50));
    }
    let screen = draw(&mut a, W, H);
    assert!(screen.iter().any(|r| r.contains("line-130")), "the wheel shows earlier lines:\n{}", screen.join("\n"));
    done(root, a);
}

fn drag_copies(name: &str, full: bool) {
    let (root, mut a) = terminal(name, full);
    let screen = draw(&mut a, W, H);
    let (x, y) = find(&screen, "line-199");
    mouse(&mut a, MouseEventKind::Down(MouseButton::Left), x, y);
    mouse(&mut a, MouseEventKind::Drag(MouseButton::Left), x + 7, y);
    let screen = draw(&mut a, W, H);
    assert!(screen.iter().any(|r| r.contains("line-199")), "the terminal stays on the screen");
    mouse(&mut a, MouseEventKind::Up(MouseButton::Left), x + 7, y);
    draw(&mut a, W, H);
    assert_eq!(a.clip.as_deref(), Some("line-199"), "the release copies the selection");
    done(root, a);
}

/// SGR mouse reports: press and release (1000), drag (1002), in the SGR encoding (1006).
const REPORTS: &str = "\\033[?1000h\\033[?1002h\\033[?1006h";

fn the_program_gets_the_mouse(name: &str, full: bool) {
    let (root, mut a, (x, y)) = program(name, full, REPORTS);
    a.focus = Focus::Note;
    mouse(&mut a, MouseEventKind::Down(MouseButton::Left), x, y);
    assert!(full || a.focus == Focus::Claude, "TERM-47: the press gives the keys to the terminal in the chat pane");
    mouse(&mut a, MouseEventKind::Drag(MouseButton::Left), x + 2, y + 1);
    mouse(&mut a, MouseEventKind::Up(MouseButton::Left), x + 2, y + 1);
    mouse(&mut a, MouseEventKind::ScrollUp, x + 1, y);
    mouse(&mut a, MouseEventKind::ScrollDown, x + 1, y);
    let want = "^[[<0;1;1M^[[<32;3;2M^[[<0;3;2m^[[<64;2;1M^[[<65;2;1M";
    assert!(got(&mut a, want).contains(want), "the program gets the press, the drag, the release and the wheel:\n{}", got(&mut a, want));
    draw(&mut a, W, H);
    assert!(a.sel.is_none() && a.clip.is_none(), "zendo does not select text itself");
    done(root, a);
}

fn the_wheel_sends_arrows(name: &str, full: bool) {
    let (root, mut a, (x, y)) = program(name, full, "\\033[?1049h");
    mouse(&mut a, MouseEventKind::ScrollUp, x, y);
    assert!(got(&mut a, "^[[A^[[A^[[A").contains("^[[A^[[A^[[A"), "the wheel up is arrow up, three lines:\n{}", got(&mut a, "^[[A"));
    std::thread::sleep(Duration::from_millis(60));
    mouse(&mut a, MouseEventKind::ScrollDown, x, y);
    assert!(got(&mut a, "^[[B^[[B^[[B").contains("^[[B^[[B^[[B"), "the wheel down is arrow down, three lines:\n{}", got(&mut a, "^[[B"));
    done(root, a);
}

// TERM-43, TERM-47: a program that reports the mouse gets the mouse events, in the chat pane.
#[tokio::test(flavor = "multi_thread")]
async fn a_program_in_the_chat_pane_gets_the_mouse() {
    the_program_gets_the_mouse("term-mouse-pane", false);
}

// TERM-43: and on the full screen.
#[tokio::test(flavor = "multi_thread")]
async fn a_program_on_the_full_screen_gets_the_mouse() {
    the_program_gets_the_mouse("term-mouse-full", true);
}

// TERM-44: the wheel over a program on the alternate screen sends arrow keys, in the chat pane.
#[tokio::test(flavor = "multi_thread")]
async fn the_wheel_scrolls_a_program_in_the_chat_pane() {
    the_wheel_sends_arrows("term-arrows-pane", false);
}

// TERM-44: and on the full screen.
#[tokio::test(flavor = "multi_thread")]
async fn the_wheel_scrolls_a_program_on_the_full_screen() {
    the_wheel_sends_arrows("term-arrows-full", true);
}

// TERM-45: the wheel over a shell shows the lines that left the screen.
#[tokio::test(flavor = "multi_thread")]
async fn the_wheel_scrolls_a_terminal_in_the_chat_pane() {
    wheel_shows_earlier_lines("term-wheel-pane", false);
}

// TERM-45: on the full screen.
#[tokio::test(flavor = "multi_thread")]
async fn the_wheel_scrolls_a_terminal_on_the_full_screen() {
    wheel_shows_earlier_lines("term-wheel-full", true);
}

// TERM-46: a drag over a shell selects text, and the release copies it.
#[tokio::test(flavor = "multi_thread")]
async fn a_drag_copies_from_a_terminal_in_the_chat_pane() {
    drag_copies("term-drag-pane", false);
}

// TERM-46: on the full screen.
#[tokio::test(flavor = "multi_thread")]
async fn a_drag_copies_from_a_terminal_on_the_full_screen() {
    drag_copies("term-drag-full", true);
}
