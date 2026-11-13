//! Flows of the chat: one thread for every file, the file that goes with a question, slash
//! commands, and the keys of the chat input.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use zendo::app::{App, CHAT_KEY, Focus, Item};

const A: &str = "docs/notes/2026-11-12-a.md";
const B: &str = "docs/notes/2026-11-12-b.md";

fn files() -> Vec<(&'static str, &'static str)> {
    vec![(A, "# A\n\n- one\n"), (B, "# B\n\n- two\n")]
}

/// The text of the agent after the last question.
fn last_answer(a: &App) -> String {
    let items = &a.threads[CHAT_KEY].items;
    let from = items.iter().rposition(|i| matches!(i, Item::User(_))).unwrap_or(0);
    items[from..].iter().filter_map(|i| if let Item::Agent(s) = i { Some(s.as_str()) } else { None }).collect()
}

/// Ask in the chat and wait until the turn ends; the edit of the mock agent goes in at once.
async fn ask(a: &mut App, q: &str) -> String {
    a.focus = Focus::Claude;
    typ(a, q);
    key(a, KeyCode::Enter);
    until(a, |a| a.threads.get(CHAT_KEY).is_some_and(|t| t.busy.is_none() && !t.items.is_empty())).await;
    last_answer(a)
}

// CLAUDE-5, CLAUDE-26: one thread for all files: the file on screen goes with a question when it did not go with one
// of the last 20 questions.
#[tokio::test]
async fn one_thread_for_every_file_and_the_file_goes_once() {
    let root = repo("chat-one-thread", &files());
    let mut a = app(&root);
    a.open_file(root.join(A));
    a.open_panel();
    let first = ask(&mut a, "what is in it").await;
    assert!(first.contains("file link") && first.contains("2026-11-12-a.md"), "the file goes with the first question: {first}");
    let again = ask(&mut a, "and now").await;
    assert!(!again.contains("file link"), "not again for the same file: {again}");
    a.open_file(root.join(B));
    a.open_panel();
    let other = ask(&mut a, "and this one").await;
    assert!(other.contains("2026-11-12-b.md"), "the other file goes with its first question: {other}");
    let users = a.threads[CHAT_KEY].items.iter().filter(|i| matches!(i, Item::User(_))).count();
    assert_eq!(users, 3, "one thread has all the questions");
    assert_eq!(a.threads.keys().filter(|k| !k.starts_with("(recap)")).count(), 1, "{:?}", a.threads.keys().collect::<Vec<_>>());
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-15: the status bar names alt-enter for a new line: a terminal without the kitty keyboard protocol
// sends shift-enter as enter.
#[tokio::test]
async fn alt_enter_is_a_new_line_and_the_status_bar_says_so() {
    let root = repo("chat-alt-enter", &files());
    let mut a = app(&root);
    a.open_panel();
    a.focus = Focus::Claude;
    let screen = draw(&mut a, 160, 30);
    let bar = screen.iter().rev().find(|r| r.contains("CLAUDE")).unwrap();
    assert!(bar.contains("alt-enter new line"), "{bar}");
    typ(&mut a, "one");
    a.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::ALT));
    typ(&mut a, "two");
    assert_eq!(a.input.text(), "one\ntwo");
    key(&mut a, KeyCode::Left);
    key(&mut a, KeyCode::Left);
    typ(&mut a, "x");
    assert_eq!(a.input.text(), "one\ntxwo", "left moves back one char");
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-28: slash commands pass through: the list comes from the agent, `/` shows it, `tab` takes one, and
// the command goes to the agent as typed.
#[tokio::test]
async fn slash_commands_come_from_the_agent_and_go_to_it_as_typed() {
    let root = repo("chat-slash", &files());
    let mut a = app(&root);
    a.open_file(root.join(A));
    a.open_panel();
    until(&mut a, |a| !a.slash.commands.is_empty()).await;
    a.focus = Focus::Claude;
    typ(&mut a, "/rev");
    assert!(a.slash.open, "`/` and a part of a name show the commands");
    let screen = draw(&mut a, 160, 30);
    assert!(screen.iter().any(|r| r.contains("/review") && r.contains("review the note")), "{screen:#?}");
    key(&mut a, KeyCode::Tab);
    assert_eq!(a.input.text(), "/review ");
    assert!(!a.slash.open);
    typ(&mut a, "the risks");
    key(&mut a, KeyCode::Enter);
    until(&mut a, |a| a.threads[CHAT_KEY].busy.is_none() && last_answer(a).contains("ran the command")).await;
    assert_eq!(last_answer(&a), "ran the command: /review the risks", "no context before the command");
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-27, CLAUDE-6: the chat shows the last session when it opens, before the first question.
#[tokio::test]
async fn the_chat_shows_the_last_session_when_it_opens() {
    let root = repo("chat-resume", &files());
    let mut a = app(&root);
    a.open_panel();
    until(&mut a, |a| a.threads.get(CHAT_KEY).is_some_and(|t| t.session.is_some())).await;
    drop(a);
    let mut b = app(&root);
    b.open_panel();
    until(&mut b, |b| b.threads.get(CHAT_KEY).is_some_and(|t| t.items.iter().any(|i| matches!(i, Item::Agent(s) if s.contains("replayed"))))).await;
    let screen = draw(&mut b, 160, 30);
    let row = |s: &str| screen.iter().position(|r| r.contains(s)).unwrap_or_else(|| panic!("{s}: {screen:#?}"));
    assert!(row("(earlier question)") < row("(earlier answer"), "the question comes before its answer: {screen:#?}");
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-29: tab from the chat input goes to the transcript; the selection keys of read mode work
// there; `y` copies and `a` adds the text to the input as a chip of its own color.
#[tokio::test]
async fn tab_goes_to_the_transcript_to_select_copy_and_add() {
    let root = repo("chat-transcript", &files());
    let mut a = app(&root);
    a.open_file(root.join(A));
    a.open_panel();
    ask(&mut a, "what is in it").await;
    draw(&mut a, 160, 30);
    key(&mut a, KeyCode::Tab);
    assert!(a.transcript.on && a.focus == Focus::Claude, "tab: the transcript");
    let bar = draw(&mut a, 160, 30).into_iter().rev().find(|r| r.contains("TRANSCRIPT")).unwrap_or_default();
    assert!(bar.contains("y copy") && bar.contains("a to chat"), "{bar}");
    while !a.transcript.rows[a.transcript.row].trim_start().starts_with("› what is in it") {
        key(&mut a, KeyCode::Char('k'));
    }
    key(&mut a, KeyCode::Char('y'));
    assert_eq!(a.clip.as_deref(), Some("what is in it"), "no selection: the message");
    a.clip = None;
    let alt_shift = KeyModifiers::ALT | KeyModifiers::SHIFT;
    let row = a.transcript.rows[a.transcript.row].clone();
    a.transcript.col = row[..row.find("what").unwrap()].chars().count();
    a.on_key(KeyEvent::new(KeyCode::Char('L'), alt_shift));
    key(&mut a, KeyCode::Char('h'));
    key(&mut a, KeyCode::Char('h'));
    key(&mut a, KeyCode::Char('a'));
    assert!(a.input.text().contains("⟦↩ what⟧"), "a chip of the chat: {}", a.input.text());
    assert!(a.chips.iter().any(|c| c.context.as_deref().is_some_and(|x| x.contains("> what"))));
    let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(160, 30)).unwrap();
    term.draw(|f| zendo::ui::draw(f, &mut a)).unwrap();
    let buf = term.backend().buffer().clone();
    let at = (0..30u16).flat_map(|y| (0..160u16).map(move |x| (x, y))).find(|p| buf[*p].symbol() == "↩").expect("the chip on the screen");
    assert_eq!(buf[at].bg, a.theme().add, "a chip of the chat has its own color");
    key(&mut a, KeyCode::Esc);
    assert!(!a.transcript.on, "esc: back to the input");
    let _ = std::fs::remove_dir_all(&root);
}

// NOTE-25, NOTE-33: say here puts a chip with the line of the mark in the input; sent alone, it
// asks the default question.
#[tokio::test]
async fn say_here_is_a_chip_with_the_line_and_sends_a_default_question() {
    let root = repo("chat-say", &files());
    let mut a = app(&root);
    a.open_file(root.join(A));
    draw(&mut a, 140, 30);
    a.read_cursor = a.read_text.iter().position(|r| r.contains("one")).unwrap();
    a.on_key(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::ALT));
    assert_eq!(a.input.text(), "⟦write here :3⟧ ");
    key(&mut a, KeyCode::Enter);
    until(&mut a, |a| a.threads.get(CHAT_KEY).is_some_and(|t| !t.items.is_empty())).await;
    let asked = a.threads[CHAT_KEY].items.iter().find_map(|i| if let Item::User(s) = i { Some(s.clone()) } else { None }).unwrap();
    assert_eq!(asked, format!("⟦write here :3⟧ {}", zendo::app::SAY_HERE));
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-31, CLAUDE-32, TERM-41: five agent sessions; F6 to F10 choose one, alt-n the next, after
// the last the F5 terminal, then the first; each has its own thread; the buttons show the active one
// in the accent color.
#[tokio::test]
async fn five_agent_sessions_with_f6_to_f10_and_alt_n() {
    let root = repo("chat-sessions", &files());
    let mut a = app(&root);
    a.cli.shell = Some("/bin/sh".into());
    a.open_file(root.join(A));
    a.open_panel();
    ask(&mut a, "first session").await;
    key(&mut a, KeyCode::F(8));
    assert_eq!((a.state.session, a.key()), (2, "(chat 3)".to_string()));
    assert!(a.panel && a.threads.get("(chat 3)").is_none_or(|t| !t.items.iter().any(|i| matches!(i, Item::User(_)))), "its own thread");
    let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(160, 30)).unwrap();
    term.draw(|f| zendo::ui::draw(f, &mut a)).unwrap();
    let buf = term.backend().buffer().clone();
    let find = |s: &str| (0..30u16).flat_map(|y| (0..159u16).map(move |x| (x, y))).find(|p| buf[*p].symbol() == &s[..1] && buf[(p.0 + 1, p.1)].symbol() == &s[1..]).unwrap_or_else(|| panic!("{s}"));
    let (f6, f8) = (find("F6"), find("F8"));
    assert_eq!(f6.0, f8.0, "one under the other");
    assert_eq!((buf[f8].bg, buf[f6].bg), (a.theme().acc, a.theme().acc2), "the active one in the accent color");
    for want in [(3, None), (4, None), (4, Some(zendo::session::shell(0))), (0, None)] {
        a.on_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::ALT));
        assert_eq!((a.state.session, a.cli.pane), want);
    }
    a.cli.stop_all();
    assert!(a.threads[CHAT_KEY].items.iter().any(|i| matches!(i, Item::User(s) if s == "first session")), "the first thread is still there");
    assert_eq!(app(&root).state.session, 0, "kept for the repository");
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-31: alt and an F key work too, where the terminal takes the F key; the button shows the F key.
#[tokio::test]
async fn alt_and_an_f_key_choose_a_session_too() {
    let root = repo("chat-alt-f", &files());
    let mut a = app(&root);
    a.on_key(KeyEvent::new(KeyCode::F(10), KeyModifiers::ALT));
    assert_eq!(a.state.session, 4);
    let screen = draw(&mut a, 160, 30);
    assert!(screen.iter().any(|r| r.trim_end().ends_with("F10")), "the button says F10: {screen:#?}");
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-33: the agent edits the file at once: no question to the user, and no diff in the chat.
#[tokio::test]
async fn edit_is_applied_without_a_question() {
    let root = repo("chat-no-question", &files());
    let mut a = app(&root);
    a.open_file(root.join(A));
    a.open_panel();
    a.focus = Focus::Claude;
    typ(&mut a, "what is missing");
    key(&mut a, KeyCode::Enter);
    until(&mut a, |a| a.threads.get(CHAT_KEY).is_some_and(|t| t.busy.is_none() && !t.items.is_empty())).await;
    let text = std::fs::read_to_string(root.join(A)).unwrap();
    assert!(text.contains("- WHEN the mock agent"), "the edit is in the file: {text}");
    draw(&mut a, 160, 50);
    let chat = a.transcript.rows.join("\n");
    assert!(!chat.contains("WHEN the mock agent") && !chat.contains("discard"), "no diff and no question in the chat:\n{chat}");
    let _ = std::fs::remove_dir_all(&root);
}

/// Where the keys go: the chat input, its transcript, or another pane.
fn place(a: &App) -> &'static str {
    match a.focus {
        Focus::Claude if a.transcript.on => "transcript",
        Focus::Claude => "chat",
        Focus::Note => "note",
        Focus::Delta => "delta",
        Focus::Tree => "tree",
    }
}

