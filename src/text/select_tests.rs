use super::*;
use ratatui::text::Line;
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
