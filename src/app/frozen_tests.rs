//! The frozen rows of the transcript (PANES-51, PANES-54 to PANES-57, ADR 0026).
use super::*;
use crate::app::Threads;
use ratatui::text::Line;

fn rows(texts: &[&str]) -> Rows {
    Rows { lines: texts.iter().map(|t| Line::raw(t.to_string())).collect(), ..Default::default() }
}

fn numbered(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("row {i}")).collect()
}

fn texts(r: &Rows) -> Vec<String> {
    r.lines.iter().map(|l| l.to_string()).collect()
}

/// The transcript has the keys, with the cursor on its last row.
fn on(t: &mut Threads) {
    t.transcript_mut().focus(true);
}

// PANES-51: while the transcript has the keys, its rows stay, and new output waits below them.
#[test]
fn the_rows_stay_while_the_transcript_has_the_keys() {
    let mut t = Threads::default();
    let ten = numbered(10);
    let ten: Vec<&str> = ten.iter().map(String::as_str).collect();
    let (shown, new) = t.view_rows("a", 80, false, rows(&ten));
    assert_eq!((texts(&shown).len(), new), (10, 0), "live rows without the keys");
    on(&mut t);
    t.view_rows("a", 80, true, rows(&ten));
    let mut more = ten.clone();
    more.extend(["row 10", "row 11", "row 12"]);
    let (shown, new) = t.view_rows("a", 80, true, rows(&more));
    assert_eq!((texts(&shown), new), (texts(&rows(&ten)), 3), "the frozen rows, and the count of new ones");
    assert_eq!(t.transcript().rows().len(), 10, "the cursor works in the frozen rows");
    let (shown, new) = t.view_rows("a", 80, false, rows(&more));
    assert_eq!((texts(&shown).len(), new), (13, 0), "without the keys the rows are live again");
}

// PANES-57: a message that grows after the freeze keeps its frozen rows; the rest is new rows.
#[test]
fn a_message_that_grows_keeps_its_frozen_rows() {
    let mut t = Threads::default();
    on(&mut t);
    t.view_rows("a", 80, true, rows(&["question", "", "the answer starts"]));
    let (shown, new) = t.view_rows("a", 80, true, rows(&["question", "", "the answer starts here and", "goes on"]));
    assert_eq!((texts(&shown), new), (texts(&rows(&["question", "", "the answer starts"])), 1));
}

// PANES-54: `bottom` takes the new rows and moves the cursor to the last row with text; the
// selection keeps its start.
#[test]
fn bottom_takes_the_new_rows_and_keeps_the_selection() {
    let mut t = Threads::default();
    on(&mut t);
    t.view_rows("a", 80, true, rows(&["one", "two", "three"]));
    t.transcript_mut().place(1, 0);
    t.transcript_mut().key("select", 1, std::time::Instant::now());
    let (shown, new) = t.view_rows("a", 80, true, rows(&["one", "two", "three", "four", "five", ""]));
    assert_eq!((shown.lines.len(), new), (3, 3));
    t.refreeze(Refreeze::End);
    let (shown, new) = t.view_rows("a", 80, true, rows(&["one", "two", "three", "four", "five", ""]));
    assert_eq!((shown.lines.len(), new), (6, 0), "the new rows are in the transcript");
    assert_eq!(t.transcript().cursor(), (4, 0), "the last row with text");
    assert_eq!(t.transcript().text(), "two\nthree\nfour\nf", "the selection grows from its start");
}

// PANES-55: another thread in the chat pane drops the frozen rows and the selection, and the
// cursor goes to the last row with text. The cursor on row 16 of a thread of 6 rows stopped zendo.
#[test]
fn a_shorter_thread_starts_the_transcript_over() {
    let mut t = Threads::default();
    on(&mut t);
    let long = numbered(17);
    t.view_rows("a", 80, true, rows(&long.iter().map(String::as_str).collect::<Vec<_>>()));
    t.transcript_mut().place(12, 0);
    t.transcript_mut().key("select", 1, std::time::Instant::now());
    t.transcript_mut().place(16, 3);
    let (shown, new) = t.view_rows("b", 80, true, rows(&["one", "two", "three", "", "five", ""]));
    assert_eq!((shown.lines.len(), new), (6, 0), "the rows of the other thread");
    assert_eq!(t.transcript().cursor(), (4, 0), "the last row with text");
    assert!(!t.transcript().selecting(), "no text is selected");
    assert_eq!(t.fit(6, 20, true), 0);
}

// PANES-56: a new width freezes the new rows, drops the selection and puts the cursor on the last
// row with text; `o` on a step freezes the new rows, keeps the cursor and drops the selection.
#[test]
fn a_new_width_or_a_step_freezes_the_new_rows() {
    let mut t = Threads::default();
    on(&mut t);
    t.view_rows("a", 80, true, rows(&["one", "two", "three"]));
    t.transcript_mut().place(0, 0);
    t.transcript_mut().key("select", 1, std::time::Instant::now());
    let (shown, _) = t.view_rows("a", 60, true, rows(&["one", "two", "thr", "ee", ""]));
    assert_eq!(shown.lines.len(), 5, "the rows of the new width");
    assert_eq!(t.transcript().cursor(), (3, 0));
    assert!(!t.transcript().selecting());
    t.transcript_mut().place(1, 1);
    t.transcript_mut().key("select", 1, std::time::Instant::now());
    t.refreeze(Refreeze::Step);
    let (shown, _) = t.view_rows("a", 60, true, rows(&["one", "two", "  step", "thr", "ee", ""]));
    assert_eq!(shown.lines.len(), 6, "the step is open");
    assert_eq!(t.transcript().cursor(), (1, 1), "the cursor stays on the step");
    assert!(!t.transcript().selecting());
}
