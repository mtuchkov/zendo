//! Text input: the rich editors (the note in write mode, the chat input) format and go on with
//! lists and code blocks; every one-line field has the keys of a text box (TEXT-1 to TEXT-11).

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use zendo::app::{App, Focus, Mode};
use zendo::keys::{Keymap, Pane};

const NOTE: &str = "docs/notes/2026-11-17-text.md";

fn files() -> Vec<(&'static str, &'static str)> {
    vec![(NOTE, "# Text\n\nThe first paragraph.\n"), ("docs/zendo/zendo.md", "# Zendo\n")]
}

fn press(a: &mut App, code: KeyCode, m: KeyModifiers) {
    a.on_key(KeyEvent::new(code, m));
}

fn alt(a: &mut App, c: char) {
    press(a, KeyCode::Char(c), KeyModifiers::ALT);
}

/// Write mode in a new paragraph at the end of the file.
fn writing(name: &str) -> (std::path::PathBuf, App) {
    let root = repo(name, &files());
    let mut a = app(&root);
    draw(&mut a, 120, 30);
    key(&mut a, KeyCode::Char('i'));
    draw(&mut a, 120, 30);
    assert_eq!(a.mode, Mode::Write);
    a.writer.editor_mut().move_to_end();
    key(&mut a, KeyCode::Enter);
    key(&mut a, KeyCode::Enter);
    (root, a)
}

/// The chat input in focus.
fn chatting(name: &str) -> (std::path::PathBuf, App) {
    let root = repo(name, &files());
    let mut a = app(&root);
    a.open_panel();
    a.focus = Focus::Claude;
    (root, a)
}

fn input(a: &App) -> String {
    a.composer().input().text()
}

// TEXT-4: ctrl-x cuts the selection in the chat input.
#[tokio::test]
async fn ctrl_x_cuts_in_the_chat_input() {
    let (root, mut a) = chatting("text-chat-cut");
    typ(&mut a, "hello world");
    press(&mut a, KeyCode::Home, KeyModifiers::SHIFT);
    ctrl(&mut a, 'x');
    assert_eq!((input(&a).as_str(), a.pointer.clip()), ("", Some("hello world")));
    let _ = std::fs::remove_dir_all(&root);
}

// TEXT-5, TEXT-13: the keys and the marks typed on a selection mark bold, italic, code and quote in
// the chat input as in write mode.
#[tokio::test]
async fn the_chat_input_marks_bold_italic_code_and_quote() {
    let (root, mut a) = chatting("text-chat-marks");
    typ(&mut a, "one two three");
    ctrl(&mut a, 'b');
    assert_eq!(input(&a), "one two **three**");
    press(&mut a, KeyCode::Char('w'), KeyModifiers::CONTROL);
    typ(&mut a, "x");
    press(&mut a, KeyCode::Left, KeyModifiers::SHIFT);
    typ(&mut a, "`");
    assert_eq!(input(&a), "one two `x`");
    press(&mut a, KeyCode::Home, KeyModifiers::NONE);
    ctrl(&mut a, 'i');
    assert_eq!(input(&a), "*one* two `x`");
    press(&mut a, KeyCode::End, KeyModifiers::SHIFT);
    typ(&mut a, ">");
    assert_eq!(input(&a), "> *one* two `x`");
    let _ = std::fs::remove_dir_all(&root);
}

// TEXT-8, TEXT-9, TEXT-10: the new line key of the chat goes on with a list; enter sends.
#[tokio::test]
async fn the_chat_goes_on_with_lists() {
    let (root, mut a) = chatting("text-chat-list");
    typ(&mut a, "12. first");
    ctrl(&mut a, 'j');
    assert_eq!(input(&a), "12. first\n13. ");
    typ(&mut a, "second");
    press(&mut a, KeyCode::Enter, KeyModifiers::SHIFT);
    typ(&mut a, "- a");
    press(&mut a, KeyCode::Enter, KeyModifiers::SHIFT);
    assert_eq!(input(&a), "12. first\n13. second\n14. - a\n15. ", "a bullet after a number is the text of the item");
    let _ = std::fs::remove_dir_all(&root);
}

// TEXT-7, TEXT-10: in the chat, enter after an opening fence closes the block, and enter in the
// block is a new line; alt-enter sends.
#[tokio::test]
async fn the_chat_closes_a_fence_and_enter_stays_in_the_block() {
    let (root, mut a) = chatting("text-chat-fence");
    typ(&mut a, "see ```rust");
    assert!(!a.composer().in_code(), "a fence is at the start of a line");
    ctrl(&mut a, 'u');
    typ(&mut a, "```rust");
    key(&mut a, KeyCode::Enter);
    assert_eq!(input(&a), "```rust\n\n```");
    typ(&mut a, "let x = 1;");
    key(&mut a, KeyCode::Enter);
    typ(&mut a, "let y = 2;");
    assert_eq!(input(&a), "```rust\nlet x = 1;\nlet y = 2;\n```");
    // alt-enter sends also from a code block.
    press(&mut a, KeyCode::Enter, KeyModifiers::ALT);
    assert_eq!(input(&a), "");
    let _ = std::fs::remove_dir_all(&root);
}

