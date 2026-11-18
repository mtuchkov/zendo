use super::*;
use std::time::{Duration, Instant};

// CLAUDE-48: two clicks on one chip within DOUBLE_CLICK are a double-click; a click on another
// chip, or a late one, starts again.
#[test]
fn two_clicks_on_one_chip_are_a_double_click() {
    let mut c = Composer::default();
    let t0 = Instant::now();
    assert!(!c.double_click("⟦a⟧", t0));
    assert!(c.double_click("⟦a⟧", t0 + Duration::from_millis(200)));
    assert!(!c.double_click("⟦a⟧", t0 + Duration::from_millis(300)), "a third click starts again");
    assert!(!c.double_click("⟦b⟧", t0 + Duration::from_millis(400)), "another chip");
    assert!(!c.double_click("⟦b⟧", t0 + Duration::from_secs(2)), "too late");
}

// CLAUDE-48: the text of a text chip: the quoted lines without `> `, the other lines dim.
#[test]
fn a_context_shows_its_quote_plain_and_its_source_dim() {
    let lines = context_lines("Text from this chat:\n> one\n> two");
    assert_eq!(lines, [("Text from this chat:".to_string(), true), ("one".into(), false), ("two".into(), false)]);
}
