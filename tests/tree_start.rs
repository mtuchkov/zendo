//! The view of the file tree at the start (FILES-66): the structure view, whatever view the user
//! left at the last quit.

mod common;
use common::{app, key, repo};
use ratatui::crossterm::event::KeyCode;
use zendo::docs::tree::View;

const STRUCTURE: &str = "[[entry]]\npath = \"docs\"\nlabel = \"Docs\"\nmain = true\n";

// FILES-66
#[tokio::test]
async fn the_tree_starts_in_the_structure_view() {
    let root = repo("tree-start", &[("project-structure.toml", STRUCTURE), ("docs/a.md", "# a\n"), ("src/b.rs", "fn b() {}\n")]);
    let mut a = app(&root);
    a.toggle_tree();
    assert_eq!(a.files.tree().view, View::Structure, "the default view");
    key(&mut a, KeyCode::Char('v'));
    key(&mut a, KeyCode::Char('v'));
    assert_eq!(a.files.tree().view, View::Text);
    a.cli.stop_all();
    drop(a);
    let mut a = app(&root);
    assert_eq!(a.files.tree().view, View::Structure, "the next start shows the structure view again");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-66
#[tokio::test]
async fn without_main_materials_the_tree_starts_in_the_docs_view() {
    let root = repo("tree-start-docs", &[("docs/a.md", "# a\n")]);
    let mut a = app(&root);
    a.toggle_tree();
    key(&mut a, KeyCode::Char('v'));
    assert_ne!(a.files.tree().view, View::Docs);
    a.cli.stop_all();
    drop(a);
    let mut a = app(&root);
    assert_eq!(a.files.tree().view, View::Docs, "no structure: the docs view");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}