/// Press `tab` once for each place, and check that the keys go there.
fn tabs(a: &mut App, places: &[&str]) {
    for want in places {
        key(a, KeyCode::Tab);
        assert_eq!(place(a), *want, "tab");
    }
}

// CLAUDE-36: the chat input and its transcript are a pair. The first tab of a run goes to the other
// pane of the pair; more tabs go around the panes, and back into the chat at the pane where the run
// started. Another key or a pause starts a new run.
#[tokio::test]
async fn the_first_tab_goes_to_the_sister_pane_of_the_chat() {
    let root = repo("chat-tab-pair", &files());
    let mut a = app(&root);
    a.open_file(root.join(A));
    a.open_panel();
    draw(&mut a, 160, 30);
    assert_eq!(place(&a), "chat");
    tabs(&mut a, &["transcript", "note", "chat", "transcript"]);
    key(&mut a, KeyCode::Char('k'));
    tabs(&mut a, &["chat", "note", "transcript", "chat", "note"]);
    tabs(&mut a, &["transcript", "chat"]);
    typ(&mut a, "x");
    tabs(&mut a, &["transcript", "note"]);
    a.tab_run.at = a.tab_run.at.map(|t| t - zendo::app::TAB_PAUSE);
    tabs(&mut a, &["chat", "transcript"]);
    a.tab_run.at = a.tab_run.at.map(|t| t - zendo::app::TAB_PAUSE);
    tabs(&mut a, &["chat", "note", "transcript"]);
    let _ = std::fs::remove_dir_all(&root);
}
