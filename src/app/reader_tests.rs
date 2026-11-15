//! The reader without an app (ADR 0008).
use super::*;

fn text() -> Vec<String> {
    ["one two  three", "", "a long second paragraph", "its end", "", "last"].map(String::from).to_vec()
}

// ADR 0008: up and down keep the column that the cursor wants; a short row does not lose it.
#[test]
fn up_and_down_keep_the_wanted_column() {
    let (t, mut r) = (text(), Reader::default());
    r.place(2, 9);
    r.move_rows(&t, 1);
    assert_eq!(r.cursor(), (3, 6), "the last char of the short row");
    r.move_rows(&t, -1);
    assert_eq!(r.cursor(), (2, 9));
    r.move_rows(&t, 100);
    assert_eq!(r.cursor(), (5, 3), "the last row is the limit");
}

// ADR 0008: left and right cross the ends of the rows and stop at the ends of the text.
#[test]
fn left_and_right_cross_the_rows() {
    let (t, mut r) = (text(), Reader::default());
    r.move_col(&t, false);
    assert_eq!(r.cursor(), (0, 0), "nothing before the first char");
    r.place(2, 22);
    r.move_col(&t, true);
    assert_eq!(r.cursor(), (3, 0));
    r.move_col(&t, false);
    assert_eq!(r.cursor(), (2, 22));
    r.place(5, 3);
    r.move_col(&t, true);
    assert_eq!(r.cursor(), (5, 3), "nothing after the last char");
}

// ADR 0008: a word move goes to the start of a word, and counts its moves by one char.
#[test]
fn a_word_move_stops_at_the_start_of_a_word() {
    let (t, mut r) = (text(), Reader::default());
    assert_eq!(r.move_word(&t, true), 4);
    assert_eq!(r.cursor(), (0, 4), "two");
    r.move_word(&t, true);
    assert_eq!(r.cursor(), (0, 9), "three, over two spaces");
    r.move_word(&t, false);
    assert_eq!(r.cursor(), (0, 4));
    r.place(0, 6);
    r.move_word(&t, false);
    assert_eq!(r.cursor(), (0, 4), "the start of this word");
}

// ADR 0008: the selection has its start before its end, and gives the chars between them.
#[test]
fn the_selection_runs_from_the_anchor_to_the_cursor() {
    let (t, mut r) = (text(), Reader::default());
    r.place(3, 2);
    r.select();
    r.place(2, 7);
    r.select();
    assert_eq!(r.selection(), Some(((2, 7), (3, 2))), "a second start does not move the anchor");
    assert_eq!(r.selection_text(&t), "second paragraph\nits");
    assert_eq!(r.selection_rows(&t), (2, 3));
    r.toggle_select();
    assert_eq!(r.selection(), None);
}

// ADR 0008: without a selection, the text is the paragraph under the cursor.
#[test]
fn without_a_selection_the_text_is_the_paragraph() {
    let (t, mut r) = (text(), Reader::default());
    r.place(3, 0);
    assert_eq!(r.selection_rows(&t), (2, 3));
    assert_eq!(r.selection_text(&t), "a long second paragraph\nits end");
}

// ADR 0008: the draw keeps the cursor in the text and the cursor in the view.
#[test]
fn the_view_follows_the_cursor() {
    let t: Vec<String> = (0..100).map(|i| format!("row {i}")).collect();
    let mut r = Reader::default();
    r.place(500, 40);
    r.fit(&t, 20);
    assert_eq!(r.cursor(), (99, 5), "the last row and its last char");
    assert_eq!((r.scroll(), r.cursor_y()), (80, 19), "the view ends with the text");
    r.to_row(50);
    r.fit(&t, 20);
    assert_eq!((r.scroll(), r.cursor_y()), (47, 3), "a margin of three rows");
    r.wheel(false, 10);
    assert_eq!((r.row(), r.scroll()), (60, 57));
}

// ADR 0008: after a mode change the cursor keeps its row on the screen, once.
#[test]
fn a_mode_change_keeps_the_screen_row_of_the_cursor() {
    let mut r = Reader::default();
    r.follow(30, 20, 100);
    let y = r.cursor_y();
    r.keep_screen_row();
    r.follow(60, 20, 100);
    assert_eq!((r.cursor_y(), r.scroll()), (y, 60 - y));
    r.follow(70, 20, 100);
    assert_eq!(r.cursor_y(), 16, "the next draw follows with the margin again");
}

// ADR 0008, FLOAT-1: the float and the screen behind it swap their views.
#[test]
fn two_views_swap() {
    let (mut a, mut b) = (Reader::default(), Reader::default());
    a.place(12, 3);
    a.select_from((10, 0));
    a.swap_view(&mut b);
    assert_eq!((a.cursor(), a.anchor()), ((0, 0), None));
    assert_eq!((b.cursor(), b.anchor()), ((12, 3), Some((10, 0))));
    a.to_start();
    a.jump(40);
    assert_eq!((a.row(), a.scroll()), (40, 37));
}