// TEXT-7, TEXT-9: in write mode, enter after a fence closes it, and a numbered list counts on.
#[test]
fn write_mode_closes_fences_and_counts_on() {
    let (root, mut a) = writing("text-write");
    typ(&mut a, "7. seven");
    key(&mut a, KeyCode::Enter);
    typ(&mut a, "eight");
    key(&mut a, KeyCode::Enter);
    // Enter on the empty item ends the list: the line is empty again.
    key(&mut a, KeyCode::Enter);
    typ(&mut a, "```sh");
    key(&mut a, KeyCode::Enter);
    typ(&mut a, "ls");
    let e = a.writer.editor();
    let tail: Vec<&str> = e.lines.iter().rev().take(5).rev().map(String::as_str).collect();
    assert_eq!(tail, ["7. seven", "8. eight", "```sh", "ls", "```"]);
    let _ = std::fs::remove_dir_all(&root);
}

// TEXT-6, TEXT-13: a backtick typed on lines in write mode puts a code block around them.
#[test]
fn a_backtick_on_lines_makes_a_code_block() {
    let (root, mut a) = writing("text-write-code");
    typ(&mut a, "a = 1");
    key(&mut a, KeyCode::Enter);
    typ(&mut a, "b = 2");
    // Write mode joins the lines of a paragraph: shift and left select back to the first line.
    (0..11).for_each(|_| press(&mut a, KeyCode::Left, KeyModifiers::SHIFT));
    typ(&mut a, "`");
    let e = a.writer.editor();
    let tail: Vec<&str> = e.lines.iter().rev().take(4).rev().map(String::as_str).collect();
    assert_eq!(tail, ["```", "a = 1", "b = 2", "```"]);
    let _ = std::fs::remove_dir_all(&root);
}

// TEXT-1, TEXT-2: the find field moves by words and to its ends, types at the cursor, and cuts.
#[tokio::test]
async fn the_find_field_is_a_text_box() {
    let root = repo("text-find", &files());
    let mut a = app(&root);
    draw(&mut a, 120, 30);
    alt(&mut a, '/');
    assert_eq!(a.mode, Mode::Find);
    typ(&mut a, "docs zendo");
    press(&mut a, KeyCode::Left, KeyModifiers::CONTROL);
    press(&mut a, KeyCode::Backspace, KeyModifiers::NONE);
    typ(&mut a, "/");
    assert_eq!(a.files.find().query.text(), "docs/zendo");
    press(&mut a, KeyCode::Home, KeyModifiers::NONE);
    press(&mut a, KeyCode::End, KeyModifiers::SHIFT);
    ctrl(&mut a, 'x');
    assert_eq!((a.files.find().query.text(), a.pointer.clip()), ("", Some("docs/zendo")));
    draw(&mut a, 120, 30);
    let _ = std::fs::remove_dir_all(&root);
}

// TEXT-1, TEXT-3: a file name takes a paste at the cursor, in one line.
#[test]
fn a_file_name_takes_a_paste_in_one_line() {
    let root = repo("text-prompt", &files());
    let mut a = app(&root);
    a.files.ask(zendo::app::FilePrompt::NewFile("docs/".into()));
    a.on_paste("idea\nnotes".into());
    press(&mut a, KeyCode::Left, KeyModifiers::ALT);
    typ(&mut a, "x");
    assert_eq!(a.files.prompt().and_then(|p| p.field()).map(|f| f.text().to_string()), Some("docs/idea xnotes".into()));
    let _ = std::fs::remove_dir_all(&root);
}

// TEXT-11: the keys of every field are in `[textbox]` of the keymap, and the help shows them.
#[test]
fn the_text_box_keys_are_in_the_keymap_and_the_help() {
    let k = Keymap::defaults();
    let ev = |c, m| KeyEvent::new(c, m);
    assert_eq!(k.local(Pane::Field, &ev(KeyCode::Char('x'), KeyModifiers::CONTROL)), Some("cut"));
    assert_eq!(k.local(Pane::Field, &ev(KeyCode::Left, KeyModifiers::CONTROL)), Some("word_left"));
    assert_eq!(k.local(Pane::Chat, &ev(KeyCode::Char('x'), KeyModifiers::CONTROL)), Some("cut"));
    assert_eq!(k.local(Pane::Chat, &ev(KeyCode::Enter, KeyModifiers::ALT)), Some("send_now"));
    assert_eq!(k.local(Pane::Field, &ev(KeyCode::Char('a'), KeyModifiers::CONTROL)), None, "home and end only");
    let root = repo("text-help", &files());
    let mut a = app(&root);
    draw(&mut a, 120, 30);
    key(&mut a, KeyCode::Char('e'));
    key(&mut a, KeyCode::F(1));
    let screen = draw(&mut a, 200, 70).join("\n");
    assert!(screen.contains("TEXTBOX") && screen.contains("clear the field"), "{screen}");
    let _ = std::fs::remove_dir_all(&root);
}

