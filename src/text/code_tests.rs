use super::*;

fn text(r: &Rendered) -> Vec<String> {
    r.lines.iter().map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect()).collect()
}

// FILES-22: a code file wraps at the column; each row knows its line and char.
#[test]
fn a_long_line_wraps_and_each_row_knows_where_it_starts() {
    let t = crate::ui::theme::THEMES[0];
    let lines = vec!["fn main() {".to_string(), "    let words = one two three four five six;".to_string(), "}".to_string()];
    let r = render(&lines, "rs", 20, &t);
    assert_eq!(text(&r).concat(), lines.concat(), "no char lost");
    assert_eq!(r.rows, vec![0, 1, 4], "the first row of each line");
    assert_eq!(r.starts[2], Some((1, 20)), "the second row of line 1");
    assert!(text(&r).iter().all(|row| row.chars().count() <= 20));
    let keyword = r.lines[0].spans.iter().find(|s| s.content == "fn").expect("fn in its own span");
    assert_eq!(keyword.style.fg, Some(t.acc), "in the colors of the language");
}

#[test]
fn a_tab_shows_as_spaces_and_the_gutter_fits_the_numbers() {
    let t = crate::ui::theme::THEMES[0];
    let r = render(&["\tx".to_string()], "Makefile", 40, &t);
    assert_eq!(text(&r), vec!["    x"]);
    assert_eq!(gutter(9), 2);
    assert_eq!(gutter(120), 4);
    assert!(is_code(Path::new("Cargo.toml")) && !is_code(Path::new("a.md")));
    assert_eq!(lang_of(Path::new("Makefile")), "Makefile");
}
