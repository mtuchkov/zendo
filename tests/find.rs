//! Find (docs/zendo/find.md): `alt-/` and `/` open the find panel; with an empty query it lists the
//! recent files, with a query the files by path and then by text; the preview shows the selection.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::{Path, PathBuf};
use zendo::app::{App, FilePrompt, Focus, Mode};

const A: &str = "docs/a.md";
const B: &str = "docs/b.md";
const C: &str = "docs/c.md";

fn files() -> Vec<(&'static str, &'static str)> {
    vec![
        (A, "# Alpha\n\nThe first file.\n"),
        (B, "# Beta\n\nThe second file.\n"),
        (C, "# Gamma\n\nThe third file.\n"),
        ("docs/notes/2026-11-12-idea.md", "# Idea\n\nA note.\n"),
    ]
}

fn alt(a: &mut App, c: char) {
    a.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT));
}

/// Open a file as the user does, and let zendo see it.
fn open(a: &mut App, root: &Path, rel: &str) {
    a.focus = Focus::Note;
    a.open_file(root.join(rel));
    a.tick();
}

/// The paths of the list, relative to the repository.
fn listed(a: &App, root: &Path) -> Vec<String> {
    a.find.items.iter().map(|i| i.path.strip_prefix(root).unwrap().display().to_string()).collect()
}

fn find_app(name: &str) -> (PathBuf, App) {
    let root = repo(name, &files());
    let a = app(&root);
    (root, a)
}

