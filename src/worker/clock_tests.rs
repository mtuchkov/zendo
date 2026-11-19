use super::*;
use std::time::Duration;

// IO-8: a tick longer than 100 ms gives a line with the phase that took the time.
#[test]
fn a_slow_tick_is_logged_with_its_phase() {
    let mut c = Clock::start("tick");
    c.time("board_tick", || std::thread::sleep(Duration::from_millis(120)));
    c.time("draw", || ());
    let line = c.report().expect("a slow tick is reported");
    assert!(line.contains("tick"), "{line}");
    assert!(line.contains("slowest board_tick"), "the phase: {line}");
    assert!(line.ends_with('\n'), "one line: {line:?}");
}

// IO-8: a tick within the limit gives no line.
#[test]
fn a_fast_tick_is_not_logged() {
    let mut c = Clock::start("key");
    let n = c.time("on_key", || 2 + 2);
    assert_eq!(n, 4, "the phase gives its value back");
    assert_eq!(c.report(), None);
}

// IO-6: a wait under 350 ms shows nothing; after it, "reading" with dots that move.
#[test]
fn reading_shows_only_after_350_ms_with_moving_dots() {
    assert_eq!(reading(Duration::from_millis(300)), None, "a short wait shows nothing");
    let frames: Vec<String> = [400, 650, 900, 1150].iter().filter_map(|ms| reading(Duration::from_millis(*ms))).collect();
    assert_eq!(frames, ["reading.  ", "reading.. ", "reading...", "reading.  "]);
}
