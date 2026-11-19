//! The view strip of the file tree and the Notes view (docs/zendo/files/files-view-column.md).

mod common;
use common::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::Color;
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
    // The first frame asks the workers; the tree walks on its thread (IO-1).
    term.draw(|f| a.draw(f)).unwrap();
    a.settle_tree();
    a.settle_facts();
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
    (tree.y..tree.bottom()).filter_map(|y| b[(tree.x, y)].symbol().chars().next().filter(|c| "SDTW∆N".contains(*c)).map(|c| (c, tree.x, y))).collect()
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
    assert_eq!(s.iter().map(|(c, _, _)| *c).collect::<String>(), "SDTW∆N", "session 0 shows the changes of the main checkout");
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

// FILES-72, FILES-81: W and ∆ only with a worktree or in session 0; S only with main materials.
#[tokio::test]
async fn the_worktree_letters_need_a_worktree() {
    let root = repo("tree-strip-no-structure", &[("docs/a.md", "# A\n")]);
    let mut a = app(&root);
    a.on_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::ALT));
    assert_eq!(strip(&mut a).iter().map(|(c, _, _)| *c).collect::<String>(), "DTW∆N", "session 0");
    a.state.session = 1;
    a.scan_tree();
    assert_eq!(strip(&mut a).iter().map(|(c, _, _)| *c).collect::<String>(), "DTN", "session 1 without .worktrees/1");
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-73, FILES-66: a click on a letter keeps the view until the quit, as `v` does; a new start
// shows the structure view.
#[tokio::test]
async fn a_click_on_a_letter_keeps_the_view() {
    let (root, mut a) = tree_app("tree-strip-keep");
    let (_, x, y) = strip(&mut a)[1];
    click(&mut a, x, y);
    assert_eq!(a.files.tree().view, View::Docs);
    a.scan_tree();
    a.apply_tree_view();
    assert_eq!(a.files.tree().view, View::Docs, "the view stays while zendo runs");
    let mut b = app(&root);
    b.toggle_tree();
    assert_eq!(b.files.tree().view, View::Structure, "a new start");
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

/// The chars on the screen inside a pane, row by row.
fn pane_text(a: &mut App, pane: Focus) -> Vec<String> {
    let b = screen(a);
    let r = a.frame.pane_rects().iter().find(|(f, _)| *f == pane).map(|(_, r)| *r).expect("the pane");
    (r.y..r.bottom()).map(|y| (r.x..r.right()).map(|x| b[(x, y)].symbol().to_string()).collect()).collect()
}

// FILES-78: the tree draws only chars that every font of `zendo setup fonts` has, so that
// no row is wider than its cells: no geometric shapes, no Greek Δ.
#[tokio::test]
async fn the_tree_draws_only_chars_of_every_font() {
    let (root, mut a) = tree_app("tree-glyphs");
    a.open_file(root.join("docs/design/a.md"));
    a.files.reveal_current();
    for keys in ["", "v"] {
        for c in keys.chars() {
            key(&mut a, KeyCode::Char(c));
        }
        let text = pane_text(&mut a, Focus::Tree).join("\n");
        let odd: Vec<char> = text.chars().filter(|c| ('\u{25A0}'..='\u{25FF}').contains(c) || *c == 'Δ').collect();
        assert!(odd.is_empty(), "{odd:?} in\n{text}");
        assert!(text.contains('•'), "the open file: {text}");
    }
    let _ = std::fs::remove_dir_all(&root);
}

// A long name of the project ends with `…` and keeps a space before the name of the view.
#[tokio::test]
async fn a_long_project_name_does_not_touch_the_view_name() {
    let structure = "[project]\nname = \"digital-operator-platform-shell\"\n[[entry]]\npath = \"docs/design\"\nmain = true\n";
    let root = repo("tree-long-name", &[("project-structure.toml", structure), ("docs/design/a.md", "# A\n")]);
    let mut a = app(&root);
    a.on_key(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::ALT));
    let head = pane_text(&mut a, Focus::Tree)[0].clone();
    assert!(head.contains("… structure · v") || head.contains("…  structure · v"), "{head}");
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-68: in a low pane the letters stand without the empty rows, so that `N` still shows.
#[tokio::test]
async fn a_low_tree_keeps_every_letter() {
    let (root, mut a) = tree_app("tree-low");
    let mut term = Terminal::new(TestBackend::new(100, 13)).unwrap();
    term.draw(|f| a.draw(f)).unwrap();
    let b = term.backend().buffer().clone();
    let tree = a.frame.pane_rects().iter().find(|(f, _)| *f == Focus::Tree).map(|(_, r)| *r).unwrap();
    let letters: String = (tree.y..tree.bottom()).filter_map(|y| b[(tree.x, y)].symbol().chars().next().filter(|c| "SDTW∆N".contains(*c))).collect();
    assert_eq!(letters, "SDTW∆N");
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-74: the hint stands on a box at 80% opacity: the tree under it shows at 20%, mixed with the
// background of the tree; the names stand over it.
#[tokio::test]
async fn the_hint_dims_the_tree() {
    let (root, mut a) = tree_app("tree-hint-dim");
    key(&mut a, KeyCode::Char('v'));
    let b = screen(&mut a);
    let t = *a.theme();
    let names = a.frame.view_buttons().to_vec();
    let right = names.iter().map(|(r, _)| r.right()).max().unwrap() + 1;
    let bottom = names.iter().map(|(r, _)| r.bottom()).max().unwrap();
    let x0 = names[0].0.x;
    let Color::Rgb(pr, pg, pb) = t.panel else { panic!("an RGB theme") };
    let near = |c: Color| matches!(c, Color::Rgb(r, g, b) if [(r, pr), (g, pg), (b, pb)].iter().all(|(v, p)| (*v as i16 - *p as i16).abs() <= 52));
    for y in names[0].0.y..bottom {
        for x in x0..right {
            // The names take clicks; the dim name of W does not (FILES-82).
            if names.iter().any(|(r, _)| r.contains((x, y).into())) || b[(x0, y)].symbol() == "W" {
                continue;
            }
            let c = &b[(x, y)];
            assert_eq!(c.bg, t.panel, "the box has the background of the tree at {x},{y}");
            assert!(c.symbol() == " " || c.symbol() == "─" || near(c.fg), "the tree shows at 20% at {x},{y}: {:?} {:?}", c.symbol(), c.fg);
        }
    }
    let _ = std::fs::remove_dir_all(&root);
}
