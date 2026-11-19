//! A file that the user opens from the file tree works like a note: read, write, and Claude.

mod common;
use common::*;
use ratatui::crossterm::event::KeyCode;
use std::path::Path;
use std::time::{Duration, Instant};
use zendo::app::{CHAT_KEY, Focus, Item, Mode};

const DESIGN: &str = "docs/design.md";

// FILES-3: a file from the tree opens like a note; the user writes in it and asks Claude about it.
#[tokio::test]
async fn file_from_the_tree_works_like_a_note() {
    let root = repo("tree-open-like-a-note", &[(DESIGN, "# Design\n\nThe first line of the design.\n"), ("docs/notes/.keep", "")]);
    let mut a = app(&root);
    a.toggle_tree();
    a.focus = Focus::Tree;
    draw(&mut a, 120, 30);
    a.files.reveal(Path::new(DESIGN));
    assert_eq!(a.files.tree_file().as_deref(), Some(Path::new(DESIGN)), "the row of the file");
    key(&mut a, KeyCode::Enter);
    assert_eq!(a.files.current(), Some(&root.join(DESIGN)), "the file opens");

    draw(&mut a, 120, 30);
    let row = a.frame.read_text().iter().position(|r| r.starts_with("The first")).expect("read mode shows the file");
    a.reader.to_row(row);
    key(&mut a, KeyCode::Char('i'));
    assert_eq!(a.mode, Mode::Write, "write mode works on the file");
    typ(&mut a, "Added ");
    a.writer.edited_at(Instant::now() - Duration::from_secs(2));
    a.tick();
    assert!(std::fs::read_to_string(root.join(DESIGN)).unwrap().contains("Added The first"), "the file on disk has the line");
    key(&mut a, KeyCode::Esc);

    a.open_panel();
    a.focus = Focus::Claude;
    typ(&mut a, "what is in it");
    key(&mut a, KeyCode::Enter);
    until(&mut a, |a| a.threads.get(CHAT_KEY).is_some_and(|t| t.busy.is_none() && t.items.iter().any(|i| matches!(i, Item::Agent(_))))).await;
    let items = &a.threads[CHAT_KEY].items;
    assert!(items.iter().any(|i| matches!(i, Item::User(q) if q.contains("what is in it"))), "the chat shows the question");
    let answer: String = items.iter().filter_map(|i| if let Item::Agent(s) = i { Some(s.as_str()) } else { None }).collect();
    assert!(answer.contains("file link") && answer.contains("design.md"), "the file goes with the question: {answer}");
    let _ = std::fs::remove_dir_all(&root);
}
