//! The text view of the file tree, and a code file in the note pane.

mod common;
use common::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::KeyCode;
use zendo::app::Focus;
use zendo::docs::tree::View;

// FILES-20, FILES-21: `v` comes to the text view: it lists code files; a code file opens in the note pane in the colors
// of its language.
#[test]
fn the_text_view_opens_a_code_file_with_its_colors() {
    let root = repo("tree-text", &[("docs/notes/2026-11-12-a.md", "# A\n"), ("src/main.rs", "fn main() {\n    let x = 42;\n}\n")]);
    let mut a = app(&root);
    a.toggle_tree();
    a.focus = Focus::Tree;
    while a.tree.view != View::Text {
        key(&mut a, KeyCode::Char('v'));
    }
    draw(&mut a, 140, 30);
    a.tree.reveal(std::path::Path::new("src/main.rs"));
    key(&mut a, KeyCode::Enter);
    assert!(a.current.as_ref().is_some_and(|p| p.ends_with("src/main.rs")), "{:?}", a.current);
    let mut term = Terminal::new(TestBackend::new(140, 30)).unwrap();
    term.draw(|f| zendo::ui::draw(f, &mut a)).unwrap();
    let buf = term.backend().buffer().clone();
    let cells: Vec<_> = (0..30u16).flat_map(|y| (0..140u16).map(move |x| (x, y))).collect();
    let at = cells.windows(2).find(|w| buf[w[0]].symbol() == "f" && buf[w[1]].symbol() == "n").map(|w| w[0]).expect("fn on the screen");
    assert_eq!(buf[at].fg, a.theme().acc, "`fn` has the keyword color");
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-22, FILES-23: a code file shows 120 chars wide with line numbers; a wrapped line has its
// number on its first row only; write mode shows it the same way.
#[test]
fn a_code_file_has_line_numbers_wraps_and_writes_as_it_reads() {
    let long = format!("    let words = \"{}\";", "word ".repeat(40).trim_end());
    let body = format!("fn main() {{\n{long}\n}}\n");
    let root = repo("code-view", &[("docs/notes/2026-11-12-a.md", "# A\n"), ("src/main.rs", &body)]);
    let mut a = app(&root);
    a.open_file(root.join("src/main.rs"));
    let screen = draw(&mut a, 200, 30);
    let row = |s: &[String], text: &str| s.iter().position(|r| r.contains(text)).unwrap_or_else(|| panic!("{text}: {s:#?}"));
    let first = row(&screen, "let words");
    assert!(screen[first].trim_start().starts_with("2 "), "the number of the line: {}", screen[first]);
    let next = &screen[first + 1];
    assert!(next.contains("word") && !next.trim_start().starts_with('3'), "a wrapped row has no number: {next}");
    assert!(screen[first + 2].trim_start().starts_with("3 }"), "the next line has the next number: {}", screen[first + 2]);
    let width = screen[first].trim_end().len() - screen[first].find("let").unwrap() + 4;
    assert!((110..=120).contains(&width), "about 120 chars of code: {width}");
    key(&mut a, KeyCode::Char('i'));
    let written = draw(&mut a, 200, 30);
    let body_of = |s: &[String]| s[row(s, "fn main")..row(s, "3 }") + 1].to_vec();
    assert_eq!(body_of(&written), body_of(&screen), "write mode looks as read mode");
    let _ = std::fs::remove_dir_all(&root);
}
