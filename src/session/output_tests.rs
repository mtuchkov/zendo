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

/// The prompt of `sh` with the marks of zendo: the end of the last command, then A and B (TERM-34).
fn sh_prompt(code: i32) -> Vec<u8> {
    format!("\x1b]133;D;{code}\x07\x1b]133;A\x07# \x1b]133;B\x07").into_bytes()
}

// TERM-26, TERM-34: `sh` sends no C mark: the output starts at enter, and D gives the end and the
// exit code. A command that zendo puts in.
#[test]
fn marks_without_c_start_the_output_at_enter() {
    let mut r = Recorder::default();
    r.feed(&sh_prompt(0));
    r.run("echo hi; false");
    marks(&mut r, &[b"echo hi; false\r\nhi\r\n", &sh_prompt(1)]);
    let o = r.result().unwrap();
    assert_eq!((o.command.as_str(), o.exit, o.text.as_str(), o.done), ("echo hi; false", Some(1), "hi", true));
}

// TERM-27, TERM-75: with marks, a command that is quiet and starts no program (a loop of the
// shell) ends when the terminal is free and quiet; a D mark that comes later gives its exit code.
#[test]
fn marks_without_c_end_a_quiet_command_and_a_late_d_gives_the_exit_code() {
    let t0 = std::time::Instant::now();
    let ms = |n| t0 + std::time::Duration::from_millis(n);
    let mut r = Recorder::default();
    r.feed(&sh_prompt(0));
    r.run("while :; do :; done; echo first");
    r.feed(b"while :; do :; done; echo first\r\nfirst\r\n");
    r.poll_free(false, ms(5), ms(10));
    r.poll_free(false, ms(5), ms(400));
    assert_eq!(r.result().map(|o| (o.text, o.exit, o.done)), Some(("first".into(), None, true)));
    r.feed(&sh_prompt(3));
    assert_eq!(r.result().map(|o| (o.text, o.exit)), Some(("first".into(), Some(3))));
}

// TERM-27, TERM-34: the user types the command; the command is the text after the B mark, without
// the prompt. An empty command line keeps the last result.
#[test]
fn marks_without_c_take_the_typed_command() {
    let mut r = Recorder::default();
    marks(&mut r, &[&sh_prompt(0), b"echo hi"]);
    r.enter("# echo hi");
    marks(&mut r, &[b"\r\nhi\r\n", &sh_prompt(0)]);
    r.enter("# ");
    marks(&mut r, &[b"\r\n", &sh_prompt(0)]);
    let o = r.result().unwrap();
    assert_eq!((o.command.as_str(), o.exit, o.text.as_str()), ("echo hi", Some(0), "hi"));
}

// TERM-26: bash, the first command after the start: the C mark comes after enter, and the output
// starts at C.
#[test]
fn a_c_mark_after_enter_starts_the_output_again() {
    let mut r = Recorder::default();
    marks(&mut r, &[b"\x1b]133;A\x07$ \x1b]133;B\x07", b"echo hi"]);
    r.enter("$ echo hi");
    marks(&mut r, &[b"\r\n\x1b]133;C\x07hi\r\n\x1b]133;D;0\x07"]);
    let o = r.result().unwrap();
    assert_eq!((o.command.as_str(), o.exit, o.text.as_str()), ("echo hi", Some(0), "hi"));
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

// TERM-27: fish without marks redraws its line and ends its prompt with a carriage return; the
// output stays, and only the prompt goes.
#[test]
fn fish_without_marks_keeps_the_output() {
    let mut r = Recorder::default();
    r.feed(b"PROMPT> ");
    r.run("echo rc=$HOME");
    r.feed(b"\x1b[K\r\x1b[39C\r\n\x1b[30m\x1b(B\x1b[m\x1b[?2004l\x1b]0;echo rc=$HOME /h/c/zendo\x07\x1b[30m\x1b(B\x1b[m\rrc=/tmp/fh\r\n");
    r.feed("\x1b[2m\u{23ce}\x1b(B\x1b[m      \r\u{23ce} \r\x1b[K\x1b[?2004h\x1b]0;/h/c/zendo\x07\x1b[30m\x1b(B\x1b[mPROMPT> \x1b[K\r\x1b[8C".as_bytes());
    r.free();
    let o = r.result().unwrap();
    assert_eq!((o.text.as_str(), o.exit), ("rc=/tmp/fh", None));
}

/// Feed `bytes`, then a free terminal that is quiet for longer than the quiet time.
fn quiet_end(r: &mut Recorder, bytes: &[u8]) {
    let t0 = std::time::Instant::now();
    r.feed(bytes);
    r.poll_free(false, t0, t0 + std::time::Duration::from_millis(1));
    r.poll_free(false, t0, t0 + std::time::Duration::from_millis(400));
}

// TERM-27, TERM-76: vte.sh sends C from preexec, and a prompt of the user (oh-my-zsh) dropped D:
// the free quiet terminal ends the command, without the prompt.
#[test]
fn a_c_mark_without_d_ends_when_the_terminal_is_quiet() {
    let mut r = Recorder::default();
    r.feed(b"PROMPT> echo hi");
    r.enter("PROMPT> echo hi");
    quiet_end(&mut r, b"\r\n\x1b]133;C\x1b\\\rhi\r\nPROMPT> ");
    let o = r.result().unwrap();
    assert_eq!((o.text.as_str(), o.exit, o.done), ("hi", None, true));
}

// TERM-27: a prompt with B and no C or D: the free quiet terminal ends the command.
#[test]
fn a_b_mark_without_c_ends_when_the_terminal_is_quiet() {
    let mut r = Recorder::default();
    r.feed(b"$ \x1b]133;B\x07echo hi");
    r.enter("$ echo hi");
    quiet_end(&mut r, b"\r\nhi\r\n$ ");
    let o = r.result().unwrap();
    assert_eq!((o.command.as_str(), o.text.as_str(), o.exit, o.done), ("echo hi", "hi", None, true));
}

// TERM-74: an A mark while the command runs (the D mark got lost) ends the command with its
// output, and the exit code is unknown.
#[test]
fn an_a_mark_in_the_output_ends_the_command() {
    let mut r = Recorder::default();
    marks(&mut r, &[b"\x1b]133;B\x07ls\r\n\x1b]133;C\x07a\r\nb\r\n", b"\x1b]133;A\x07$ \x1b]133;B\x07"]);
    let o = r.result().unwrap();
    assert_eq!((o.command.as_str(), o.text.as_str(), o.exit, o.done), ("ls", "a\nb", None, true));
}

// TERM-74: a B mark while the command runs (a prompt with B only) ends the command too.
#[test]
fn a_b_mark_in_the_output_ends_the_command() {
    let mut r = Recorder::default();
    marks(&mut r, &[b"\x1b]133;B\x07ls\r\n\x1b]133;C\x07a\r\n", b"$ \x1b]133;B\x07"]);
    let o = r.result().unwrap();
    assert_eq!((o.text.as_str(), o.exit, o.done), ("a", None, true));
}
