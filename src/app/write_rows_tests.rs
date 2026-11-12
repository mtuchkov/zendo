use super::*;

fn lines(s: &str) -> Vec<String> {
    s.lines().map(String::from).collect()
}

// NOTE-32: a row holds the end of one source line and the start of the next.
#[test]
fn a_row_spans_two_source_lines() {
    let src = lines("aaa bbb\nccc **ddd**\neee\n\n#tag");
    let starts = [(0, (0, 0), 0), (1, (1, 4), 0), (2, (3, 0), 0)];
    let rows = [0, 0, 0, 2, 3];
    let s = spans(&src, &starts, &rows);
    assert_eq!(s[0].text(), "aaa bbb ccc");
    assert_eq!(s[1].text(), "**ddd** eee", "not into the blank line");
    assert_eq!(s[1].end, (2, 3));
    assert_eq!(s[2].text(), "", "the blank line, not the hidden tag line after it");
    assert_eq!(cursor_span(&s, (1, 2)), Some(0));
    assert_eq!(cursor_span(&s, (1, 4)), Some(1));
    assert_eq!(cursor_span(&s, (2, 3)), Some(1), "the end of a line");
    assert_eq!(s[1].x_of((2, 1)), 9);
    assert_eq!(s[0].at_x(5), (0, 5));
    assert_eq!(s[0].at_x(40), (1, 3), "past the end: the space before the next row");
    assert_eq!(s[1].at_x(40), (2, 3));
}
