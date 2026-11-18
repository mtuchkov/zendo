//! Find with the selection (docs/zendo/find.md, FIND-19, FIND-20): `/` in read mode and `alt-/` put
//! the selected text in the query, each line break as `⏎`, and the query finds the text across lines.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::{Path, PathBuf};
use zendo::app::{App, Focus, Mode};

const A: &str = "docs/a.md";
const BODY: &str = "# Alpha\n\nFirst line of para\nsecond line of para\n\n- item one\n- item two\n\nThis is a very long line that should wrap around the narrow column for sure because it is long enough.\n";

fn open(name: &str) -> (PathBuf, App) {
    let root = repo(
        name,
        &[(A, BODY), ("docs/b.md", "# Beta\n\nitem one item two\n")],
    );
    let mut a = app(&root);
    a.focus = Focus::Note;
    a.open_file(root.join(A));
    a.tick();
    draw(&mut a, 80, 40);
    (root, a)
}

/// The row of read mode that starts with `text`.
fn row(a: &App, text: &str) -> usize {
    a.frame
        .read_text()
        .iter()
        .position(|l| l.trim_start().starts_with(text))
        .unwrap_or_else(|| panic!("no row {text}"))
}

/// Select in read mode from the start of row `from` to the end of row `to`, and press `/`.
fn select_and_find(a: &mut App, from: usize, to: usize) {
    a.reader.place(from, 0);
    a.reader.select();
    a.reader
        .place(to, a.frame.read_text()[to].trim_end().chars().count() - 1);
    key(a, KeyCode::Char('/'));
}

fn cleanup(root: &Path) {
    let _ = std::fs::remove_dir_all(root);
}

// FIND-19, FIND-20: two list items: the query has `⏎` between them, and finds the file at the
// first item.
#[tokio::test]
async fn slash_on_a_selection_keeps_its_line_breaks() {
    let (root, mut a) = open("find-sel-lines");
    let (one, two) = (row(&a, "• item one"), row(&a, "• item two"));
    select_and_find(&mut a, one, two);
    assert_eq!(
        (a.mode, a.files.find().query.text()),
        (Mode::Find, "• item one⏎• item two")
    );
    let item = a.files.find().items.first().expect("a match");
    assert_eq!(
        (item.path.clone(), item.snippets.first().map(|s| s.line)),
        (root.join(A), Some(5)),
        "not b.md: its text is one line"
    );
    key(&mut a, KeyCode::Enter);
    assert_eq!(
        (a.mode, a.reader.row()),
        (Mode::Read, one),
        "enter opens the file at the match"
    );
    cleanup(&root);
}

// FIND-19: a line that read mode wraps joins with a space, not with `⏎`.
#[tokio::test]
async fn a_wrapped_line_joins_with_a_space() {
    let (root, mut a) = open("find-sel-wrap");
    let first = row(&a, "This is a very long line");
    select_and_find(&mut a, first, first + 1);
    let q = a.files.find().query.text().to_string();
    assert!(!q.contains('⏎') && q.ends_with("long enough."), "{q}");
    assert_eq!(
        a.files
            .find()
            .items
            .iter()
            .find(|i| !i.snippets.is_empty())
            .map(|i| i.path.clone()),
        Some(root.join(A))
    );
    cleanup(&root);
}

// FIND-19: `alt-/` in write mode puts the selected source lines in the query.
#[tokio::test]
async fn alt_slash_in_write_mode_takes_the_selection() {
    let (root, mut a) = open("find-sel-write");
    key(&mut a, KeyCode::Char('i'));
    draw(&mut a, 80, 40);
    let e = a.writer.editor_mut();
    assert_eq!(e.lines[5], "- item one");
    (e.anchor, e.row, e.col) = (Some((5, 0)), 6, e.lines[6].chars().count());
    a.on_key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::ALT));
    assert_eq!(
        (a.mode, a.files.find().query.text()),
        (Mode::Find, "- item one⏎- item two")
    );
    cleanup(&root);
}

// FIND-1: without a selection the query stays empty.
#[tokio::test]
async fn slash_without_a_selection_opens_an_empty_query() {
    let (root, mut a) = open("find-sel-none");
    key(&mut a, KeyCode::Char('/'));
    assert_eq!((a.mode, a.files.find().query.text()), (Mode::Find, ""));
    cleanup(&root);
}

// FIND-20: a query across lines that starts with `#` finds the text, not a tag.
#[tokio::test]
async fn a_heading_in_the_selection_is_no_tag() {
    let (root, mut a) = open("find-sel-heading");
    key(&mut a, KeyCode::Char('i'));
    draw(&mut a, 80, 40);
    let e = a.writer.editor_mut();
    (e.anchor, e.row, e.col) = (Some((0, 0)), 2, 10);
    a.on_key(KeyEvent::new(KeyCode::Char('/'), KeyModifiers::ALT));
    assert_eq!(a.files.find().query.text(), "# Alpha⏎⏎First line");
    assert_eq!(
        a.files
            .find()
            .items
            .iter()
            .find(|i| !i.snippets.is_empty())
            .map(|i| i.path.clone()),
        Some(root.join(A))
    );
    cleanup(&root);
}
