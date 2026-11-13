use super::*;
use std::time::{Duration, Instant};

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

fn ms(t0: Instant, n: u64) -> Instant {
    t0 + Duration::from_millis(n)
}

#[test]
fn the_cursor_stops_at_the_edge_of_the_cell() {
    // PANES-42
    let (mut s, t0) = (SoftStop::default(), Instant::now());
    assert_eq!(s.step(&TABLE, (4, 2), (4, 5), (4, 6), Dir::Right, t0), (4, 6), "a move in the cell");
    assert_eq!(s.step(&TABLE, (4, 2), (4, 6), (4, 7), Dir::Right, ms(t0, 300)), (4, 6), "the edge: a stop");
}

#[test]
fn two_quick_presses_go_on_to_the_next_cell() {
    // PANES-42: the stop and one more quick press in the same direction.
    let (mut s, t0) = (SoftStop::default(), Instant::now());
    assert_eq!(s.step(&TABLE, (4, 2), (4, 6), (4, 7), Dir::Right, t0), (4, 6));
    assert_eq!(s.step(&TABLE, (4, 2), (4, 6), (4, 7), Dir::Right, ms(t0, 150)), (4, 8));
    let mut s = SoftStop::default();
    assert_eq!(s.step(&TABLE, (4, 9), (4, 8), (4, 7), Dir::Left, t0), (4, 8));
    assert_eq!(s.step(&TABLE, (4, 9), (4, 8), (4, 7), Dir::Left, ms(t0, 150)), (4, 6), "to the left too");
}

#[test]
fn a_slow_press_or_a_held_key_stops_again() {
    // PANES-42
    let (mut s, t0) = (SoftStop::default(), Instant::now());
    s.step(&TABLE, (4, 2), (4, 6), (4, 7), Dir::Right, t0);
    assert_eq!(s.step(&TABLE, (4, 2), (4, 6), (4, 7), Dir::Right, ms(t0, 900)), (4, 6), "too slow");
    assert_eq!(s.step(&TABLE, (4, 2), (4, 6), (4, 7), Dir::Right, ms(t0, 1050)), (4, 8), "quick after the slow one");
    let mut s = SoftStop::default();
    for i in 0..20 {
        assert_eq!(s.step(&TABLE, (4, 2), (4, 6), (4, 7), Dir::Right, ms(t0, 30 * i)), (4, 6), "a held key repeats fast");
    }
}

#[test]
fn the_direction_of_the_press_counts() {
    // PANES-42: a stop to the right and a quick press down is a move down in the cell.
    let (mut s, t0) = (SoftStop::default(), Instant::now());
    s.step(&TABLE, (4, 2), (4, 6), (4, 7), Dir::Right, t0);
    assert_eq!(s.step(&TABLE, (4, 2), (4, 6), (5, 6), Dir::Down, ms(t0, 150)), (5, 6));
    assert_eq!(s.step(&TABLE, (4, 2), (5, 6), (5, 7), Dir::Right, ms(t0, 300)), (5, 6), "a new stop");
}

#[test]
fn rules_and_borders_are_edges_up_and_down() {
    // PANES-42
    let (mut s, t0) = (SoftStop::default(), Instant::now());
    assert_eq!(s.step(&TABLE, (2, 2), (2, 2), (3, 2), Dir::Down, t0), (2, 2), "the rule under the head");
    assert_eq!(s.step(&TABLE, (2, 2), (2, 2), (3, 2), Dir::Down, ms(t0, 150)), (4, 2), "over the rule");
    assert_eq!(s.step(&TABLE, (2, 2), (6, 2), (7, 2), Dir::Down, ms(t0, 300)), (6, 2), "the bottom border");
    assert_eq!(s.step(&TABLE, (2, 2), (6, 2), (7, 2), Dir::Down, ms(t0, 450)), (6, 2), "no cell under the table");
    let mut s = SoftStop::default();
    assert_eq!(s.step(&TABLE, (5, 2), (4, 2), (3, 2), Dir::Up, t0), (4, 2));
    assert_eq!(s.step(&TABLE, (5, 2), (4, 2), (3, 2), Dir::Up, ms(t0, 150)), (2, 2), "up over the rule");
}

#[test]
fn a_long_move_stops_at_the_edge() {
    // PANES-42: a word move that leaves the cell goes to its edge first.
    let mut s = SoftStop::default();
    assert_eq!(s.step(&TABLE, (4, 9), (4, 9), (5, 0), Dir::Right, Instant::now()), (4, 17));
}

#[test]
fn outside_a_table_the_move_is_free() {
    // PANES-36
    let mut s = SoftStop::default();
    assert_eq!(s.step(&TABLE, (0, 2), (0, 2), (4, 7), Dir::Down, Instant::now()), (4, 7));
}
