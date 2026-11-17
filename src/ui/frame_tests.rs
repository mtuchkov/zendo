//! The frame without an app (ADR 0008).
use super::*;

fn span(row: usize) -> RowSpan {
    RowSpan { row, ..RowSpan::default() }
}

// ADR 0008: a frame builds without an `App`; before the first draw it has only the sizes.
#[test]
fn a_frame_before_the_first_draw_has_the_sizes_and_nothing_on_the_screen() {
    let fr = Frame::sized(20, 70, 1000);
    assert_eq!((fr.view_height(), fr.text_width(), fr.word_limit()), (20, 70, 1000));
    assert_eq!((fr.word_count(), fr.over_limit()), (0, None));
    assert!(fr.visible().is_empty() && fr.pane_rects().is_empty() && fr.regions().is_empty());
    assert!(fr.read_text().is_empty() && fr.read_rows().is_empty() && fr.write_rows().is_empty());
    assert!(fr.tree_rows().is_empty() && fr.session_buttons().is_empty() && fr.play_buttons().is_empty());
}

// ADR 0008, FLOAT-1: the float and the screen behind it keep their own rows of write mode.
#[test]
fn the_rows_of_write_mode_swap_with_the_rows_of_the_other_screen() {
    let mut fr = Frame { write_rows: vec![span(1)], ..Frame::default() };
    let mut behind = vec![span(7), span(8)];
    fr.swap_write_rows(&mut behind);
    assert_eq!(fr.write_rows(), [span(7), span(8)]);
    assert_eq!(behind, [span(1)]);
}
