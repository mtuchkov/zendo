//! The attention of the user (docs/zendo/attention.md): the bell and the blinking button when the turn
//! of an agent session or the command of a terminal ends, and the sound button.

mod common;
use common::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::style::{Color, Modifier};
use std::time::{Duration, Instant};
use zendo::agent::Event;
use zendo::app::{App, Focus, session_key};
use zendo::session::{Command, State, shell};

fn mock() -> Command {
    Command { program: env!("CARGO_BIN_EXE_zendo-mock-cli").into(), args: vec!["--resume".into()] }
}

/// An app with the mock command line, `/bin/sh` for the terminals, the chat open, and a config file
/// in the repository, not in the home folder.
fn app_here(name: &str) -> (std::path::PathBuf, App) {
    let root = repo(name, &[("docs/notes/2026-11-12-a.md", "# A\n\n- one\n")]);
    let mut a = app(&root);
    a.cli.command = Some(mock());
    a.cli.shell = Some("/bin/sh".into());
    a.attention.config = root.join("config.toml");
    a.attention.sound = true;
    a.thread(&session_key(0)).session = Some("s1".into());
    a.open_panel();
    (root, a)
}

fn done(root: &std::path::Path, a: &mut App) {
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(root);
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

/// Tick for `ms` milliseconds.
fn idle(a: &mut App, ms: u64) {
    let end = Instant::now() + Duration::from_millis(ms);
    while Instant::now() < end {
        a.tick();
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn chat_on(a: &mut App, n: usize) {
    a.switch_session(n);
    a.focus = Focus::Claude;
}

fn leave(a: &mut App) {
    a.on_key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::ALT));
}

/// Start the Claude Code session of agent session 1 and wait for its first output.
fn start_cli(a: &mut App) {
    chat_on(a, 0);
    ctrl(a, 'o');
    wait(a, 10, |a| a.cli.text(0).contains("tick "));
}

/// One turn of the mock command line: the hooks report *works*, then *idle*.
fn turn(a: &mut App) {
    a.cli.write(0, b"r");
    wait(a, 5, |a| a.cli.state(0) == Some(State::Works) && a.cli.text(0).contains("tick "));
    idle(a, 100);
    a.cli.write(0, b"i");
    wait(a, 5, |a| a.cli.state(0) == Some(State::Idle));
    idle(a, 100);
}

/// Draw one frame; the buffer, with the styles.
fn frame(a: &mut App, w: u16, h: u16) -> Buffer {
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| zendo::ui::draw(f, a)).unwrap();
    term.backend().buffer().clone()
}

/// The first cell of `text` in the buffer: column and row.
fn find(buf: &Buffer, text: &str) -> Option<(u16, u16)> {
    let (w, h) = (buf.area.width, buf.area.height);
    (0..h).find_map(|y| {
        let row: String = (0..w).map(|x| buf[(x, y)].symbol().to_string()).collect();
        row.find(text).map(|i| (row[..i].chars().count() as u16, y))
    })
}

fn click(a: &mut App, (column, row): (u16, u16)) {
    a.on_mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column, row, modifiers: KeyModifiers::NONE });
    a.on_mouse(MouseEvent { kind: MouseEventKind::Up(MouseButton::Left), column, row, modifiers: KeyModifiers::NONE });
}

/// The background of the button `label` on the right edge of the chat.
fn button_bg(a: &mut App, label: &str) -> Color {
    let buf = frame(a, 160, 40);
    let (x, y) = find(&buf, label).unwrap_or_else(|| panic!("no button {label}"));
    buf[(x, y)].bg
}

// ATTN-1, ATTN-2, ATTN-4, ATTN-6: the start of a Claude Code session rings nothing; the end of a turn
// that the user does not see rings once and blinks the button; the chat of the session stops it.
#[tokio::test(flavor = "multi_thread")]
async fn a_claude_code_turn_rings_and_blinks_while_the_user_is_away() {
    let (root, mut a) = app_here("attn-cli");
    start_cli(&mut a);
    leave(&mut a);
    chat_on(&mut a, 1);
    idle(&mut a, 1500);
    assert_eq!((a.attention.bells, a.blink(0)), (0, None), "no hook reported a turn: no bell");
    turn(&mut a);
    assert_eq!(a.attention.bells, 1, "one bell for one turn");
    assert!(a.blink(0).is_some(), "the F6 button blinks");
    chat_on(&mut a, 0);
    idle(&mut a, 300);
    assert_eq!(a.blink(0), None, "the chat of the session stops the blink");
    done(&root, &mut a);
}

// ATTN-1, ATTN-2, ATTN-4, ATTN-11: a turn on the screen rings nothing while the terminal has the
// focus; without the focus it rings and blinks, and the focus stops the blink.
#[tokio::test(flavor = "multi_thread")]
async fn a_turn_on_the_screen_rings_only_without_the_focus() {
    let (root, mut a) = app_here("attn-seen");
    start_cli(&mut a);
    turn(&mut a);
    assert_eq!((a.attention.bells, a.blink(0)), (0, None), "the user sees the session");
    a.on_focus(false);
    turn(&mut a);
    assert_eq!(a.attention.bells, 1);
    assert!(a.blink(0).is_some());
    a.on_focus(true);
    idle(&mut a, 300);
    assert_eq!(a.blink(0), None, "the user is back");
    done(&root, &mut a);
}

