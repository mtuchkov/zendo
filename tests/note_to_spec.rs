//! Note to spec (docs/zendo/note-to-spec.md): `[ → spec ]` and `s` give the agent the task to turn
//! a note into specifications.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::crossterm::event::KeyEvent;
use std::path::{Path, PathBuf};
use zendo::app::{App, Focus, Item, Mode, session_key, spec_question};

const NOTE: &str = "docs/notes/2026-11-17-tea.md";
const DESIGN: &str = "docs/design.md";
const BUTTON: &str = "[ → spec ]";

/// The questions of the user in the chat of the active agent session.
fn asked(a: &App) -> Vec<String> {
    asked_in(a, a.state.session)
}

/// The questions of the user in the chat of agent session `n`.
fn asked_in(a: &App, n: usize) -> Vec<String> {
    a.threads.get(&session_key(n)).map_or(Vec::new(), |t| t.items.iter().filter_map(|i| if let Item::User(s) = i { Some(s.clone()) } else { None }).collect())
}

/// A click of the left button at `x`, `y`.
fn click(a: &mut App, x: u16, y: u16) {
    a.on_mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column: x, row: y, modifiers: KeyModifiers::NONE });
    a.on_mouse(MouseEvent { kind: MouseEventKind::Up(MouseButton::Left), column: x, row: y, modifiers: KeyModifiers::NONE });
}

/// `s` on the open note in read mode.
fn s_on_note(a: &mut App) {
    a.mode = Mode::Read;
    a.focus = Focus::Note;
    key(a, KeyCode::Char('s'));
}

/// The last question that zendo sent to the agent, from the agent log: the line of its prompt.
/// The questions of the name model have no `Question:` line.
fn last_question(root: &Path) -> String {
    question_in_log(root).expect("a question in the agent log")
}

/// The same, when the agent got a question.
fn question_in_log(root: &Path) -> Option<String> {
    let log = std::fs::read_to_string(root.join(".zendo/agent.log")).unwrap_or_default();
    log.lines().rfind(|l| l.contains("session/prompt") && l.contains("Question: ")).map(str::to_string)
}

/// The column and the row of `text` on the screen.
fn find(screen: &[String], text: &str) -> Option<(u16, u16)> {
    screen.iter().enumerate().find_map(|(y, l)| l.find(text).map(|i| (l[..i].chars().count() as u16, y as u16)))
}

// SPEC-1, SPEC-3: a click on `[ → spec ]` of a note sends the task to the chat; a document
// outside the notes folder has no button, and `s` sends nothing for it.
#[tokio::test]
async fn s_and_the_button_send_the_note_to_spec() {
    let root = repo("note-to-spec", &[(NOTE, "# Tea\n\nGreen tea is calm.\n"), (DESIGN, "# Design\n\nThe design.\n")]);
    let mut a = app(&root);
    a.open_file(root.join(DESIGN));
    let screen = draw(&mut a, 120, 40);
    assert!(find(&screen, BUTTON).is_none(), "no button on a document:\n{}", screen.join("\n"));
    key(&mut a, KeyCode::Char('s'));
    assert!(asked(&a).is_empty(), "no task for a document: {:?}", asked(&a));
    a.open_file(root.join(NOTE));
    let screen = draw(&mut a, 120, 40);
    let (x, y) = find(&screen, BUTTON).unwrap_or_else(|| panic!("the button on a note:\n{}", screen.join("\n")));
    a.on_mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column: x + 3, row: y, modifiers: KeyModifiers::NONE });
    a.on_mouse(MouseEvent { kind: MouseEventKind::Up(MouseButton::Left), column: x + 3, row: y, modifiers: KeyModifiers::NONE });
    let q = asked(&a);
    assert_eq!(q.len(), 1, "one task: {q:?}");
    assert!(q[0].contains(NOTE) && q[0].contains("specifications"), "{q:?}");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// SPEC-1, SPEC-9: the task goes to the free agent session with the lowest id: a session with a