// FIND-1: alt-/ opens the find panel in write mode (no `/` is typed), in the chat, in the file
// tree, and `/` in read mode.
#[tokio::test]
async fn alt_slash_opens_find_from_every_pane() {
    let (root, _) = find_app("find-every-pane");
    type Setup = (&'static str, fn(&mut App));
    let setups: [Setup; 4] = [
        ("write", |a| key(a, KeyCode::Char('i'))),
        ("chat", |a| key(a, KeyCode::Char('c'))),
        ("tree", |a| alt(a, 'b')),
        ("read", |_| {}),
    ];
    for (name, setup) in setups {
        let mut a = app(&root);
        open(&mut a, &root, A);
        draw(&mut a, 140, 40);
        setup(&mut a);
        draw(&mut a, 140, 40);
        if name == "read" {
            key(&mut a, KeyCode::Char('/'));
        } else {
            alt(&mut a, '/');
        }
        assert_eq!((a.mode, a.find.query.as_str()), (Mode::Find, ""), "{name}");
        assert!(!std::fs::read_to_string(root.join(A)).unwrap().contains('/'), "{name}: no `/` typed");
        a.cli.stop_all();
    }
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-2: with an empty query the list has the recent files, the most recent first, without the
// open file; the first row is selected.
#[tokio::test]
async fn an_empty_query_lists_the_recent_files() {
    let (root, mut a) = find_app("find-recent");
    for f in [A, B, C] {
        open(&mut a, &root, f);
    }
    alt(&mut a, '/');
    assert_eq!(listed(&a, &root), vec![B, A]);
    assert_eq!(a.find.sel, 0);
    key(&mut a, KeyCode::Enter);
    assert_eq!((a.mode, a.current.clone()), (Mode::Read, Some(root.join(B))), "alt-/ enter goes back to the file before");
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-3: a move in the list shows the selected file in the preview, as read mode shows it.
#[tokio::test]
async fn moving_in_the_list_shows_the_preview() {
    let (root, mut a) = find_app("find-preview");
    for f in [A, B, C] {
        open(&mut a, &root, f);
    }
    alt(&mut a, '/');
    key(&mut a, KeyCode::Down);
    let screen = draw(&mut a, 140, 40);
    let row = screen.iter().position(|l| l.contains("The first file.")).unwrap_or_else(|| panic!("no preview:\n{}", screen.join("\n")));
    assert!(screen[..row].iter().any(|l| l.matches("Alpha").count() >= 1), "the title of the preview above its text");
    assert!(!screen.join("\n").contains("The second file."), "only the selected file");
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-4: the recent files are in the state of the repository and stay after a restart.
#[tokio::test]
async fn the_recent_files_stay_after_a_restart() {
    let (root, mut a) = find_app("find-restart");
    open(&mut a, &root, A);
    open(&mut a, &root, B);
    a.cli.stop_all();
    drop(a);
    let mut a = app(&root);
    open(&mut a, &root, C);
    alt(&mut a, '/');
    assert_eq!(listed(&a, &root), vec![B, A]);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-5: a row shows the title, the path and when the user opened the file.
#[tokio::test]
async fn the_row_shows_title_path_and_time() {
    let (root, mut a) = find_app("find-row");
    open(&mut a, &root, A);
    open(&mut a, &root, B);
    alt(&mut a, '/');
    let screen = draw(&mut a, 140, 40);
    let now = chrono::Local::now().format("%H:%M").to_string();
    let row = screen.iter().find(|l| l.contains("Alpha")).unwrap_or_else(|| panic!("{}", screen.join("\n")));
    assert!(row.contains(A) && row.contains(&now), "{row}");
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-6: a file that only the preview of the file tree showed is not a recent file.
#[tokio::test]
async fn a_tree_preview_is_not_recent() {
    let (root, mut a) = find_app("find-tree-preview");
    open(&mut a, &root, A);
    a.focus = Focus::Tree;
    a.open_file(root.join(B));
    a.tick();
    open(&mut a, &root, C);
    alt(&mut a, '/');
    assert_eq!(listed(&a, &root), vec![A]);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-7: a recent file that is gone does not show.
#[tokio::test]
async fn a_deleted_file_is_not_recent() {
    let (root, mut a) = find_app("find-deleted");
    open(&mut a, &root, A);
    open(&mut a, &root, B);
    open(&mut a, &root, C);
    std::fs::remove_file(root.join(A)).unwrap();
    alt(&mut a, '/');
    assert_eq!(listed(&a, &root), vec![B]);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-8: a file that the user renames in zendo keeps its place in the recent files.
#[tokio::test]
async fn a_rename_keeps_the_recent_place() {
    let (root, mut a) = find_app("find-rename");
    open(&mut a, &root, A);
    open(&mut a, &root, B);
    open(&mut a, &root, C);
    a.file_prompt = Some(FilePrompt::Rename(PathBuf::from(A), "a.md".into()));
    for _ in 0.."a.md".len() {
        key(&mut a, KeyCode::Backspace);
    }
    typ(&mut a, "alpha.md");
    key(&mut a, KeyCode::Enter);
    assert!(root.join("docs/alpha.md").exists());
    alt(&mut a, '/');
    assert_eq!(listed(&a, &root), vec![B, "docs/alpha.md"]);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-9: a query lists the files whose path has every word, then the markdown files whose text
// has it, without regard to letter case.
#[tokio::test]
async fn a_query_finds_paths_then_text() {
    let root = repo("find-query", &[("docs/design.md", "# Design\n\nWe drink TEA here.\n"), ("docs/tea-notes.md", "# Notes\n\nNothing.\n"), ("src/tea.rs", "fn main() {}\n")]);
    let mut a = app(&root);
    alt(&mut a, '/');
    typ(&mut a, "tea");
    let got = listed(&a, &root);
    assert_eq!(got.len(), 3, "{got:?}");
    assert!(got[..2].contains(&"docs/tea-notes.md".to_string()) && got[..2].contains(&"src/tea.rs".to_string()), "paths first: {got:?}");
    assert_eq!(got[2], "docs/design.md", "then the text");
    typ(&mut a, " notes");
    assert_eq!(listed(&a, &root), vec!["docs/tea-notes.md"], "every word of the path");
    let _ = std::fs::remove_dir_all(&root);
}

fn long_design() -> String {
    let mut s = "# Design\n\n".to_string();
    for i in 1..=60 {
        s += &format!("Paragraph {i} of the design.\n\n");
    }
    s + "The coffee machine is eventually consistent.\n"
}

// FIND-10, FIND-11: the preview of a match in the text shows the line of the match; enter opens
// the file at that line.
#[tokio::test]
async fn the_preview_shows_the_match_and_enter_opens_at_it() {
    let design = long_design();
    let root = repo("find-match", &[("docs/design.md", design.as_str())]);
    let mut a = app(&root);
    alt(&mut a, '/');
    typ(&mut a, "coffee");
    assert_eq!(listed(&a, &root), vec!["docs/design.md"]);
    let screen = draw(&mut a, 140, 40).join("\n");
    assert!(screen.contains("The coffee machine is eventually consistent."), "the line of the match:\n{screen}");
    key(&mut a, KeyCode::Enter);
    assert_eq!((a.mode, a.current.clone()), (Mode::Read, Some(root.join("docs/design.md"))));
    draw(&mut a, 140, 40);
    assert!(a.read_text[a.read_cursor].contains("coffee"), "the cursor on the match: {}", a.read_text[a.read_cursor]);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-12: esc closes the panel and changes nothing.
#[tokio::test]
async fn esc_changes_nothing() {
    let (root, mut a) = find_app("find-esc");
    open(&mut a, &root, A);
    open(&mut a, &root, B);
    draw(&mut a, 140, 40);
    key(&mut a, KeyCode::Char('i'));
    draw(&mut a, 140, 40);
    let cursor = (a.editor.row, a.editor.col);
    alt(&mut a, '/');
    key(&mut a, KeyCode::Down);
    key(&mut a, KeyCode::Esc);
    assert_eq!((a.mode, a.focus, a.current.clone(), (a.editor.row, a.editor.col)), (Mode::Write, Focus::Note, Some(root.join(B)), cursor));
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-13: in a narrow panel the preview stands under the list.
#[tokio::test]
async fn a_narrow_panel_shows_the_preview_under_the_list() {
    let (root, mut a) = find_app("find-narrow");
    open(&mut a, &root, A);
    open(&mut a, &root, B);
    alt(&mut a, '/');
    for (w, under) in [(140, false), (90, true)] {
        let screen = draw(&mut a, w, 40);
        let at = |s: &str| screen.iter().enumerate().find_map(|(y, l)| l.find(s).map(|x| (y, l[..x].chars().count()))).unwrap_or_else(|| panic!("no {s}:\n{}", screen.join("\n")));
        let ((path_y, path_x), (text_y, text_x)) = (at(A), at("The first file."));
        assert_eq!(text_x < path_x, under, "{w} columns:\n{}", screen.join("\n"));
        assert!(text_y > path_y, "{w} columns: the text of the preview is under its title");
    }
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-16: the text of a file in a hidden folder or that git ignores is not searched.
#[tokio::test]
async fn ignored_files_are_not_searched() {
    let root = repo("find-ignored", &[(".gitignore", "build/\n"), ("build/out.md", "# Out\n\nzebra\n"), (".hidden/x.md", "# X\n\nzebra\n"), ("docs/d.md", "# D\n\nzebra\n")]);
    let mut a = app(&root);
    alt(&mut a, '/');
    typ(&mut a, "zebra");
    assert_eq!(listed(&a, &root), vec!["docs/d.md"]);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-17: `/` in the notes list keeps the search of the notes.
#[tokio::test]
async fn slash_in_the_notes_list_searches_the_notes() {
    let (root, mut a) = find_app("find-notes-list");
    key(&mut a, KeyCode::Char('o'));
    key(&mut a, KeyCode::Char('/'));
    assert_eq!(a.mode, Mode::Search);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-6: a new note from the float is a recent file when the float closes.
#[tokio::test]
async fn a_note_from_the_float_is_recent() {
    let (root, mut a) = find_app("find-float");
    open(&mut a, &root, A);
    open(&mut a, &root, B);
    alt(&mut a, 'n');
    typ(&mut a, "Tea");
    key(&mut a, KeyCode::Esc);
    alt(&mut a, '/');
    let got = listed(&a, &root);
    assert!(got[0].ends_with("-tea.md") && got[1] == A, "{got:?}");
    let _ = std::fs::remove_dir_all(&root);
}
