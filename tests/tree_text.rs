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