// ATTN-1, ATTN-2: the end of a command in a terminal that the user left rings and blinks.
#[tokio::test(flavor = "multi_thread")]
async fn the_end_of_a_terminal_command_rings_and_blinks() {
    let (root, mut a) = app_here("attn-term");
    key(&mut a, KeyCode::F(5));
    assert_eq!(a.cli.pane, Some(shell(0)), "F5 shows the F5 terminal");
    wait(&mut a, 10, |a| a.cli.text(shell(0)).contains('$') || a.cli.text(shell(0)).contains('#'));
    a.cli.write(shell(0), b"sleep 1\r");
    wait(&mut a, 5, |a| a.cli.busy(0));
    leave(&mut a);
    wait(&mut a, 5, |a| !a.cli.busy(0));
    idle(&mut a, 100);
    assert_eq!(a.attention.bells, 1);
    assert!(a.blink(shell(0)).is_some(), "the F5 button blinks");
    done(&root, &mut a);
}

// ATTN-1, ATTN-2, ATTN-5: the end of an answer in the chat of another agent session rings and blinks;
// an answer that the user stopped does not.
#[tokio::test(flavor = "multi_thread")]
async fn the_end_of_a_chat_answer_rings_and_blinks() {
    let (root, mut a) = app_here("attn-chat");
    chat_on(&mut a, 0);
    a.on_agent(Event::TurnEnd { key: session_key(2), stop: "cancelled".into() });
    assert_eq!((a.attention.bells, a.blink(2)), (0, None), "the user stopped it");
    a.on_agent(Event::TurnEnd { key: session_key(2), stop: "end_turn".into() });
    assert_eq!(a.attention.bells, 1);
    assert!(a.blink(2).is_some(), "the F8 button blinks");
    a.on_agent(Event::TurnEnd { key: session_key(0), stop: "end_turn".into() });
    assert_eq!((a.attention.bells, a.blink(0)), (1, None), "the user sees the chat of session 1");
    done(&root, &mut a);
}

// ATTN-3: the button changes its color smoothly from its normal color to the accent and back.
#[tokio::test(flavor = "multi_thread")]
async fn a_blinking_button_fades_to_the_accent_and_back() {
    let (root, mut a) = app_here("attn-fade");
    chat_on(&mut a, 0);
    let t = *a.theme();
    let normal = button_bg(&mut a, " F7");
    assert_eq!(normal, t.acc2);
    let colors: Vec<Color> = (0..=10)
        .map(|i| {
            a.attention.blink_since(1, Instant::now() - Duration::from_millis(100 * i));
            button_bg(&mut a, " F7")
        })
        .collect();
    assert_eq!(colors[0], t.acc2, "the blink starts at the normal color");
    assert_eq!(colors[10], t.acc, "after 1 s the button has the accent color");
    assert!(colors.windows(2).all(|w| w[0] != w[1]), "a new shade each 100 ms: {colors:?}");
    a.attention.blink_since(1, Instant::now() - Duration::from_millis(2000));
    assert_eq!(button_bg(&mut a, " F7"), t.acc2, "after 2 s the normal color again");
    done(&root, &mut a);
}

// ATTN-7, ATTN-8, ATTN-1: the sound button left of the branch name turns the sound off and on, and
// saves the setting; without sound, the end of a turn only blinks.
#[tokio::test(flavor = "multi_thread")]
async fn the_sound_button_turns_the_bell_off_and_saves_it() {
    let (root, mut a) = app_here("attn-sound");
    a.branch = Some("main".into());
    chat_on(&mut a, 0);
    let buf = frame(&mut a, 160, 40);
    let at = find(&buf, "[sound]").expect("the sound button");
    let branch = find(&buf, "main").expect("the branch");
    assert_eq!(at.1, branch.1, "on the line of the branch");
    assert!(at.0 < branch.0, "left of the branch");
    assert!(!buf[at].modifier.contains(Modifier::CROSSED_OUT));
    click(&mut a, at);
    assert!(!a.attention.sound);
    let saved = std::fs::read_to_string(root.join("config.toml")).unwrap();
    assert!(saved.contains("sound = false"), "{saved}");
    let buf = frame(&mut a, 160, 40);
    assert!(buf[at].modifier.contains(Modifier::CROSSED_OUT), "struck through while silent");
    a.on_agent(Event::TurnEnd { key: session_key(3), stop: "end_turn".into() });
    assert_eq!(a.attention.bells, 0, "no bell while silent");
    assert!(a.blink(3).is_some(), "but the button blinks");
    click(&mut a, at);
    assert!(a.attention.sound);
    assert!(std::fs::read_to_string(root.join("config.toml")).unwrap().contains("sound = true"));
    done(&root, &mut a);
}

// ATTN-10: while the chat is closed, a blinking session shows in the top line; a click opens it.
#[tokio::test(flavor = "multi_thread")]
async fn a_blinking_session_shows_in_the_top_line_while_the_chat_is_closed() {
    let (root, mut a) = app_here("attn-top");
    chat_on(&mut a, 0);
    a.panel = false;
    a.on_agent(Event::TurnEnd { key: session_key(1), stop: "end_turn".into() });
    assert!(a.blink(1).is_some());
    let buf = frame(&mut a, 160, 40);
    let sound = find(&buf, "[sound]").unwrap();
    let chip = find(&buf, " F7 ").expect("the F7 chip in the top line");
    assert_eq!(chip.1, sound.1);
    click(&mut a, (chip.0 + 1, chip.1));
    assert!(a.panel && a.state.session == 1, "the click opens the chat of session 2");
    idle(&mut a, 300);
    assert_eq!(a.blink(1), None);
    done(&root, &mut a);
}
