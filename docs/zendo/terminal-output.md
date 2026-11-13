---
status: "draft"
revision: "1"
date: "2026-11-12"
parent: "zendo.md"
prev: "terminal.md"
next: "attention.md"
level: "the result of the last command of a terminal: how zendo finds it, and how the user puts it into the chat of an agent session"
note: "Agent draft from the requests of the user on 2026-11-12. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document. A sibling of terminal.md, because terminal.md is over the length limit."
---

# Terminal output

A command in a [terminal](terminal.md) fails, and the user wants an agent to see why. zendo keeps the
result of the last command of each terminal: the command, the exit code, and the output without ANSI
codes. In the input of a chat, `alt-o` puts the result of the terminal that the user viewed last into
the input, and does not send it. The user reads it, changes it if necessary, and presses `enter`. An
agent sees the output of a terminal only when the user puts it in.

**Through the terminals.** The next `alt-o` replaces the inserted text with the result of the next
terminal, from the terminal viewed last to the terminal viewed first, and then from the start again.
A terminal without a result does not count. The result of a command that still runs is the output up
to that moment, marked as not complete.

**Long output.** Of a long output, zendo keeps the last 500 lines and tells how many lines it
dropped.

**The limits of a command.** The shell tells zendo where the output of a command starts and ends,
and its exit code, with OSC 133 marks. zendo loads a short script for the marks when it starts bash,
zsh or fish, and does not change the files of the user. For a shell without marks, such as
`/bin/sh`, the result is the output from the moment when the command went into the shell to the
moment when the terminal became free; the exit code is then unknown.

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TERM-23 | Ubiquitous | The system shall keep the result of the last command of each terminal: the command, the exit code, and the output. | User, 2026-11-12 |
| TERM-24 | Event      | When the user presses the insert-output key in the input of a chat, the system shall put into that input the result of the terminal that the user viewed last, without sending it. | User, 2026-11-12 |
| TERM-25 | Ubiquitous | The system shall give the output of a terminal to an agent only when the user puts it into the input. | Design; TERM-2 |
| TERM-26 | State      | While the shell sends OSC 133 marks, the system shall take as the result the output between the output-start mark and the command-end mark, and the exit code from the command-end mark. | User, 2026-11-12 |
| TERM-27 | Unwanted   | If the shell sends no OSC 133 marks, then the system shall take as the result the output from the moment when the command went into the shell to the moment when the terminal became free, and shall show the exit code as unknown. | User, 2026-11-12 |

### Acceptance tests

The tests are in `tests/terminal_output.rs`. They use `bash` as the shell with marks, and `/bin/sh`
as the shell without marks.

**`alt_o_puts_the_result_into_the_input_and_does_not_send_it`** (terminal check; verifies TERM-23, TERM-24, TERM-25, TERM-26)

- Given the `F5` terminal with `bash`, where `echo zendo-out; false` ended, and the user in the
  input of a chat.
- When the user presses `alt-o`.
- Then the input holds the command, the exit code `1` and `zendo-out`, and not the prompt of the
  shell. The agent got no message.

**`a_shell_without_marks_gives_the_output_and_no_exit_code`** (terminal check; verifies TERM-27)

- Given the `F5` terminal with `/bin/sh`, where `alt-F5` ran a block `echo plain-out`.
- When the user presses `alt-q`, and `alt-o` in the input of the chat.
- Then the input holds `plain-out`, and shows the exit code as unknown.

**`alt_o_takes_the_terminal_viewed_last`** (terminal check; verifies TERM-24)

- Given the `F5` terminal where `echo first` ended, and a `[>]` terminal where `echo second` ended
  later. The user viewed `[>]`, and then `F5`.
- When the user presses `alt-o` in the input of a chat.
- Then the input holds `first`, and not `second`.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TERM-28 | Event      | When the user presses the insert-output key again before any other key, the system shall replace the inserted result with the result of the next terminal, in the sequence from the terminal viewed last to the terminal viewed first, and then from the start again. | User, 2026-11-12 |
| TERM-29 | State      | While the system goes through the terminals, the system shall skip each terminal that has no result. | Design |
| TERM-30 | State      | While a terminal runs a command, the system shall give the output up to that moment, and mark it as not complete. | Design |
| TERM-31 | Unwanted   | If no terminal has a result, then the system shall put nothing into the input, and shall tell the user. | Design |
| TERM-32 | Event      | When the output is longer than 500 lines, the system shall keep the last 500 lines, and shall tell how many lines it dropped. | User, 2026-11-12 |
| TERM-33 | Ubiquitous | The system shall put the result into the input as the command, the exit code, and the output without ANSI codes. | Design |
| TERM-34 | Event      | When a terminal starts a shell that is bash, zsh or fish, the system shall load its OSC 133 script into that shell, without a change to the files of the user. | User, 2026-11-12 |
| TERM-35 | Ubiquitous | The system shall not show the OSC 133 marks on the screen of the terminal. | Design |
| TERM-36 | Ubiquitous | The system shall take the insert-output key from the keymap, with `alt-o` as the default. | Design; TERM-19 |

Tests that verify detailed requirements: `alt_o_again_goes_through_the_terminals`,
`a_command_that_runs_gives_output_that_is_not_complete`, `no_result_puts_nothing`,
`a_long_output_keeps_the_last_500_lines` and `the_marks_do_not_change_the_files_of_the_user` in
`tests/terminal_output.rs` (TERM-28 to TERM-35), and the unit tests of the result in
`src/session/output_tests.rs` (TERM-26, TERM-27, TERM-32, TERM-33).

## Design notes

- bash: zendo starts the shell with `--rcfile` set to its own file. That file reads `~/.bashrc`, and
  then adds the marks.
- zsh: zendo sets `ZDOTDIR` to its own directory. The files there read the files of the user from
  the old `ZDOTDIR` or the home directory, and then add the marks.
- fish: zendo gives the script with `-C`.
- The scripts are in the binary of zendo; zendo writes them to `.zendo/shell/`. If the shell of the
  user already sends OSC 133 marks, the script does not send them a second time.

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| One result is kept for each terminal. Is a history of more than one command necessary? | TERM-23 | User |
