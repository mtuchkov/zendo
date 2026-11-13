use super::*;

fn t(rows: &[&str]) -> Transcript {
    Transcript { rows: rows.iter().map(|r| r.to_string()).collect(), ..Default::default() }
}

// CLAUDE-29: the cursor and the selection keys of read mode work in the transcript.
#[test]
fn the_cursor_moves_and_selects_in_the_transcript() {
    let mut x = t(&[" › what is it", "", "It is a log of events.", "It keeps them a week."]);
    x.to_end();
    assert_eq!((x.row, x.col), (3, 0));
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
    assert_eq!((x.row, x.col), (0, 0), "left stops at the start");
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
    assert_eq!((x.row, x.col), (1, 6), "the edge of the cell");
    let before = (x.row, x.col);
    x.move_col(1);
    x.soft_stop("select_right", before, t0 + std::time::Duration::from_millis(300));
    assert_eq!((x.row, x.col), (1, 8), "the next cell");
}
