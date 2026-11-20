use super::*;

fn t(rows: &[&str]) -> Transcript {
    Transcript { rows: rows.iter().map(|r| r.to_string()).collect(), ..Default::default() }
}

// CLAUDE-29: the cursor and the selection keys of read mode work in the transcript.
#[test]
fn the_cursor_moves_and_selects_in_the_transcript() {
    let mut x = t(&[" › what is it", "", "It is a log of events.", "It keeps them a week."]);
    x.to_end();
    assert_eq!(x.cursor(), (3, 0));
    assert_eq!(x.text(), "It is a log of events.\nIt keeps them a week.", "no selection: the message");
    x.move_rows(-1);
    x.move_word(true);
    x.move_word(true);
    x.anchor = Some((x.row, x.col));
    x.move_word(true);
    x.move_word(true);
    x.move_col(-1);
    x.move_col(-1);
    assert_eq!(x.text(), "a log", "a selection: {:?}", x.selection());
    x.anchor = None;
    x.row = 0;
    x.col = 3;
    assert_eq!(x.text(), "what is it", "the mark of the user is not text");
    x.move_col(-1);
    x.move_col(-1);
    x.move_col(-1);
    x.move_col(-1);
    assert_eq!(x.cursor(), (0, 0), "left stops at the start");
}

// PANES-34: a selection that starts in a cell of a table keeps to its column.
#[test]
fn a_selection_in_a_table_of_the_transcript_keeps_to_the_column() {
    let mut x = t(&["Here:", "┌──────┬──────────┐", "│ Name │ Value    │", "├──────┼──────────┤", "│ a    │ first    │", "│      │ wrapped  │", "└──────┴──────────┘", "Done."]);
    (x.row, x.col) = (4, 9);
    x.anchor = Some((x.row, x.col));
    (x.row, x.col) = (7, 30);
    assert_eq!(x.text(), "first\nwrapped", "the end stays in the table");
    let table = x.table();
    assert_eq!(x.marked(table.as_ref(), 4), Some((9, 18)));
    assert_eq!(x.marked(table.as_ref(), 6), None, "the border is not selected");
    assert_eq!(x.marked(table.as_ref(), 7), None);
}

// PANES-42: the selection keys of the transcript stop at the edge of a cell; two quick presses go on.
#[test]
fn the_keys_of_the_transcript_stop_at_the_edge_of_a_cell() {
    let mut x = t(&["┌──────┬──────────┐", "│ a    │ first    │", "└──────┴──────────┘"]);
    (x.row, x.col) = (1, 2);
    x.anchor = Some((1, 2));
    let t0 = Instant::now();
    for i in 0..8 {
        let before = (x.row, x.col);
        x.move_col(1);
        x.soft_stop("select_right", before, t0 + std::time::Duration::from_millis(20 * i));
    }
    assert_eq!(x.cursor(), (1, 6), "the edge of the cell");
    let before = (x.row, x.col);
    x.move_col(1);
    x.soft_stop("select_right", before, t0 + std::time::Duration::from_millis(300));
    assert_eq!(x.cursor(), (1, 8), "the next cell");
}

// ADR 0008: the transcript handles its own keys, and leaves `copy` and `add_to_chat` to the app.
#[test]
fn the_transcript_takes_its_own_keys_and_leaves_the_rest() {
    let mut x = t(&["one two", "three"]);
    x.focus(true);
    assert!(x.on() && x.cursor() == (1, 0), "the keys come with the cursor at the end");
    let now = Instant::now();
    assert!(x.key("up", 5, now) && x.cursor() == (0, 0));
    assert!(x.key("select_right", 5, now) && x.selection() == Some(((0, 0), (0, 1))));
    assert!(!x.key("copy", 5, now) && !x.key("add_to_chat", 5, now), "the app does these");
    assert!(x.key("back", 5, now) && x.selection().is_none() && x.on(), "back drops the selection first");
    assert!(x.key("back", 5, now) && !x.on(), "then the keys go back to the input");
}

// PANES-55: when the rows of a new frame end above the cursor, the cursor goes to the last row,
// and a selection that started below them ends; the guard of ADR 0026 in `show_rows`.
#[test]
fn the_cursor_stays_in_the_rows_when_they_get_fewer() {
    let mut x = t(&["a"; 17]);
    x.place(16, 9);
    x.anchor = Some((12, 0));
    x.show_rows(["one", "two", "three", "", "five", "six"].map(String::from).to_vec());
    assert_eq!(x.cursor(), (5, 2), "the last row, at most its last char");
    assert_eq!(x.selection(), None, "the anchor is not in the rows");
    x.move_word(false);
    assert_eq!(x.text(), "five\nsix");
}

// PANES-51: rows that stay keep the cursor and the selection where they are.
#[test]
fn the_cursor_stays_put_when_its_row_stays() {
    let mut x = t(&["one", "two", "three"]);
    x.place(1, 2);
    x.anchor = Some((0, 1));
    x.show_rows(vec!["one".into(), "two".into(), "three".into(), "four".into()]);
    assert_eq!((x.cursor(), x.anchor), ((1, 2), Some((0, 1))));
}