const TASKS: &str = "docs/notes/2026-11-17-tasks.md";

fn click(a: &mut App, x: u16, y: u16) {
    use ratatui::crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
    for kind in [MouseEventKind::Down(MouseButton::Left), MouseEventKind::Up(MouseButton::Left)] {
        a.on_mouse(MouseEvent { kind, column: x, row: y, modifiers: KeyModifiers::NONE });
    }
    // A worker writes the box into the file (IO-1).
    a.settle_saves();
}

/// The place of the first `c` on the screen.
fn find_on(screen: &[String], c: char) -> (u16, u16) {
    let y = screen.iter().position(|r| r.contains(c)).unwrap_or_else(|| panic!("no {c} on {screen:#?}"));
    let x = screen[y].chars().position(|x| x == c).unwrap();
    (x as u16, y as u16)
}

// TEXT-14: a click on the box of a task in read mode checks it in the file; a second click opens it.
#[test]
fn a_click_on_a_task_box_checks_it() {
    let root = repo("text-task-read", &[(TASKS, "# Tasks\n\n- [ ] milk\n- [x] bread\n"), ("docs/zendo/zendo.md", "# Zendo\n")]);
    let mut a = app(&root);
    a.open_file(root.join(TASKS));
    let screen = draw(&mut a, 120, 30);
    let (x, y) = find_on(&screen, '☐');
    click(&mut a, x, y);
    assert_eq!(std::fs::read_to_string(root.join(TASKS)).unwrap(), "# Tasks\n\n- [x] milk\n- [x] bread\n");
    let screen = draw(&mut a, 120, 30);
    assert!(!screen.iter().any(|r| r.contains('☐')), "{screen:#?}");
    click(&mut a, x, y);
    assert_eq!(std::fs::read_to_string(root.join(TASKS)).unwrap(), "# Tasks\n\n- [ ] milk\n- [x] bread\n");
    let _ = std::fs::remove_dir_all(&root);
}

// TEXT-14: in write mode a click on the box changes the task in the text.
#[test]
fn a_click_on_a_task_box_in_write_mode() {
    let root = repo("text-task-write", &[(TASKS, "# Tasks\n\nintro\n\n- [ ] milk\n"), ("docs/zendo/zendo.md", "# Zendo\n")]);
    let mut a = app(&root);
    a.open_file(root.join(TASKS));
    draw(&mut a, 120, 30);
    a.reader.to_row(a.frame.read_text().iter().position(|r| r.starts_with("intro")).unwrap());
    key(&mut a, KeyCode::Char('i'));
    let screen = draw(&mut a, 120, 30);
    let (x, y) = find_on(&screen, '☐');
    click(&mut a, x, y);
    assert!(a.writer.editor().lines.iter().any(|l| l == "- [x] milk"), "{:?}", a.writer.editor().lines);
    let _ = std::fs::remove_dir_all(&root);
}

fn today() -> String {
    zendo::text::editor::DateFormat::system().format(chrono::Local::now().date_naive())
}

// TEXT-15, TEXT-16, TEXT-17: in write mode `//` puts the date of today; while the selector is on, `k`
// and `j` change the part and are no text; enter leaves it, and the keys type again.
#[test]
fn slashes_open_the_date_selector_in_write_mode() {
    let (root, mut a) = writing("text-date-write");
    typ(&mut a, "due //");
    let line = |a: &App| a.writer.editor().lines[a.writer.editor().row].clone();
    assert_eq!(line(&a), format!("due {}", today()));
    typ(&mut a, "kj");
    assert_eq!(line(&a), format!("due {}", today()), "up and down: the same date again");
    key(&mut a, KeyCode::Enter);
    typ(&mut a, " x");
    assert_eq!(line(&a), format!("due {} x", today()));
    let _ = std::fs::remove_dir_all(&root);
}

// TEXT-17: in the chat, enter leaves the date selector and does not send.
#[tokio::test]
async fn enter_in_the_chat_date_selector_does_not_send() {
    let (root, mut a) = chatting("text-date-chat");
    typ(&mut a, "//");
    key(&mut a, KeyCode::Up);
    key(&mut a, KeyCode::Enter);
    assert_ne!(input(&a), "", "not sent");
    assert!(a.composer().input().date.is_none());
    let _ = std::fs::remove_dir_all(&root);
}
