//! What the user can do on the steps of a turn (docs/zendo/interactivity.md): open a step, open
//! its file, add its result to the question; and the output of a command (docs/zendo/activity.md).

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::time::Instant;
use zendo::agent::{Event, ToolInfo, ToolState};
use zendo::app::{App, Focus, session_key};

fn chat(name: &str) -> (std::path::PathBuf, App) {
    let root = repo(name, &[("docs/a.md", "# A\n\none\n\ntwo\n\nthree\n")]);
    let mut a = app(&root);
    a.open_panel();
    a.focus = Focus::Claude;
    a.thread(&session_key(0)).busy = Some(Instant::now());
    (root, a)
}

fn tool(id: &str, title: &str, kind: &str, state: ToolState, info: ToolInfo) -> Event {
    Event::Tool { key: session_key(0), id: id.into(), title: title.into(), kind: kind.into(), state, info }
}

fn screen(a: &mut App) -> String {
    draw(a, 400, 50).join("\n")
}

/// Put the transcript cursor on the row that holds `text`.
fn cursor_on(a: &mut App, text: &str) -> usize {
    screen(a);
    let row = a.threads.transcript().rows().iter().position(|r| r.contains(text)).expect("the row is on screen");
    a.threads.transcript_mut().focus(true);
    a.threads.transcript_mut().place(row, 0);
    row
}

// INTER-2: `o` on a step opens it with its output, and `o` again closes it.
#[tokio::test]
async fn a_step_opens_and_closes() {
    let (root, mut a) = chat("inter-open");
    let out = vec!["line one".to_string(), "line two".into(), "line three".into()];
    a.on_agent(tool("g1", "grep three", "search", ToolState::Done, ToolInfo { output: out, ..Default::default() }));
    a.on_agent(tool("r1", "Read docs/a.md", "read", ToolState::Done, ToolInfo::default()));
    cursor_on(&mut a, "search · read");
    key(&mut a, KeyCode::Char('o'));
    let s = screen(&mut a);
    assert!(s.contains("▾ grep three") && s.contains("│ line one") && s.contains("│ line three"), "{s}");
    cursor_on(&mut a, "▾ grep three");
    key(&mut a, KeyCode::Char('o'));
    let s = screen(&mut a);
    assert!(!s.contains("line one") && !s.contains("▾ grep three"), "{s}");
    let _ = std::fs::remove_dir_all(&root);
}

// ACT-6: a command shows no output until its step is open, then its last three lines, and its
// exit code after it ends.
#[tokio::test]
async fn a_command_shows_its_output_and_exit_code() {
    let (root, mut a) = chat("act-command");
    let input = ToolInfo { input: Some("make test".into()), ..Default::default() };
    a.on_agent(tool("b1", "make test", "execute", ToolState::Running, input));
    assert!(!screen(&mut a).contains("│ $ make test"), "no output while closed");
    cursor_on(&mut a, "shell");
    key(&mut a, KeyCode::Char('o'));
    assert!(screen(&mut a).contains("│ $ make test"));
    let out: Vec<String> = (1..=5).map(|n| format!("out {n}")).collect();
    a.on_agent(tool("b1", "", "", ToolState::Failed, ToolInfo { output: out, exit: Some(1), ..Default::default() }));
    let s = screen(&mut a);
    assert!(s.contains("▾ make test · exit 1"), "{s}");
    assert!(s.contains("│ out 3") && s.contains("│ out 5") && !s.contains("out 2"), "the last three lines: {s}");
    let _ = std::fs::remove_dir_all(&root);
}

// INTER-9: a click on the file of a step opens the file at its line in the note pane.
#[tokio::test]
async fn a_click_on_the_file_of_a_step_opens_it() {
    let (root, mut a) = chat("inter-file");
    let info = ToolInfo { place: Some("docs/a.md".into()), line: Some(5), ..Default::default() };
    a.on_agent(tool("r1", "Read docs/a.md", "read", ToolState::Done, info));
    cursor_on(&mut a, "read ·");
    key(&mut a, KeyCode::Char('o'));
    let rows = draw(&mut a, 400, 50);
    let y = rows.iter().position(|r| r.contains("▾ Read docs/a.md")).unwrap();
    let x = rows[y][..rows[y].find("Read docs/a.md").unwrap()].chars().count() + "Read ".len() + 2;
    let (x, y) = (x as u16, y as u16);
    a.on_mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column: x, row: y, modifiers: KeyModifiers::NONE });
    a.on_mouse(MouseEvent { kind: MouseEventKind::Up(MouseButton::Left), column: x, row: y, modifiers: KeyModifiers::NONE });
    assert_eq!(a.files.current().map(|p| p.ends_with("docs/a.md")), Some(true), "the file opened");
    assert_eq!(a.reader.row(), 4, "at its line");
    let _ = std::fs::remove_dir_all(&root);
}

// INTER-10: `a` on a step adds its result to the question as a chip.
#[tokio::test]
async fn a_step_goes_to_the_question() {
    let (root, mut a) = chat("inter-chip");
    let info = ToolInfo { input: Some("cargo test".into()), output: vec!["test result: ok".into()], exit: Some(0), ..Default::default() };
    a.on_agent(tool("b1", "cargo test", "execute", ToolState::Done, info));
    cursor_on(&mut a, "shell ·");
    key(&mut a, KeyCode::Char('o'));
    cursor_on(&mut a, "▾ cargo test");
    key(&mut a, KeyCode::Char('a'));
    let ctx = a.composer().chips().iter().filter_map(|c| c.context.clone()).collect::<Vec<_>>().join("\n");
    assert!(ctx.contains("cargo test") && ctx.contains("Exit code: 0") && ctx.contains("> test result: ok"), "{ctx}");
    let _ = std::fs::remove_dir_all(&root);
}
