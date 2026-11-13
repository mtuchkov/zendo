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
