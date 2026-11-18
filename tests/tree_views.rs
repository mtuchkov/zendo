//! The view strip of the file tree and the Notes view (docs/zendo/files/files-view-column.md).

mod common;
use common::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::time::{Duration, SystemTime};
use zendo::app::{App, Focus, Mode};
use zendo::docs::tree::View;

const STRUCTURE: &str = "[project]\nname = \"x\"\n[[entry]]\npath = \"docs/design\"\nmain = true\n";

fn files() -> Vec<(&'static str, &'static str)> {
    vec![
        ("project-structure.toml", STRUCTURE),
        ("docs/design/a.md", "# A\n"),
        ("docs/notes/2026-11-18-today.md", "# Today note\n"),
        ("docs/notes/2020-01-01-old.md", "# Old note\n"),
    ]
}

/// A repository with the older note a year old on disk, and the tree open.
fn tree_app(name: &str) -> (std::path::PathBuf, App) {
    let root = repo(name, &files());
    let old = std::fs::File::options().write(true).open(root.join("docs/notes/2020-01-01-old.md")).unwrap();
    old.set_modified(SystemTime::now() - Duration::from_secs(400 * 24 * 3600)).unwrap();
    let mut a = app(&root);
    a.on_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::ALT));
    (root, a)
}

fn screen(a: &mut App) -> Buffer {
    let mut term = Terminal::new(TestBackend::new(140, 40)).unwrap();
    term.draw(|f| a.draw(f)).unwrap();
    term.backend().buffer().clone()
}

fn rows(b: &Buffer) -> Vec<String> {
    (0..b.area.height).map(|y| (0..b.area.width).map(|x| b[(x, y)].symbol().to_string()).collect()).collect()
}

/// The letters of the strip, top to bottom, with their places.
fn strip(a: &mut App) -> Vec<(char, u16, u16)> {
    let b = screen(a);
    let tree = a.frame.pane_rects().iter().find(|(f, _)| *f == Focus::Tree).map(|(_, r)| *r).expect("the tree pane");
    (tree.y..tree.bottom()).filter_map(|y| b[(tree.x, y)].symbol().chars().next().filter(|c| "SDTWΔN".contains(*c)).map(|c| (c, tree.x, y))).collect()
}

fn click(a: &mut App, x: u16, y: u16) {
    a.on_mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column: x, row: y, modifiers: KeyModifiers::NONE });
}

