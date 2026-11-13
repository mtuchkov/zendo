use super::*;
use ratatui::text::Line;
use ratatui::crossterm::event::KeyModifiers;
use ratatui::widgets::{Paragraph, Widget};

#[test]
fn dropped_files_are_paths_and_nothing_else() {
    let dir = std::env::temp_dir().join(format!("zendo-drop-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (a, b) = (dir.join("shot 1.png"), dir.join("notes.md"));
    std::fs::write(&a, b"png").unwrap();
    std::fs::write(&b, b"# n").unwrap();
    let (sa, sb) = (a.display().to_string(), b.display().to_string());
    assert_eq!(dropped_files(&sa), Some(vec![a.clone()]), "one path with a space");
    assert_eq!(dropped_files(&format!("{} {sb}", sa.replace(' ', "\\ "))), Some(vec![a.clone(), b.clone()]), "escaped space, two files");
    assert_eq!(dropped_files(&format!("'{sa}' '{sb}' ")), Some(vec![a.clone(), b.clone()]), "quoted");
    assert_eq!(dropped_files(&format!("file://{}", sa.replace(' ', "%20"))), Some(vec![a.clone()]));
    assert_eq!(dropped_files("look at notes.md please"), None, "text stays text");
    assert_eq!(dropped_files(&format!("{sb} and more")), None, "a path and words is text");
    assert_eq!(dropped_files("/no/such/file.png"), None);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn base64_matches_known_values() {
    assert_eq!(base64(b"Man"), "TWFu");
    assert_eq!(base64(b"Ma"), "TWE=");
    assert_eq!(base64(b"M"), "TQ==");
    assert_eq!(base64("кэш".as_bytes()), "0LrRjdGI");
}

#[test]
fn selection_stays_inside_its_panel() {
    let area = Rect::new(0, 0, 40, 3);
    let mut buf = Buffer::empty(area);
    let lines = vec![Line::raw("left text one       right side A"), Line::raw("left text two       right side B"), Line::raw("left three          right side C")];
    Paragraph::new(lines).render(area, &mut buf);
    // The left panel is columns 0..20.
    let left = Rect::new(0, 0, 20, 3);
    let mut s = Selection::start(Region::Note, left, 5, 0);
    s.drag(35, 1); // dragged into the right panel: clamped to the left one
    assert_eq!(s.text(&buf), "text one\nleft text two");
}

const TABLE: [&str; 6] = [
    "┌──────┬──────────┐",
    "│ Name │ Value    │",
    "├──────┼──────────┤",
    "│ a    │ first    │",
    "│      │ wrapped  │",
    "└──────┴──────────┘",
];

fn table_buffer() -> (Rect, Buffer) {
    let area = Rect::new(0, 0, 30, 7);
    let mut buf = Buffer::empty(area);
    let lines: Vec<Line> = std::iter::once("text before").chain(TABLE).map(Line::raw).collect();
    Paragraph::new(lines).render(area, &mut buf);
    (area, buf)
}

#[test]
fn a_mouse_selection_in_a_table_keeps_to_the_column() {
    // PANES-34: from "first" down and to the left; the column of the names stays out.
    let (area, mut buf) = table_buffer();
    let mut s = Selection::start(Region::Note, area, 9, 4);
    s.drag(16, 5);
    assert_eq!(s.text(&buf), "first\nwrapped");
    s.paint(&mut buf, Color::Red, Color::Black);
    assert_eq!(buf[(9, 4)].bg, Color::Red);
    assert_ne!(buf[(2, 5)].bg, Color::Red, "the other column is not selected");
    assert_ne!(buf[(19, 5)].bg, Color::Red, "past the table is not selected");
}

#[test]
fn a_mouse_selection_that_starts_before_a_table_goes_line_by_line() {
    // PANES-34
    let (area, buf) = table_buffer();
    let mut s = Selection::start(Region::Note, area, 0, 0);
    s.drag(5, 2);
    assert_eq!(s.text(&buf), "text before\n┌──────┬──────────┐\n│ Name");
}

#[test]
fn ctrl_or_shift_selects_whole_lines_in_a_table() {
    // PANES-43: the modifier breaks the glass of the cell.
    let (area, buf) = table_buffer();
    let mut s = Selection::start(Region::Note, area, 9, 4).with(KeyModifiers::NONE);
    s.drag(12, 5);
    s.hold(KeyModifiers::CONTROL);
    assert!(s.free);
    assert_eq!(s.text(&buf), "first    │\n│      │ wrap");
    let s = Selection::start(Region::Note, area, 9, 4).with(KeyModifiers::SHIFT);
    assert!(s.free && !s.chat);
}

#[test]
fn alt_marks_the_selection_for_the_chat() {
    // PANES-44
    let s = Selection::start(Region::Note, Rect::new(0, 0, 10, 2), 1, 1).with(KeyModifiers::ALT);
    assert!(s.chat && !s.free);
}
