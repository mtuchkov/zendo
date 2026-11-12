//! Write mode saves by itself, and the alt keys that work in every terminal.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};
use zendo::app::{Focus, Mode};

const NOTE: &str = "docs/notes/2026-11-12-idea.md";

fn files() -> Vec<(&'static str, &'static str)> {
    vec![(NOTE, "# Idea\n\nThe first paragraph of the idea.\n"), ("docs/zendo/zendo.md", "# Zendo\n")]
}

fn alt(a: &mut zendo::app::App, c: char) {
    a.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT));
}

// NOTE-3: write mode has no save key: a pause in the typing saves the file.
#[tokio::test]
async fn writing_saves_after_a_pause() {
    let root = repo("flow-autosave", &files());
    let mut a = app(&root);
    draw(&mut a, 120, 30);
    a.read_cursor = a.read_text.iter().position(|r| r.starts_with("The first")).unwrap();
    key(&mut a, KeyCode::Char('i'));
    draw(&mut a, 120, 30);
    typ(&mut a, "New ");
    assert_eq!(a.keys.action(zendo::keys::Pane::Write, &KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)), None, "no save key");
    a.tick();
    assert!(!std::fs::read_to_string(root.join(NOTE)).unwrap().contains("New "), "not while typing");
    a.last_edit = Instant::now() - Duration::from_secs(2);
    a.tick();
    assert!(std::fs::read_to_string(root.join(NOTE)).unwrap().contains("New "), "saved after a pause");
    let _ = std::fs::remove_dir_all(&root);
}

// NOTE-18, NOTE-25, PANES: alt-a adds to the chat, alt-s says it here, alt-b opens the file tree, in
// read mode and in write mode.
#[tokio::test]
async fn alt_a_s_b_work_in_read_and_write_mode() {
    let root = repo("flow-alt-keys", &files());
    for write in [false, true] {
        let mut a = app(&root);
        draw(&mut a, 120, 30);
        a.read_cursor = a.read_text.iter().position(|r| r.starts_with("The first")).unwrap();
        if write {
            key(&mut a, KeyCode::Char('i'));
            draw(&mut a, 120, 30);
        }
        let mode = a.mode;
        alt(&mut a, 'a');
        assert!(a.chips.last().is_some_and(|c| c.label.contains("The first")), "alt-a, write {write}");
        a.chips.clear();
        a.input.take();
        a.focus = Focus::Note;
        alt(&mut a, 's');
        assert!(a.say.is_some() && a.input.text().starts_with("⟦write here :"), "alt-s, write {write}");
        a.say = None;
        a.input.take();
        a.focus = Focus::Note;
        let tree = a.state.tree;
        alt(&mut a, 'b');
        assert_ne!(a.state.tree, tree, "alt-b, write {write}");
        assert_eq!(a.mode, if write { Mode::Write } else { mode });
    }
    let _ = std::fs::remove_dir_all(&root);
}

// NOTE-32: in write mode the rows wrap as in read mode, the cursor row too: it shows the source of
// its own words, and the rows around it stay.
#[tokio::test]
async fn the_cursor_row_keeps_its_words() {
    let rel = "docs/notes/2026-11-12-wrap.md";
    let long = "# Wrap\n\nA zendo is the hall where Zen practitioners sit for zazen, seated meditation.\nDeep thinking about the **architecture** of a project feels much the same: you sit\nwith a problem until it settles. zendo is a minimal, distraction-free place for\nthat work.\n";
    let root = repo("flow-cursor-row", &[(rel, long)]);
    let mut a = app(&root);
    let read = draw(&mut a, 100, 30);
    a.read_cursor = a.read_text.iter().position(|r| r.contains("architecture")).unwrap();
    key(&mut a, KeyCode::Char('i'));
    let write = draw(&mut a, 100, 30);
    let body = |s: &Vec<String>| s[..s.len() - 2].iter().map(|l| l.trim_end().to_string()).collect::<Vec<_>>();
    let (read, write) = (body(&read), body(&write));
    let diff: Vec<usize> = (0..read.len()).filter(|i| read[*i] != write[*i]).collect();
    assert_eq!(diff.len(), 1, "only the cursor row differs:\n{read:#?}\n{write:#?}");
    let row = diff[0];
    assert!(write[row].contains("**architecture**"), "{}", write[row]);
    assert_eq!(write[row].replace("**", "").trim(), read[row].trim(), "the same words on the row");
    let y = a.cursor_y;
    key(&mut a, KeyCode::Down);
    draw(&mut a, 100, 30);
    assert_eq!(a.cursor_y, y + 1, "down goes to the next row of the paragraph");
    key(&mut a, KeyCode::Up);
    draw(&mut a, 100, 30);
    assert_eq!(a.cursor_y, y);
    let _ = std::fs::remove_dir_all(&root);
}
