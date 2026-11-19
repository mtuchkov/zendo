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