// FILES-68: the views stand in a strip of letters on the left edge; a click on a letter shows its
// view.
#[tokio::test]
async fn the_views_stand_in_a_strip() {
    let (root, mut a) = tree_app("tree-strip");
    let s = strip(&mut a);
    assert_eq!(s.iter().map(|(c, _, _)| *c).collect::<String>(), "SDTN");
    assert!(s.windows(2).all(|w| w[1].2 > w[0].2 + 1), "one under the other, with a row between: {s:?}");
    let b = screen(&mut a);
    let (_, x, y) = s[0];
    assert_eq!(b[(x + 1, y)].symbol(), " ", "an empty column after the letters");
    let (_, x, y) = s[2];
    click(&mut a, x, y);
    assert_eq!(a.files.tree().view, View::Text);
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-70, FILES-71: the letters, and the accent color under the active one.
#[tokio::test]
async fn the_active_letter_has_the_accent_color() {
    let (root, mut a) = tree_app("tree-strip-color");
    let s = strip(&mut a);
    let b = screen(&mut a);
    let t = *a.theme();
    let at = |c: char| s.iter().find(|(l, _, _)| *l == c).map(|(_, x, y)| &b[(*x, *y)]).unwrap();
    assert_eq!(at('S').bg, t.acc, "the active view: structure");
    assert_eq!((at('D').fg, at('T').fg, at('N').fg), (t.acc2, t.acc2, t.acc2));
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-72: W and Δ only with a worktree; S only with main materials.
#[tokio::test]
async fn the_worktree_letters_need_a_worktree() {
    let root = repo("tree-strip-no-structure", &[("docs/a.md", "# A\n")]);
    let mut a = app(&root);
    a.on_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::ALT));
    assert_eq!(strip(&mut a).iter().map(|(c, _, _)| *c).collect::<String>(), "DTN");
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-73: a click on a letter keeps the view for the repository.
#[tokio::test]
async fn a_click_on_a_letter_keeps_the_view() {
    let (root, mut a) = tree_app("tree-strip-keep");
    let (_, x, y) = strip(&mut a)[1];
    click(&mut a, x, y);
    assert_eq!(a.files.tree().view, View::Docs);
    let b = app(&root);
    assert_eq!(b.state.tree_view, "docs", "the state keeps the view");
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-74: `v` shows the hint with the names over the tree for 2.5 seconds; a click on a name
// shows its view.
#[tokio::test]
async fn v_shows_the_hint_for_2_5_seconds() {
    let (root, mut a) = tree_app("tree-hint");
    let width = |a: &App| a.frame.pane_rects().iter().find(|(f, _)| *f == Focus::Tree).map(|(_, r)| r.width).unwrap();
    screen(&mut a);
    let before = width(&a);
    key(&mut a, KeyCode::Char('v'));
    let shown = rows(&screen(&mut a)).join("\n");
    assert!(shown.contains("S structure") && shown.contains("N notes"), "{shown}");
    assert_eq!(width(&a), before, "the tree keeps its width");
    let r = a.frame.view_buttons().iter().find(|(_, v)| *v == View::Text).map(|(r, _)| *r).unwrap();
    click(&mut a, r.x + 3, r.y);
    assert_eq!(a.files.tree().view, View::Text, "a click on a name");
    assert!(rows(&screen(&mut a)).join("\n").contains("S structure"), "a click does not close the hint");
    std::thread::sleep(Duration::from_millis(2600));
    assert!(!rows(&screen(&mut a)).join("\n").contains("S structure"), "the hint closes after 2.5 s");
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-75: the head names the view and its key.
#[tokio::test]
async fn the_head_names_the_view() {
    let (root, mut a) = tree_app("tree-head");
    assert!(rows(&screen(&mut a)).iter().any(|r| r.contains("structure · v")));
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-69, FILES-76: `o` opens the tree in the Notes view, by age; `enter` opens the note.
#[tokio::test]
async fn the_notes_view_lists_the_notes_by_age() {
    let (root, mut a) = tree_app("tree-notes");
    a.focus = Focus::Note;
    key(&mut a, KeyCode::Char('o'));
    assert_eq!((a.files.tree().view, a.focus, a.mode), (View::Notes, Focus::Tree, Mode::Read));
    let b = screen(&mut a);
    let tree = a.frame.pane_rects().iter().find(|(f, _)| *f == Focus::Tree).map(|(_, r)| *r).unwrap();
    let text: String = (tree.y..tree.bottom()).map(|y| (tree.x..tree.right()).map(|x| b[(x, y)].symbol().to_string()).collect::<String>() + "\n").collect();
    let (today, older) = (text.find("TODAY").expect("Today"), text.find("OLDER").expect("Older"));
    let (t, o) = (text.find("Today note").unwrap(), text.find("Old note").unwrap());
    assert!(today < t && t < older && older < o, "{text}");
    key(&mut a, KeyCode::Enter);
    assert_eq!(a.files.current().cloned(), Some(root.join("docs/notes/2026-11-18-today.md")));
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-76: `o` opens the tree also when it is closed.
#[tokio::test]
async fn o_opens_the_notes_view() {
    let root = repo("tree-o", &files());
    let mut a = app(&root);
    assert!(!a.state.tree);
    key(&mut a, KeyCode::Char('o'));
    assert!(a.state.tree && a.files.tree().view == View::Notes && a.focus == Focus::Tree);
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-77: the Notes view has no `[+]` and `[-]`; `n` writes a new note there.
#[tokio::test]
async fn the_notes_view_has_no_fold_buttons() {
    let (root, mut a) = tree_app("tree-notes-fold");
    key(&mut a, KeyCode::Char('o'));
    screen(&mut a);
    assert!(a.frame.tree_fold().is_empty());
    key(&mut a, KeyCode::Char('*'));
    key(&mut a, KeyCode::Char('n'));
    assert!(a.writer.float().is_some(), "n writes a new note");
    let _ = std::fs::remove_dir_all(&root);
}
