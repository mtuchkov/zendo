//! The threads without an app (ADR 0008).
use super::*;
use std::time::Instant;

// ADR 0008: a thread comes with its first use, and keeps its items under a new key.
#[test]
fn a_thread_comes_with_its_first_use_and_moves_to_a_new_key() {
    let mut t = Threads::default();
    assert!(t.get("a").is_none() && !t.busy("a"));
    t.entry("a").busy = Some(Instant::now());
    assert!(t.busy("a") && t["a"].busy.is_some());
    t.rename("a", "b".into());
    assert!(t.get("a").is_none() && t.busy("b"));
    t.rename("none", "c".into());
    assert_eq!(t.keys().collect::<Vec<_>>(), ["b"]);
    t.values_mut().for_each(|th| th.busy = None);
    assert!(t.values().all(|th| th.busy.is_none()));
    assert!(t.remove("b").is_some() && t.get_mut("b").is_none());
}

// ADR 0008: the view shows the end of the thread, scrolls away from it, and stays in the thread.
#[test]
fn the_view_scrolls_from_the_end_of_the_thread() {
    let mut t = Threads::default();
    assert_eq!(t.fit(100, 20, false), 80, "the end of the thread");
    t.scroll_by(true, 30);
    assert_eq!(t.fit(100, 20, false), 50);
    t.scroll_by(true, 1000);
    assert_eq!(t.fit(100, 20, false), 0, "the start of the thread is the limit");
    t.scroll_by(false, 10);
    assert_eq!(t.fit(100, 20, false), 10);
    t.to_end();
    assert_eq!(t.fit(100, 20, false), 80);
    assert_eq!(t.fit(5, 20, false), 0, "a short thread does not scroll");
}

// ADR 0008, CLAUDE-36: while the transcript has the keys, the view follows its cursor.
#[test]
fn the_view_follows_the_cursor_of_the_transcript() {
    let mut t = Threads::default();
    t.transcript_mut().place(10, 0);
    assert_eq!(t.fit(100, 20, true), 10, "the cursor row is the first row");
    t.transcript_mut().place(95, 0);
    assert_eq!(t.fit(100, 20, true), 76, "the cursor row is the last row");
    assert_eq!(t.fit(100, 20, false), 76, "the view stays when the input takes the keys");
}

// PANES-46: a click in the transcript puts its cursor under the pointer, and the view stays.
#[test]
fn a_click_puts_the_cursor_of_the_transcript_under_the_pointer() {
    let mut t = Threads::default();
    t.transcript_mut().show_rows((0..100).map(|i| format!("row {i}")).collect());
    t.scroll_by(true, 30);
    assert_eq!(t.fit(100, 20, false), 50);
    t.click(3, 20, 2);
    assert!(t.transcript().on(), "the transcript has the keys");
    assert_eq!(t.transcript().cursor(), (53, 2));
    assert_eq!(t.fit(100, 20, true), 50, "the view stays");
    t.click(19, 20, 50);
    assert_eq!(t.transcript().cursor(), (69, 5), "the cursor stays in the row");
    t.transcript_mut().show_rows(vec!["one".into(), "two".into()]);
    t.click(10, 20, 0);
    assert_eq!(t.transcript().cursor(), (1, 0), "a click below a short thread goes to its last row");
}

// ADR 0008, FLOAT-1: a thread stays pinned until the unpin; the float swaps it with its own.
#[test]
fn the_pinned_thread_swaps_with_the_one_of_the_float() {
    let mut t = Threads::default();
    t.pin("(all notes)");
    let mut other = None;
    t.swap_pinned(&mut other);
    assert_eq!((t.pinned(), other.as_deref()), (None, Some("(all notes)")));
    t.swap_pinned(&mut other);
    t.unpin();
    assert_eq!(t.pinned(), None);
    assert!(!t.details() && !t.shown());
    t.show();
    assert!(t.shown());
    t.hide();
    assert!(!t.shown());
}
