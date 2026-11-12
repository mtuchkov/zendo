//! Flows of the file tree, the chat and the note pane, through keys and a test terminal.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use zendo::agent::Event;
use zendo::app::{FilePrompt, Focus, Mode};

const NOTE: &str = "docs/notes/2026-11-12-idea.md";

fn files() -> Vec<(&'static str, &'static str)> {
    vec![
        (NOTE, "# Idea\n\nThe first paragraph of the idea.\n\nThe second **bold** paragraph.\n\n| A | B |\n|---|---|\n| one | two |\n"),
        ("docs/zendo/zendo.md", "# Zendo\n"),
        ("docs/zendo/notes.md", "# Notes\n"),
    ]
}

/// An app with the file tree open and in focus, and the row of `rel` selected.
fn in_tree(root: &std::path::Path, rel: &str) -> zendo::app::App {
    let mut a = app(root);
    a.toggle_tree();
    draw(&mut a, 140, 30);
    a.focus = Focus::Tree;
    a.tree.sel = a.tree.rows().iter().position(|r| r.rel.ends_with(rel)).unwrap_or_else(|| panic!("{rel} in the tree"));
    a
}

// FILES-14, FILES-16: `m` asks for a folder on a line under the status bar, and moves the file.
#[test]
fn m_moves_a_file_to_a_folder() {
    let root = repo("flow-move", &files());
    let mut a = in_tree(&root, "zendo/notes.md");
    key(&mut a, KeyCode::Char('m'));
    assert!(matches!(a.file_prompt, Some(FilePrompt::Move(..))));
    let screen = draw(&mut a, 140, 30);
    assert!(screen[29].contains("MOVE") && screen[29].contains("docs/zendo"), "the last row: {}", screen[29]);
    (0..10).for_each(|_| key(&mut a, KeyCode::Backspace));
    typ(&mut a, "docs/old");
    key(&mut a, KeyCode::Enter);
    assert!(root.join("docs/old/notes.md").is_file() && !root.join("docs/zendo/notes.md").exists());
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-18: `f4` starts with the folder of the selection; `tab` completes a folder name.
#[test]
fn f4_starts_in_the_folder_of_the_selection_and_tab_completes() {
    let root = repo("flow-new", &files());
    let mut a = in_tree(&root, "zendo/notes.md");
    key(&mut a, KeyCode::F(4));
    assert_eq!(a.file_prompt, Some(FilePrompt::NewFolder("docs/zendo/".into())));
    key(&mut a, KeyCode::Esc);
    key(&mut a, KeyCode::F(3));
    (0..11).for_each(|_| key(&mut a, KeyCode::Backspace));
    typ(&mut a, "docs/zen");
    key(&mut a, KeyCode::Tab);
    assert_eq!(a.file_prompt, Some(FilePrompt::NewFile("docs/zendo/".into())));
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-11: `esc` in the name filter shows the tree with the match selected.
#[test]
fn esc_in_the_filter_keeps_the_match_selected() {
    let root = repo("flow-find", &files());
    let mut a = in_tree(&root, "zendo/zendo.md");
    key(&mut a, KeyCode::Char('/'));
    typ(&mut a, "notes");
    key(&mut a, KeyCode::Down);
    let picked = a.tree.rows()[a.tree.sel].rel.clone();
    key(&mut a, KeyCode::Esc);
    assert!(a.tree.find.is_none());
    assert_eq!(a.tree.rows()[a.tree.sel].rel, picked);
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-15: `a` in the tree adds the file to the chat; the focus stays in the tree.
#[tokio::test]
async fn a_in_the_tree_adds_the_file_to_the_chat() {
    let root = repo("flow-attach", &files());
    let mut a = in_tree(&root, "zendo/zendo.md");
    key(&mut a, KeyCode::Char('a'));
    assert_eq!(a.focus, Focus::Tree);
    assert!(a.chips.last().is_some_and(|c| c.attach.is_some()));
    assert!(a.input.text().contains("endo/zendo.md"), "{}", a.input.text());
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-21: the delta closes the file tree when both do not fit.
#[test]
fn the_delta_closes_the_tree_when_it_does_not_fit() {
    let root = repo("flow-delta", &files());
    let mut a = in_tree(&root, "notes/2026-11-12-idea.md");
    key(&mut a, KeyCode::Enter);
    draw(&mut a, 90, 30);
    a.focus = Focus::Note;
    key(&mut a, KeyCode::Char('d'));
    draw(&mut a, 90, 30);
    assert!(a.delta_open && !a.state.tree, "tree {}", a.state.tree);
    assert!(a.visible.contains(&Focus::Delta));
    let _ = std::fs::remove_dir_all(&root);
}

// PANES-22: the width keys change the text column and keep it for the repository.
#[test]
fn the_width_keys_change_the_text_column() {
    let root = repo("flow-width", &files());
    let mut a = app(&root);
    let w = a.column_width();
    a.on_key(KeyEvent::new(KeyCode::Char('='), KeyModifiers::ALT));
    assert_eq!(a.column_width(), w + 4);
    a.on_key(KeyEvent::new(KeyCode::Char('-'), KeyModifiers::ALT));
    a.on_key(KeyEvent::new(KeyCode::Char('-'), KeyModifiers::ALT));
    assert_eq!(a.column_width(), w - 4);
    assert_eq!(app(&root).column_width(), w - 4, "kept for the repository");
    let _ = std::fs::remove_dir_all(&root);
}

// CLAUDE-22, CLAUDE-23: `@` and a part of a name add a file chip; the arrows and backspace take it whole.
#[tokio::test]
async fn at_adds_a_file_chip_that_moves_and_goes_as_one() {
    let root = repo("flow-at", &files());
    let mut a = app(&root);
    a.open_panel();
    a.focus = Focus::Claude;
    typ(&mut a, "see @zend");
    assert!(a.at.as_ref().is_some_and(|p| p.hits[0].ends_with("zendo/zendo.md")), "{:?}", a.at);
    key(&mut a, KeyCode::Tab);
    let text = a.input.text();
    assert!(text.starts_with("see ⟦endo/zendo.md⟧ "), "{text}");
    key(&mut a, KeyCode::Left);
    key(&mut a, KeyCode::Left);
    assert_eq!(a.input.col, 4, "one left over the space, one over the whole chip");
    key(&mut a, KeyCode::Right);
    key(&mut a, KeyCode::Backspace);
    assert_eq!(a.input.text(), "see  ");
    assert!(a.chips.is_empty());
    let _ = std::fs::remove_dir_all(&root);
}

// NOTE-25, NOTE-27: ctrl-h shows the arrow and starts the sentence; the end of the answer removes it.
#[tokio::test]
async fn ctrl_h_marks_the_place_until_the_answer_ends() {
    let root = repo("flow-say", &files());
    let mut a = app(&root);
    draw(&mut a, 120, 30);
    a.read_cursor = a.read_text.iter().position(|r| r.starts_with("The first")).unwrap();
    ctrl(&mut a, 'h');
    assert!(a.input.text().starts_with("Here is what I want to say here: "));
    a.focus = Focus::Note;
    let screen = draw(&mut a, 120, 30);
    let row = screen.iter().find(|r| r.contains("The first paragraph")).unwrap();
    assert!(row.find('→').unwrap() < row.find("The first").unwrap(), "the arrow is left of the line: {row}");
    let key_of = a.say.as_ref().unwrap().key.clone();
    a.on_agent(Event::TurnEnd { key: key_of, stop: "end_turn".into() });
    assert!(a.say.is_none());
    ctrl(&mut a, 'h');
    a.focus = Focus::Note;
    key(&mut a, KeyCode::Esc);
    assert!(a.say.is_none(), "esc in the note");
    let _ = std::fs::remove_dir_all(&root);
}

// NOTE-28, NOTE-30: write mode keeps the char of the cursor, shows only its line as source, and a
// table keeps its borders but for the row of the cursor.
#[test]
fn write_mode_keeps_the_char_and_shows_only_the_cursor_line_as_source() {
    let root = repo("flow-write", &files());
    let mut a = app(&root);
    draw(&mut a, 120, 30);
    let row = a.read_text.iter().position(|r| r.starts_with("The second bold")).unwrap();
    a.read_cursor = row;
    a.read_col = "The second ".len();
    key(&mut a, KeyCode::Char('i'));
    assert_eq!(a.mode, Mode::Write);
    let line = &a.editor.lines[a.editor.row];
    assert_eq!(&line[a.editor.col..a.editor.col + 4], "bold", "{line}");
    let screen = draw(&mut a, 120, 30);
    assert!(screen.iter().any(|r| r.contains("The second **bold** paragraph.")), "the cursor line is source");
    assert!(screen.iter().any(|r| r.contains("│ one │ two │")), "the table keeps its borders");
    a.editor.row = a.editor.lines.iter().position(|l| l.starts_with("| one")).unwrap();
    let screen = draw(&mut a, 120, 30);
    assert!(screen.iter().any(|r| r.contains("| one | two |")) && screen.iter().any(|r| r.contains("│ A   │ B   │")), "only the row of the cursor is source");
    a.editor.row = a.editor.lines.iter().position(|l| l.starts_with("The second")).unwrap();
    a.editor.col = "The second **".len();
    key(&mut a, KeyCode::Esc);
    draw(&mut a, 120, 30);
    assert_eq!(&a.read_text[a.read_cursor][a.read_col..a.read_col + 4], "bold");
    let _ = std::fs::remove_dir_all(&root);
}

// NOTE-29: typing past the text column breaks the line at a word, in the file.
#[test]
fn typing_past_the_column_breaks_the_line() {
    let root = repo("flow-wrap", &files());
    let mut a = app(&root);
    draw(&mut a, 120, 30);
    a.read_cursor = a.read_text.iter().position(|r| r.starts_with("The first")).unwrap();
    key(&mut a, KeyCode::Char('i'));
    draw(&mut a, 120, 30);
    a.editor.end();
    typ(&mut a, " And it goes on with more words, so many that the line is longer than the text column.");
    key(&mut a, KeyCode::Esc);
    let text = std::fs::read_to_string(root.join(NOTE)).unwrap();
    let longest = text.lines().map(|l| l.chars().count()).max().unwrap();
    assert!(longest <= a.text_width, "{longest} > {}: {text}", a.text_width);
    assert!(text.contains("The first paragraph of the idea. And it goes on"));
    let _ = std::fs::remove_dir_all(&root);
}

// NOTE-31: write mode wraps a paragraph as read mode does, and keeps the head of the note (title,
// date, recap) where read mode has it. Only the line of the cursor shows as source.
#[test]
fn write_mode_wraps_as_read_mode_and_keeps_the_head() {
    let long = "# Wrap\n\nA zendo is the hall where Zen practitioners sit for zazen, seated meditation.\nDeep thinking about the architecture of a project feels much the same: you sit\nwith a problem until it settles. zendo is a minimal, distraction-free place for\nthat work.\n";
    let root = repo("flow-wrap-same", &[("docs/notes/2026-11-12-wrap.md", long)]);
    let mut a = app(&root);
    let rel = "docs/notes/2026-11-12-wrap.md";
    let recap = zendo::setup::state::Recap { text: "- Decides: the hall.".into(), of_hash: String::new(), of_text: String::new(), at: chrono::Local::now(), auto: true };
    a.state.file(rel).recap = Some(recap);
    let read = draw(&mut a, 100, 30);
    let row = a.read_text.iter().position(|r| r.starts_with("A zendo is")).unwrap();
    a.read_cursor = row;
    key(&mut a, KeyCode::Char('i'));
    let write = draw(&mut a, 100, 30);
    let trim = |s: &Vec<String>| s.iter().map(|l| l.trim_end().to_string()).collect::<Vec<_>>();
    let (read, write) = (trim(&read), trim(&write));
    let at = |screen: &Vec<String>, s: &str| screen.iter().position(|l| l.contains(s));
    assert_eq!(at(&read, "Wrap"), at(&write, "Wrap"), "the title stays in place");
    assert!(at(&write, "RECAP").is_some(), "write mode keeps the recap");
    assert_eq!(at(&read, "RECAP"), at(&write, "RECAP"));
    // The lines after the cursor line wrap as in read mode: no row ends with a lone word.
    let after = |screen: &Vec<String>| screen.iter().skip_while(|l| !l.contains("Deep thinking")).take(3).cloned().collect::<Vec<_>>();
    assert!(after(&write).iter().all(|l| l.split_whitespace().count() > 2), "{:#?}", after(&write));
    let _ = std::fs::remove_dir_all(&root);
}

fn ctrl_shift(app: &mut zendo::app::App, c: char) {
    app.on_key(KeyEvent::new(KeyCode::Char(c.to_ascii_uppercase()), KeyModifiers::CONTROL | KeyModifiers::SHIFT));
}

// NOTE-25, NOTE-18: say here, ask and add to chat work in write mode too.
#[tokio::test]
async fn say_and_add_to_chat_work_in_write_mode() {
    let root = repo("flow-write-keys", &files());
    let mut a = app(&root);
    draw(&mut a, 120, 30);
    a.read_cursor = a.read_text.iter().position(|r| r.starts_with("The first")).unwrap();
    key(&mut a, KeyCode::Char('i'));
    draw(&mut a, 120, 30);
    ctrl_shift(&mut a, 's');
    assert!(a.input.text().starts_with("Here is what I want to say here: "), "{}", a.input.text());
    a.focus = Focus::Note;
    let screen = draw(&mut a, 120, 30);
    let row = screen.iter().find(|r| r.contains("The first paragraph")).unwrap();
    assert!(row.find('→').unwrap() < row.find("The first").unwrap(), "the arrow in write mode: {row}");
    a.say = None;
    a.input.take();
    a.chips.clear();
    a.on_key(KeyEvent::new(KeyCode::Char('+'), KeyModifiers::CONTROL | KeyModifiers::SHIFT));
    let chip = a.chips.last().expect("the line as a chip");
    assert!(chip.label.contains("The first parag"), "{}", chip.label);
    assert!(chip.context.as_deref().unwrap_or_default().contains("> The first paragraph of the idea."));
    let _ = std::fs::remove_dir_all(&root);
}

// The status bar names the four cursor keys and the four select keys at once.
#[test]
fn the_status_bar_names_hjkl() {
    let root = repo("flow-hints", &files());
    let mut a = app(&root);
    let screen = draw(&mut a, 160, 30);
    let bar = screen.iter().rev().find(|r| r.contains("READ")).unwrap();
    assert!(bar.contains("h/j/k/l cursor") && bar.contains("^⇧h/j/k/l select") && bar.contains("^⇧+ to chat"), "{bar}");
    let _ = std::fs::remove_dir_all(&root);
}
