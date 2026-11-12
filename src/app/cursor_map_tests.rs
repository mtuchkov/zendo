use super::*;

// NOTE-28: a joined paragraph and a list item map char by char, both ways.
#[test]
fn the_same_char_in_both_views() {
    let src = ["# T", "", "One **bold** line", "goes on.", "", "- an item"];
    // Read mode: the title row, a blank, the joined paragraph wrapped in two rows, a blank, the item.
    let shown: Vec<String> = ["T", "", "One bold line goes", "on.", "", "• an item"].iter().map(|s| s.to_string()).collect();
    let rows = vec![0, 1, 2, 2, 4, 5];
    // "goes" is on the second source line and on the first shown row.
    assert_eq!(shown_to_source(&shown, &rows, &src, (2, 14)), (3, 0));
    assert_eq!(source_to_shown(&shown, &rows, &src, (3, 0)), (2, 14));
    // "on." is on the second shown row.
    assert_eq!(shown_to_source(&shown, &rows, &src, (3, 0)), (3, 5));
    assert_eq!(source_to_shown(&shown, &rows, &src, (2, 6)), (2, 4), "inside **bold**");
    assert_eq!(shown_to_source(&shown, &rows, &src, (5, 2)), (5, 2), "the bullet: \"an\"");
}

// NOTE-28: the title and a blank line that read mode does not show share the row of the paragraph.
#[test]
fn hidden_lines_do_not_shift_the_cursor() {
    let src = ["# T", "", "Security wants every", "audit event."];
    let shown: Vec<String> = ["Security wants every audit", "event."].iter().map(|s| s.to_string()).collect();
    let rows = vec![0, 0, 0, 0];
    assert_eq!(shown_to_source(&shown, &rows, &src, (0, 4)), (2, 4));
    assert_eq!(source_to_shown(&shown, &rows, &src, (3, 0)), (0, 21));
    assert_eq!(shown_to_source(&shown, &rows, &src, (1, 2)), (3, 8), "the second e of \"event\"");
}

#[test]
fn a_link_url_does_not_lose_the_way() {
    let src = ["See [the spec](https://example.com/a/very/long/path/to/the/spec) for more words here."];
    let shown: Vec<String> = ["See the spec for more words here."].iter().map(|s| s.to_string()).collect();
    let rows = vec![0];
    let (_, c) = shown_to_source(&shown, &rows, &src, (0, 22));
    assert_eq!(&src[0][c..c + 5], "words");
}

// NOTE-28: a wrapped row that ends in its break space.
#[test]
fn a_row_with_a_trailing_space() {
    let src = ["# T", "", "Security wants every audit event kept long enough for an incident review,", "and finance wants the storage bill flat. The ingest service writes about", "40 GB of events a day, and nothing deletes them today."];
    let shown: Vec<String> = ["Security wants every audit event kept long enough for an incident review, ", "and finance wants the storage bill flat. The ingest service writes about 40 ", "GB of events a day, and nothing deletes them today."].iter().map(|s| s.to_string()).collect();
    let rows = vec![0, 0, 0, 0, 0];
    assert_eq!(shown_to_source(&shown, &rows, &src, (2, 7)), (4, 10));
    assert_eq!(source_to_shown(&shown, &rows, &src, (4, 10)), (2, 7));
}
