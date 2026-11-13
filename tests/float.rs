//! The new note in a float (docs/zendo/float.md): alt-n from every pane, only the float on the
//! screen, esc saves and gives the screen back.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use zendo::app::{App, Focus, Mode};

const NOTE: &str = "docs/notes/2026-11-12-idea.md";
const DESIGN: &str = "docs/design.md";

fn files() -> Vec<(&'static str, &'static str)> {
    let design = "# Design\n\n".to_string() + &(1..=30).map(|i| format!("Line {i} of the design.\n\n")).collect::<String>();
    vec![(NOTE, "# Idea\n\nThe first paragraph of the idea.\n"), (DESIGN, Box::leak(design.into_boxed_str()))]
}

fn alt(a: &mut App, c: char) {
    a.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT));
}

fn in_float(a: &App) -> bool {
    a.float.is_some() && a.mode == Mode::Write && a.editor.text() == "# " && a.editor_path.is_none()
}

/// The notes folder after the test: the files in it.
fn notes(root: &std::path::Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(root.join("docs/notes")).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    v.sort();
    v
}

// FLOAT-1: alt-n opens a new note in write mode in a float, from read mode, write mode, the chat,
// the file tree and the delta.
#[tokio::test]
async fn alt_n_opens_a_float_from_every_pane() {
    let root = repo("float-every-pane", &files());
    type Setup = (&'static str, fn(&mut App));
    let setups: [Setup; 5] = [
        ("read", |_| {}),
        ("write", |a| key(a, KeyCode::Char('i'))),
        ("chat", |a| key(a, KeyCode::Char('c'))),
        ("tree", |a| alt(a, 'b')),
        ("delta", |a| key(a, KeyCode::Char('d'))),
    ];
    for (name, setup) in setups {
        let mut a = app(&root);
        a.open_file(root.join(NOTE));
        draw(&mut a, 160, 40);
        setup(&mut a);
        draw(&mut a, 160, 40);
        alt(&mut a, 'n');
        assert!(in_float(&a), "{name}: a float with an empty note");
        assert_eq!((a.editor.row, a.editor.col), (0, 2), "{name}: the cursor after `# `");
        a.cli.stop_all();
    }
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-2, FLOAT-6: the float shows the notes folder, `new note`, the branch and `[ esc close ]`;
// the panes behind are dim, and the status bar has no key hints.
#[tokio::test]
async fn the_float_hides_the_panes_and_the_hints() {
    let root = repo("float-hides", &files());
    let mut a = app(&root);
    a.open_file(root.join(NOTE));
    a.branch = Some("master".into());
    key(&mut a, KeyCode::Char('c'));
    let before = draw(&mut a, 120, 40);
    let hint = a.keys.first(zendo::keys::Pane::Chat, "send");
    assert!(before[38].contains(&hint) || before[39].contains(&hint), "the chat shows its hints first:\n{}", before.join("\n"));
    alt(&mut a, 'n');
    let screen = draw(&mut a, 120, 40);
    let top = screen.iter().find(|l| l.contains("new note")).unwrap_or_else(|| panic!("no top line:\n{}", screen.join("\n")));
    assert!(top.contains("docs/notes/new note") && top.contains("master") && top.contains("[ esc close ]"), "{top}");
    assert!(!screen[39].contains(&hint), "no key hints:\n{}", screen.join("\n"));
    let (dim, fg) = (float_colors(&mut a, 120, 40), a.theme().fg);
    assert_ne!(dim.0, fg, "the screen behind is dim");
    assert_eq!(dim.1, fg, "the note in the float is not");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

/// The color of a char of the screen behind (top left) and of the title in the float.
fn float_colors(a: &mut App, w: u16, h: u16) -> (ratatui::style::Color, ratatui::style::Color) {
    typ(a, "Tea");
    let mut term = ratatui::Terminal::new(ratatui::backend::TestBackend::new(w, h)).unwrap();
    term.draw(|f| zendo::ui::draw(f, a)).unwrap();
    let buf = term.backend().buffer().clone();
    let behind = (0..w).map(|x| &buf[(x, 0)]).find(|c| c.symbol() != " ").map(|c| c.fg).unwrap();
    let title = (0..h).flat_map(|y| (0..w - 2).map(move |x| (x, y))).find(|p| buf[*p].symbol() == "T" && buf[(p.0 + 1, p.1)].symbol() == "e" && buf[(p.0 + 2, p.1)].symbol() == "a").unwrap();
    (behind, buf[title].fg)
}

// FLOAT-3, FLOAT-6, FLOAT-7, FLOAT-13: esc saves the note and gives back the file, the pane, the
// mode, the cursor and the selection; the date and time stand under the title; the new note does
// not open in the note pane.
#[tokio::test]
async fn esc_saves_the_float_and_restores_the_screen() {
    let root = repo("float-esc", &files());
    let mut a = app(&root);
    a.open_file(root.join(DESIGN));
    draw(&mut a, 120, 40);
    a.read_cursor = 12;
    a.read_anchor = Some((10, 0));
    let scroll = a.scroll;
    alt(&mut a, 'n');
    typ(&mut a, "Tea");
    let screen = draw(&mut a, 120, 40).join("\n");
    let today = chrono::Local::now().format("%a %-d %b").to_string();
    assert!(screen.contains(&today), "the date under the title:\n{screen}");
    key(&mut a, KeyCode::Enter);
    key(&mut a, KeyCode::Enter);
    typ(&mut a, "A line of text.");
    key(&mut a, KeyCode::Esc);
    assert!(a.float.is_none(), "esc closes the float");
    let saved: Vec<String> = notes(&root).into_iter().filter(|n| n.ends_with("-tea.md")).collect();
    assert_eq!(saved.len(), 1, "{:?}", notes(&root));
    let text = std::fs::read_to_string(root.join("docs/notes").join(&saved[0])).unwrap();
    assert!(text.starts_with("# Tea\n") && text.contains("A line of text."), "{text}");
    assert_eq!(a.current.as_deref(), Some(root.join(DESIGN).as_path()), "the same file");
    assert_eq!((a.mode, a.focus, a.read_cursor, a.read_anchor, a.scroll), (Mode::Read, Focus::Note, 12, Some((10, 0)), scroll));
    assert!(a.notes.iter().any(|n| n.title == "Tea"), "the new note is in the notes list");
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-4: the open file in write mode is saved before the float opens, and is in write mode again
// after it.
#[tokio::test]
async fn the_float_saves_the_open_file_first() {
    let root = repo("float-save-first", &files());
    let mut a = app(&root);
    a.open_file(root.join(NOTE));
    draw(&mut a, 120, 40);
    key(&mut a, KeyCode::Char('i'));
    draw(&mut a, 120, 40);
    typ(&mut a, "Unsaved ");
    alt(&mut a, 'n');
    assert!(std::fs::read_to_string(root.join(NOTE)).unwrap().contains("Unsaved "), "saved before the float");
    key(&mut a, KeyCode::Esc);
    assert_eq!((a.mode, a.editor_path.as_deref()), (Mode::Write, Some(root.join(NOTE).as_path())), "write mode again");
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-5, FLOAT-15: `n` in read mode and in the notes list opens the float; enter in the notes
// list opens a note in the note pane.
#[tokio::test]
async fn n_opens_the_float() {
    let root = repo("float-n", &files());
    let mut a = app(&root);
    a.open_file(root.join(NOTE));
    key(&mut a, KeyCode::Char('n'));
    assert!(in_float(&a), "n in read mode");
    key(&mut a, KeyCode::Esc);
    key(&mut a, KeyCode::Char('o'));
    assert_eq!(a.mode, Mode::Notes);
    key(&mut a, KeyCode::Char('n'));
    assert!(in_float(&a), "n in the notes list");
    key(&mut a, KeyCode::Esc);
    assert_eq!(a.mode, Mode::Notes, "back in the notes list");
    key(&mut a, KeyCode::Enter);
    assert!(a.float.is_none() && a.mode == Mode::Read && a.current.is_some(), "a note that is there opens in the note pane");
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-6: after the first save, the top line shows the path of the note.
#[tokio::test]
async fn the_top_line_shows_the_path_after_the_first_save() {
    let root = repo("float-path", &files());
    let mut a = app(&root);
    alt(&mut a, 'n');
    typ(&mut a, "Green builds");
    a.save();
    let screen = draw(&mut a, 120, 40).join("\n");
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    assert!(screen.contains(&format!("docs/notes/{date}-green-builds.md")), "{screen}");
    assert!(a.float.is_some(), "the float stays open after a save");
    let _ = std::fs::remove_dir_all(&root);
}

/// The columns of the float: the first and the last column of its border on the row of its top line.
fn float_box(screen: &[String]) -> (usize, usize) {
    let row = screen.iter().find(|l| l.contains("[ esc close ]")).unwrap();
    let chars: Vec<char> = row.chars().collect();
    let left = chars.iter().position(|c| *c == '│').unwrap();
    let right = chars.iter().rposition(|c| *c == '│').unwrap();
    (left, right)
}

// FLOAT-8, FLOAT-9: the float is the text column wide with a margin of 4 on each side, over the
// note pane.
#[tokio::test]
async fn the_float_fits_the_text_column() {
    let root = repo("float-width", &files());
    let mut a = app(&root);
    a.open_file(root.join(NOTE));
    alt(&mut a, 'n');
    let screen = draw(&mut a, 160, 40);
    let (l, r) = float_box(&screen);
    assert_eq!(r - l - 1, a.column_width() as usize + 8, "the column and two margins");
    let centre = (l + r) / 2;
    assert!(centre.abs_diff(80) <= 1, "over the note pane, alone on the screen: {l}..{r}");
    let top = screen.iter().position(|row| row.contains("[ esc close ]")).unwrap();
    assert_eq!(top, 2, "one row above the box, then its border");
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-9: where the note pane is narrower than the float, the float stands at the center of the
// screen.
#[tokio::test]
async fn a_narrow_note_pane_centers_the_float() {
    let root = repo("float-narrow", &files());
    let mut a = app(&root);
    a.open_file(root.join(NOTE));
    key(&mut a, KeyCode::Char('c'));
    alt(&mut a, 'b');
    draw(&mut a, 100, 40);
    alt(&mut a, 'n');
    let screen = draw(&mut a, 100, 40);
    let (l, r) = float_box(&screen);
    assert!(((l + r) / 2).abs_diff(50) <= 1, "at the center: {l}..{r}");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-10, FLOAT-11: in the float, the keys of write mode work, and the keys that open a pane,
// move the focus, or open another float do nothing.
#[tokio::test]
async fn pane_keys_do_nothing_in_the_float() {
    let root = repo("float-keys", &files());
    let mut a = app(&root);
    a.open_file(root.join(NOTE));
    alt(&mut a, 'n');
    typ(&mut a, "Tea");
    ctrl(&mut a, 'g');
    alt(&mut a, 'b');
    ctrl(&mut a, 'd');
    a.on_key(KeyEvent::new(KeyCode::F(5), KeyModifiers::NONE));
    alt(&mut a, ']');
    alt(&mut a, 'n');
    assert!(a.float.is_some() && a.mode == Mode::Write, "still in the float");
    assert_eq!((a.panel, a.state.tree, a.delta_open, a.cli.pane), (false, false, false, None), "no pane opened");
    assert_eq!(a.editor.text(), "# Tea", "the same note");
    key(&mut a, KeyCode::Tab);
    assert_eq!(a.focus, Focus::Note, "tab does not move the focus");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// FLOAT-12: a float without text makes no file.
#[tokio::test]
async fn an_empty_float_makes_no_file() {
    let root = repo("float-empty", &files());
    let mut a = app(&root);
    let before = notes(&root);
    alt(&mut a, 'n');
    key(&mut a, KeyCode::Esc);
    assert!(a.float.is_none());
    assert_eq!(notes(&root), before);
    let _ = std::fs::remove_dir_all(&root);
}