// session ID, or with a draft in its chat input, is not free.
#[tokio::test]
async fn the_spec_task_goes_to_the_lowest_free_session() {
    let root = repo("note-to-spec-free", &[(NOTE, "# Tea\n\nGreen tea is calm.\n")]);
    let mut a = app(&root);
    a.open_panel();
    a.focus = Focus::Claude;
    typ(&mut a, "first question");
    key(&mut a, KeyCode::Enter);
    until(&mut a, |a| a.threads.values().all(|t| t.busy.is_none()) && a.state.session(&a.cfg.threads_key(), &session_key(0)).is_some()).await;
    a.switch_session(1);
    a.focus = Focus::Claude;
    typ(&mut a, "draft");
    a.switch_session(0);
    a.open_file(root.join(NOTE));
    s_on_note(&mut a);
    assert_eq!(a.state.session, 2, "the lowest free session");
    let q = asked_in(&a, 2);
    assert!(q.len() == 1 && q[0].contains(NOTE), "{q:?}");
    assert_eq!(asked_in(&a, 0), ["first question"]);
    assert_eq!(a.chat.composer(1).input().text(), "draft", "the draft of session 1 stays");
    assert_eq!(a.files.current(), Some(&root.join(NOTE)), "the note stays open");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

/// Give each of the five agent sessions a session ID: no session is free (SPEC-9).
fn no_session_is_free(a: &mut App) {
    for n in 0..5 {
        let agent = a.cfg.agent.clone();
        a.state.set_session(&agent, &session_key(n), format!("mock-old-{n}"));
    }
}

// SPEC-10: when no agent session is free, `/new` and the task go to the start of the chat input of
// the active session, with the draft under them, and nothing goes to the agent. The instructions of
// the task are a chip of the task: they go only with it, and a second `s` does not add it again.
#[tokio::test]
async fn no_free_session_puts_new_and_the_task_into_the_input() {
    let root = repo("note-to-spec-busy", &[(NOTE, "# Tea\n\nGreen tea is calm.\n")]);
    let mut a = app(&root);
    no_session_is_free(&mut a);
    a.open_file(root.join(NOTE));
    a.open_panel();
    typ(&mut a, "draft");
    let float = chrono::Local::now();
    a.writer.float_closed(float, root.join(NOTE));
    s_on_note(&mut a);
    assert!(a.writer.keeps_name(float), "the task in the input names the file: it keeps its name");
    assert_eq!(a.state.session, 0, "the active session");
    let text = a.composer().input().text();
    assert!(text.starts_with(&format!("/new {}", spec_question(NOTE))) && text.ends_with("\ndraft"), "{text:?}");
    let chips = a.composer().chips().to_vec();
    assert_eq!(chips.len(), 1, "the instructions are one chip: {chips:?}");
    assert!(text.contains(&chips[0].label) && chips[0].context.as_deref().is_some_and(|c| c.contains("Do not change the note")), "{chips:?}");
    assert!(!a.composer().has_context(), "no context apart from the task");
    assert_eq!(a.focus, Focus::Claude, "the chat input has the focus");
    assert!(asked(&a).is_empty(), "nothing goes to the agent: {:?}", asked(&a));
    s_on_note(&mut a);
    assert_eq!(a.composer().input().text(), text, "the task is in the input one time");
    // The user takes `/new` away to ask in this thread: `s` still adds no second task.
    let e = a.composer_mut().input_mut();
    (e.row, e.col) = (0, 0);
    (0..5).for_each(|_| e.delete());
    let without_new = a.composer().input().text();
    assert!(without_new.starts_with(&spec_question(NOTE)), "{without_new:?}");
    s_on_note(&mut a);
    assert_eq!(a.composer().input().text(), without_new, "the task is in the input one time, also without `/new`");
    // The user takes the task away: the session is free of it, and nothing of it goes later.
    a.composer_mut().input_mut().clear_text();
    assert!(a.composer().is_blank(), "an input without the task is empty");
    a.focus = Focus::Claude;
    typ(&mut a, "another question");
    let q = a.composer_mut().take_question();
    assert!(q.text == "another question" && !a.composer().has_context(), "the next question goes alone: {q:?}");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// SPEC-12, SESSIONS-4: while Claude Code runs in the active session, its chat input goes to Claude
// Code: with no free session, zendo says so and puts no task into the input.
#[tokio::test]
async fn no_task_goes_into_the_input_of_a_claude_code_session() {
    let root = repo("note-to-spec-cli", &[(NOTE, "# Tea\n\nGreen tea is calm.\n")]);
    let mut a = app(&root);
    a.cli.command = Some(zendo::session::Command { program: env!("CARGO_BIN_EXE_zendo-mock-cli").into(), args: vec![], claude: true });
    no_session_is_free(&mut a);
    a.thread(&session_key(0)).session = Some("mock-old-0".into());
    a.open_panel();
    a.focus = Focus::Claude;
    ctrl(&mut a, 'o');
    until(&mut a, |a| a.cli.running(0)).await;
    a.cli.view = None;
    a.open_file(root.join(NOTE));
    let float = chrono::Local::now();
    a.writer.float_closed(float, root.join(NOTE));
    s_on_note(&mut a);
    assert!(!a.writer.keeps_name(float), "no task names the file: a title can still rename it");
    assert_eq!(a.composer().input().text(), "", "no task in the input of Claude Code");
    assert!(a.composer().is_blank());
    let said = a.threads.get(&session_key(0)).is_some_and(|t| t.items.iter().any(|i| matches!(i, Item::Info(s) if s.contains("open in Claude Code"))));
    assert!(said, "zendo says why");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

/// The task for a note of the main checkout went to agent session 1, which has a worktree with an
/// old version of the note; the answer is in.
async fn task_in_a_session_with_a_worktree(name: &str) -> (PathBuf, App) {
    let old = format!(".worktrees/1/{NOTE}");
    let root = repo(name, &[(NOTE, "# Tea\n\nGreen tea is calm.\n"), (&old, "# Tea\n\nAn old version.\n"), (DESIGN, "# Design\n")]);
    std::fs::write(root.join(".worktrees/1/.git"), "gitdir: nowhere\n").unwrap();
    let mut a = app(&root);
    let agent = a.cfg.agent.clone();
    a.state.set_session(&agent, &session_key(0), "mock-old-0".into());
    a.open_file(root.join(NOTE));
    s_on_note(&mut a);
    assert_eq!(a.state.session, 1, "the free session with the lowest id");
    until(&mut a, |a| a.threads.values().all(|t| t.busy.is_none())).await;
    (root, a)
}

// SPEC-11: the note of the user stays the document of the session through a float of another note
// and through a rename: the next question still names the file of the user.
#[tokio::test]
async fn the_note_of_the_user_stays_after_a_float_and_a_rename() {
    let (root, mut a) = task_in_a_session_with_a_worktree("note-to-spec-worktree-float").await;
    let note = root.join(NOTE);
    a.focus = Focus::Note;
    a.on_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::ALT));
    typ(&mut a, "Another note.");
    key(&mut a, KeyCode::Esc);
    assert!(a.writer.float().is_none(), "the float is closed");
    assert_eq!(a.files.current(), Some(&note), "the screen behind the float shows the note again");
    assert_eq!(a.session_path(&note), note, "a question still names the note of the user");
    let moved = root.join("docs/notes/2026-11-17-green-tea.md");
    a.files.move_file(&note, &moved).unwrap();
    assert_eq!(a.files.current(), Some(&moved));
    assert_eq!(a.session_path(&moved), moved, "the note of the user under its new name");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// SPEC-1, SPEC-11: the task for a session with a checkout of its own names and attaches the file of the
// note where the user wrote it, not a version of it in the checkout of that session. The note pane
// shows that file, and the next question names it, until another document opens.
#[tokio::test]
async fn the_task_names_the_note_where_it_is() {
    let old = format!(".worktrees/1/{NOTE}");
    let (root, mut a) = task_in_a_session_with_a_worktree("note-to-spec-worktree").await;
    let prompt = last_question(&root);
    let real = root.join(NOTE).display().to_string();
    assert!(prompt.contains(&format!("Note file: {real}")), "the prompt names the note of the user:\n{prompt}");
    assert!(prompt.contains(&format!("file://{real}")), "the prompt has the note of the user as a file:\n{prompt}");
    assert!(!prompt.contains(&root.join(&old).display().to_string()), "no version of the other checkout:\n{prompt}");
    assert_eq!(a.files.current(), Some(&root.join(NOTE)), "the note pane shows the note of the user");
    a.focus = Focus::Claude;
    typ(&mut a, "and now?");
    key(&mut a, KeyCode::Enter);
    until(&mut a, |a| asked(a).len() == 2 && a.threads.values().all(|t| t.busy.is_none())).await;
    assert!(last_question(&root).contains(&format!("Note file: {real}")), "the next question names the note on the screen");
    // Another document ends it: the note opens in the version of the session again (FILES-38).
    a.open_file(root.join(DESIGN));
    a.open_file(root.join(NOTE));
    assert_eq!(a.files.current(), Some(&root.join(&old)), "the version of the session");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

/// A float over the design document, with one line of text in its new note.
fn float_with_a_line(a: &mut App, root: &Path) {
    a.open_file(root.join(DESIGN));
    a.on_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::ALT));
    typ(a, "Only one line.");
}

/// Save the note of the float and click `[ → spec ]`. The file of the note.
fn click_spec_in_the_float(a: &mut App) -> PathBuf {
    a.save();
    let note = a.writer.path().cloned().expect("the file of the note");
    let screen = draw(a, 120, 40);
    let (x, y) = find(&screen, BUTTON).unwrap_or_else(|| panic!("the button in the float:\n{}", screen.join("\n")));
    click(a, x + 3, y);
    note
}

/// The note has the title of the mock name model.
fn titled(note: &Path) -> bool {
    std::fs::read_to_string(note).is_ok_and(|t| t.starts_with("# Mock title of 1 lines"))
}

// SPEC-6, SPEC-7, SPEC-8: the float shows `[ → spec ]` after the first save; a click closes the
// float, opens the note in the note pane, and sends the task at the click with the name that the
// file has. The title that comes later goes into the note, and the file keeps its name.
#[tokio::test]
async fn the_float_sends_the_note_to_spec_at_the_click() {
    let root = repo("note-to-spec-float", &[(DESIGN, "# Design\n\nThe design.\n")]);
    let mut a = app(&root);
    float_with_a_line(&mut a, &root);
    let screen = draw(&mut a, 120, 40);
    assert!(find(&screen, BUTTON).is_none(), "no button before the first save:\n{}", screen.join("\n"));
    let note = click_spec_in_the_float(&mut a);
    assert!(a.writer.float().is_none(), "the float is closed");
    assert_eq!(a.files.current(), Some(&note), "the note pane shows the note");
    let rel = note.strip_prefix(&root).unwrap().to_string_lossy().into_owned();
    let q = asked(&a);
    assert!(q.len() == 1 && q[0].contains(&rel), "the task goes at the click, with the name of the file: {q:?}");
    until(&mut a, |_| titled(&note)).await;
    let notes: Vec<_> = std::fs::read_dir(root.join("docs/notes")).unwrap().flatten().map(|e| e.path()).collect();
    assert_eq!(notes, std::slice::from_ref(&note), "the file keeps its name");
    assert_eq!(a.files.current(), Some(&note), "the note pane shows the note");
    assert_eq!(asked(&a).len(), 1, "one task: {:?}", asked(&a));
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// SPEC-1, SPEC-6, FLOAT-14: `[ → spec ]` in a float over a session or a terminal on the full screen
// leaves the full screen and the terminal of the chat pane, so the note and the chat with the task
// show.
#[tokio::test]
async fn the_float_over_a_full_screen_session_shows_the_note() {
    let root = repo("note-to-spec-full-screen", &[(DESIGN, "# Design\n\nThe design.\n")]);
    let mut a = app(&root);
    float_with_a_line(&mut a, &root);
    a.save();
    let screen = draw(&mut a, 120, 40);
    let (x, y) = find(&screen, BUTTON).unwrap_or_else(|| panic!("the button in the float:\n{}", screen.join("\n")));
    // A terminal in the chat pane, on the full screen (TERM-37, FLOAT-14).
    (a.cli.view, a.cli.pane) = (Some(0), Some(0));
    click(&mut a, x + 3, y);
    assert!(a.writer.float().is_none(), "the float is closed");
    assert_eq!(a.cli.view, None, "the full screen is closed");
    assert_eq!(a.cli.pane, None, "the chat shows, not the terminal in its place");
    assert_eq!(asked(&a).len(), 1, "the task went");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// SPEC-7: the title that comes after the click changes only the note, also when another document
// is open: the open document, the agent session, the mode and the focus stay.
#[tokio::test]
async fn a_late_title_changes_only_the_note() {
    let root = repo("note-to-spec-late-title", &[(DESIGN, "# Design\n\nThe design.\n")]);
    let mut a = app(&root);
    float_with_a_line(&mut a, &root);
    let note = click_spec_in_the_float(&mut a);
    a.open_file(root.join(DESIGN));
    a.focus = Focus::Note;
    let before = (a.state.session, a.mode, asked(&a).len());
    until(&mut a, |_| titled(&note)).await;
    assert_eq!(a.files.current(), Some(&root.join(DESIGN)), "the open document stays");
    assert_eq!((a.state.session, a.mode, asked(&a).len()), before, "the session, the mode and the chat stay");
    assert_eq!(a.focus, Focus::Note, "the focus stays");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// SPEC-7: the title that comes while the user edits the note goes into the editor too, and the
// text that is not saved stays.
#[tokio::test]
async fn a_late_title_goes_into_the_open_editor() {
    let root = repo("note-to-spec-title-in-editor", &[(DESIGN, "# Design\n\nThe design.\n")]);
    let mut a = app(&root);
    float_with_a_line(&mut a, &root);
    let note = click_spec_in_the_float(&mut a);
    a.focus = Focus::Note;
    a.start_write(false);
    for _ in 0..5 {
        key(&mut a, KeyCode::Down);
    }
    key(&mut a, KeyCode::End);
    typ(&mut a, " More.");
    until(&mut a, |_| titled(&note)).await;
    let text = a.writer.editor().text();
    assert!(text.starts_with("# Mock title of 1 lines") && text.trim_end().ends_with("Only one line. More."), "the title and the text of the user: {text:?}");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// SPEC-7, FLOAT-20: a note keeps the name of its file also when it goes to spec after `esc` closed
// its float, while the question for its title still waits.
#[tokio::test]
async fn a_note_that_goes_to_spec_after_esc_keeps_its_name() {
    let root = repo("note-to-spec-after-esc", &[(DESIGN, "# Design\n\nThe design.\n")]);
    let mut a = app(&root);
    float_with_a_line(&mut a, &root);
    a.save();
    let note = a.writer.path().cloned().expect("the file of the note");
    key(&mut a, KeyCode::Esc);
    assert!(a.writer.float().is_none(), "the float is closed");
    a.open_file(note.clone());
    s_on_note(&mut a);
    assert_eq!(asked(&a).len(), 1, "the task went");
    until(&mut a, |_| titled(&note)).await;
    let notes: Vec<_> = std::fs::read_dir(root.join("docs/notes")).unwrap().flatten().map(|e| e.path()).collect();
    assert_eq!(notes, std::slice::from_ref(&note), "the file keeps its name");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// SPEC-7: a note that went to spec with a title keeps that title: a title that comes later goes in
// only where the note has none.
#[tokio::test]
async fn a_late_title_does_not_replace_the_title_of_a_note_that_went_to_spec() {
    let root = repo("note-to-spec-keeps-title", &[(DESIGN, "# Design\n\nThe design.\n")]);
    let mut a = app(&root);
    a.open_file(root.join(DESIGN));
    a.on_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::ALT));
    for line in ["One.", "Two.", "Three."] {
        typ(&mut a, line);
        key(&mut a, KeyCode::Enter);
    }
    until(&mut a, |a| a.writer.editor().lines[0] == "# Mock title of 3 lines").await;
    typ(&mut a, "Four.");
    let note = click_spec_in_the_float(&mut a);
    assert_eq!(asked(&a).len(), 1, "the task went");
    // The question for a title at the close runs, and then its answer is in.
    let titling = |a: &App| a.work.job().is_some_and(|j| j.title.is_some());
    until(&mut a, titling).await;
    until(&mut a, |a| !titling(a)).await;
    let text = std::fs::read_to_string(&note).unwrap();
    assert!(text.starts_with("# Mock title of 3 lines"), "the title of the click stays: {text:?}");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// SPEC-10, HISTORY-8: `enter` on the task of the chat input starts a new thread, and the task goes
// to it with its instructions and without `/new`.
#[tokio::test]
async fn enter_sends_the_task_of_the_input_to_a_new_thread() {
    let root = repo("note-to-spec-enter", &[(NOTE, "# Tea\n\nGreen tea is calm.\n")]);
    let mut a = app(&root);
    no_session_is_free(&mut a);
    a.open_file(root.join(NOTE));
    s_on_note(&mut a);
    key(&mut a, KeyCode::Enter);
    // The mock agent says the long task again word by word: the test waits only for the prompt.
    until(&mut a, |_| question_in_log(&root).is_some()).await;
    let prompt = last_question(&root);
    assert!(prompt.contains("Do not change the note") && prompt.contains(&spec_question(NOTE)), "the task and its instructions go:\n{prompt}");
    assert!(!prompt.contains("/new"), "zendo does `/new` itself:\n{prompt}");
    assert!(!prompt.contains("mock-old-0"), "the task goes to a new thread:\n{prompt}");
    assert!(a.composer().is_blank(), "the input is empty after the task went");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// SPEC-10: a line of the task that lost its chip has no instructions: `s` puts the task into the
// input again, with its chip.
#[tokio::test]
async fn a_task_without_its_instructions_is_put_again() {
    let root = repo("note-to-spec-no-chip", &[(NOTE, "# Tea\n\nGreen tea is calm.\n")]);
    let mut a = app(&root);
    no_session_is_free(&mut a);
    a.open_file(root.join(NOTE));
    s_on_note(&mut a);
    let label = a.composer().chips()[0].label.clone();
    let bare = a.composer_mut().input_mut().take().replace(&label, "");
    a.composer_mut().input_mut().type_str(&bare);
    s_on_note(&mut a);
    let text = a.composer().input().text();
    let with_rules = |c: &zendo::app::Chip| text.contains(c.label.as_str()) && c.context.as_deref().is_some_and(|x| x.contains("Do not change the note"));
    assert!(a.composer().chips().iter().any(with_rules), "the task has its instructions again: {text:?}");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}
