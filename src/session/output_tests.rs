use super::*;

fn marks(r: &mut Recorder, chunks: &[&[u8]]) {
    chunks.iter().for_each(|c| r.feed(c));
}

// TERM-26, TERM-33: with marks, the result is the output between C and D, and the exit code of D;
// not the prompt, and no ANSI codes.
#[test]
fn marks_give_the_output_and_the_exit_code() {
    let mut r = Recorder::default();
    marks(&mut r, &[b"\x1b]133;A\x07$ \x1b]133;B\x07", b"echo hi; false\r\n", b"\x1b]133;C\x07\x1b[31mhi\x1b[0m\r\n", b"\x1b]133;D;1\x07\x1b]133;A\x07$ "]);
    let o = r.result().unwrap();
    assert_eq!((o.command.as_str(), o.exit, o.text.as_str(), o.done), ("echo hi; false", Some(1), "hi", true));
}

// TERM-26: a mark that comes in two reads, and the ESC \ end of a mark.
#[test]
fn a_mark_in_two_reads() {
    let mut r = Recorder::default();
    marks(&mut r, &[b"\x1b]133;B\x07ls\r\n\x1b]1", b"33;C\x1b\\a\r\nb\r\n\x1b]133;D;0", b"\x07"]);
    assert_eq!(r.result().unwrap().text, "a\nb");
    assert_eq!(r.result().unwrap().exit, Some(0));
}

// TERM-30: while the command runs, the result is the output so far, not complete.
#[test]
fn a_command_that_runs_gives_what_it_printed() {
    let mut r = Recorder::default();
    marks(&mut r, &[b"\x1b]133;B\x07sleep\r\n\x1b]133;C\x07part\r\n"]);
    let o = r.result().unwrap();
    assert_eq!((o.text.as_str(), o.done), ("part", false));
}

// TERM-27: without marks, the result is from enter to the free terminal: no echo of the command,
// no prompt, and no exit code.
#[test]
fn without_marks_enter_to_free() {
    let mut r = Recorder::default();
    r.feed(b"$ ");
    r.enter("$ echo plain");
    r.feed(b"echo plain\r\nplain\r\n$ ");
    assert!(r.waits());
    r.free();
    let o = r.result().unwrap();
    assert_eq!((o.command.as_str(), o.exit, o.text.as_str(), o.done), ("$ echo plain", None, "plain", true));
}

// TERM-27: a command that is quiet for a long time (`sleep 1; echo first`) ends only when the
// terminal is free and quiet: the output that the shell prints just after the free terminal counts.
#[test]
fn without_marks_the_quiet_time_starts_at_the_free_terminal() {
    let t0 = std::time::Instant::now();
    let ms = |n| t0 + std::time::Duration::from_millis(n);
    let mut r = Recorder::default();
    r.enter("sleep 1; echo first");
    r.feed(b"sleep 1; echo first\r\n");
    r.poll_free(false, ms(5), ms(10));
    r.poll_free(true, ms(5), ms(500));
    // `sleep` ended: the terminal is free, and its last output is 1 s old.
    r.poll_free(false, ms(5), ms(1000));
    assert!(r.waits(), "the shell did not print the rest yet");
    r.feed(b"first\r\n# ");
    r.poll_free(false, ms(1002), ms(1100));
    assert!(r.waits(), "not quiet for long enough");
    r.poll_free(false, ms(1002), ms(1400));
    let o = r.result().unwrap();
    assert_eq!((o.text.as_str(), o.done), ("first", true));
}

// TERM-27: a command that prints and ends at once ends after the quiet time, and a busy terminal
// does not end it.
#[test]
fn without_marks_a_free_quiet_terminal_ends_the_command() {
    let t0 = std::time::Instant::now();
    let ms = |n| t0 + std::time::Duration::from_millis(n);
    let mut r = Recorder::default();
    r.enter("echo second");
    r.feed(b"echo second\r\nsecond\r\n# ");
    r.poll_free(true, ms(5), ms(400));
    assert!(r.waits(), "busy");
    r.poll_free(false, ms(5), ms(410));
    r.poll_free(false, ms(5), ms(720));
    assert_eq!(r.result().map(|o| (o.text, o.done)), Some(("second".to_string(), true)));
}

// TERM-26: once the shell sends marks, enter and free do not decide.
#[test]
fn marks_win_over_enter() {
    let mut r = Recorder::default();
    r.feed(b"\x1b]133;A\x07$ \x1b]133;B\x07");
    r.enter("$ x");
    assert!(!r.waits());
    r.free();
    assert_eq!(r.result(), None);
}

// TERM-32: of a long output, the last 500 lines and the number of the lines dropped.
#[test]
fn a_long_output_keeps_the_last_lines() {
    let mut r = Recorder::default();
    let out: String = (1..=600).map(|n| format!("{n}\r\n")).collect();
    marks(&mut r, &[b"\x1b]133;B\x07seq 1 600\r\n\x1b]133;C\x07", out.as_bytes(), b"\x1b]133;D;0\x07"]);
    let o = r.result().unwrap();
    assert_eq!((o.dropped, o.text.lines().next(), o.text.lines().last()), (100, Some("101"), Some("600")));
}

// TERM-33: the text without ANSI codes: colors, OSC titles, carriage returns and backspaces.
#[test]
fn plain_text() {
    assert_eq!(plain(b"\x1b[1;32mok\x1b[0m\x1b]0;title\x07 done\r\n10%\r50%\r\nab\x08c"), "ok done\n50%\nac");
}
