use super::*;

const TABLE: [&str; 9] = [
    "intro text",
    "┌──────┬──────────┐",
    "│ Name │ Value    │",
    "├──────┼──────────┤",
    "│ a    │ first    │",
    "│      │ wrapped  │",
    "│ b    │ second   │",
    "└──────┴──────────┘",
    "after",
];

#[test]
fn a_position_in_a_cell_is_in_the_table() {
    // PANES-34
    assert_eq!(table_at(&TABLE, 4, 2), Some(Table { top: 2, bottom: 6, lines: vec![0, 7, 18] }));
    assert_eq!(table_at(&TABLE, 2, 12).map(|t| (t.top, t.bottom)), Some((2, 6)));
}

#[test]
fn text_borders_and_lines_are_not_in_a_cell() {
    // PANES-34
    assert_eq!(table_at(&TABLE, 0, 2), None, "text before the table");
    assert_eq!(table_at(&TABLE, 1, 3), None, "the top border");
    assert_eq!(table_at(&TABLE, 3, 3), None, "a rule inside the table");
    assert_eq!(table_at(&TABLE, 4, 7), None, "a vertical line");
    assert_eq!(table_at(&TABLE, 8, 1), None, "text after the table");
}

#[test]
fn ascii_tables_are_plain_text() {
    // PANES-36
    let rows = ["+---+---+", "| a | b |", "+---+---+", "| c | d |", "+---+---+"];
    assert_eq!(table_at(&rows, 1, 2), None);
    let md = ["| a | b |", "|---|---|", "| c | d |"];
    assert_eq!(table_at(&md, 2, 2), None);
}

#[test]
fn the_borders_are_found_only_within_the_reach() {
    // PANES-35
    let mut rows = vec!["┌───┐".to_string()];
    rows.extend((0..2 * REACH).map(|_| "│ x │".to_string()));
    assert!(table_at(&rows, 10, 2).is_some(), "the top border is near");
    assert_eq!(table_at(&rows, REACH + 50, 2), None, "no border within the reach");
}

#[test]
fn a_table_cut_by_the_edge_of_the_rows_is_still_a_table() {
    // PANES-35: a pane shows only a part of a long table.
    let rows = ["│ a │ b │", "│ c │ d │", "└───┴───┘"];
    assert_eq!(table_at(&rows, 0, 2).map(|t| (t.top, t.bottom)), Some((0, 1)));
    let open = ["│ a │ b │", "│ c │ d │"];
    assert_eq!(table_at(&open, 0, 2), None, "no border at all");
}

#[test]
fn a_selection_in_a_cell_keeps_to_its_column() {
    // PANES-34
    let r = resolve(&TABLE, (4, 9), (6, 12));
    assert_eq!(r.cols, Some((8, 18)));
    assert_eq!(r.text(&TABLE), "first\nwrapped\nseco");
}

#[test]
fn a_drag_into_the_next_column_takes_both_columns() {
    // PANES-34, PANES-40: a tab between the cells, no lines
    let r = resolve(&TABLE, (4, 2), (6, 12));
    assert_eq!(r.cols, Some((1, 18)));
    assert_eq!(r.text(&TABLE), "a\tfirst\n\twrapped\nb\tseco");
}

#[test]
fn a_drag_out_of_the_table_stays_in_it() {
    // PANES-41
    // Below or above the table: the column of the anchor, to its end.
    let r = resolve(&TABLE, (4, 9), (8, 0));
    assert_eq!((r.start, r.end, r.cols), ((4, 9), (6, 17), Some((8, 18))));
    assert_eq!(r.text(&TABLE), "first\nwrapped\nsecond");
    let up = resolve(&TABLE, (4, 9), (0, 30));
    assert_eq!((up.start, up.end, up.cols), ((2, 8), (4, 9), Some((8, 18))));
    // Left or right of the table in its rows: to its first or last column.
    let left = resolve(&TABLE, (4, 9), (5, 0));
    assert_eq!((left.end, left.cols), ((5, 1), Some((1, 18))));
}

#[test]
fn rules_inside_the_table_are_not_selected() {
    // PANES-40
    let r = resolve(&TABLE, (2, 2), (4, 4));
    assert_eq!(r.span(3, 19), None);
    assert_eq!(r.span(2, 19), Some((2, 7)));
    assert_eq!(r.text(&TABLE), "Name\na");
}

#[test]
fn a_selection_outside_a_table_is_plain() {
    // PANES-34: it goes line by line, also through a table.
    let r = resolve(&TABLE, (0, 2), (4, 4));
    assert_eq!(r.cols, None);
    assert_eq!(r.span(2, 19), Some((0, 19)));
    assert_eq!(r.span(4, 19), Some((0, 5)));
}

#[test]
fn wide_chars_take_two_columns() {
    // PANES-34
    let rows = ["┌──────┬───┐", "│ 日本 │ x │", "│ abcd │ y │", "└──────┴───┘"];
    assert_eq!(char_at(rows[1], 3), Some('日'));
    assert_eq!(col_of(rows[1], 7), 9);
    assert_eq!(index_at(rows[1], 9), 7);
    let r = in_table(&rows, (1, 2), (2, 4)).expect("in the table");
    assert_eq!(r.text(&rows), "日本\nabc");
    assert_eq!(r.chars(&rows, 1), Some((2, 5)));
    assert!(in_table(&rows, (0, 1), (2, 4)).is_none());
}

#[test]
fn heavy_double_rounded_and_mixed_tables_are_tables() {
    // PANES-34: every char of the Box Drawing block, by its arms.
    let tables: [[&str; 6]; 4] = [
        ["┏━━━┳━━━┓", "┃ a ┃ b ┃", "┣━━━╋━━━┫", "┃ c ┃ d ┃", "┃ e ┃ f ┃", "┗━━━┻━━━┛"],
        ["╔═══╦═══╗", "║ a ║ b ║", "╠═══╬═══╣", "║ c ║ d ║", "║ e ║ f ║", "╚═══╩═══╝"],
        ["╭───┬───╮", "│ a │ b │", "├───┼───┤", "│ c │ d │", "│ e │ f │", "╰───┴───╯"],
        ["┍━━━┯━━━┑", "│ a │ b │", "┝━━━┿━━━┥", "╎ c ╎ d ╎", "┆ e ┆ f ┆", "┕━━━┷━━━┙"],
    ];
    for rows in tables {
        assert_eq!(table_at(&rows, 3, 2), Some(Table { top: 1, bottom: 4, lines: vec![0, 4, 8] }), "{rows:?}");
        assert_eq!(resolve(&rows, (1, 2), (4, 2)).text(&rows), "a\nc\ne", "{rows:?}");
    }
}

#[test]
fn each_char_of_the_block_but_the_diagonals_has_one_class() {
    // PANES-34
    for c in ('\u{2500}'..='\u{257F}').filter(|c| !"╱╲╳".contains(*c)) {
        let n = [VERTICAL, HORIZONTAL, TOP, BOTTOM, MIDDLE].iter().filter(|class| class.contains(c)).count();
        assert_eq!(n, 1, "{c}");
    }
}
