---
status: "draft"
revision: "4"
date: "2026-11-18"
parent: "terminal.md"
prev: "terminal-view.md"
next: "terminal-watch.md"
level: "the result of the last command of a terminal: how zendo finds it, and how the user puts it into the chat of an agent session"
note: "Draft from the requests of the user on 2026-11-12. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document. A child of terminal.md. Revision 2 changed TERM-25: a watched command also gives its output to the agent (terminal-watch.md, user, 2026-11-17). Revision 4 (user, 2026-11-18, in chat; not reviewed): TERM-26 and TERM-27 changed, TERM-74 to TERM-76 added."
---

# Terminal output

A command in a [terminal](terminal.md) fails, and the user wants an agent to see why. zendo keeps the
result of the last command of each terminal: the command, the exit code, and the output without ANSI
codes. In the input of a chat, `alt-o` puts the result of the terminal that the user viewed last into
the input, and does not send it. The user reads it, changes it if necessary, and presses `enter`. An
agent sees the output of a terminal only when the user puts it in, or when the user started the
command with `▶▶` ([Watched commands](terminal-watch.md)).

**Through the terminals.** The next `alt-o` replaces the inserted text with the result of the next
terminal, from the terminal viewed last to the terminal viewed first, and then from the start again.
A terminal without a result does not count. The result of a command that still runs is the output up
to that moment, marked as not complete.

**Long output.** Of a long output, zendo keeps the last 500 lines and tells how many lines it
dropped.

**The limits of a command.** zendo starts the shell as it is: it gives the shell no startup file
and no variable for its startup files, so that an endpoint security product does not see a change of
the shell configuration ([ADR 0018](../../architecture/decisions/0018-shells-start-as-they-are.md)).
A command ends when the terminal becomes free (no program of the command runs, and the terminal was
quiet for 300 ms); the exit code is then unknown. This rule holds also while the shell sends OSC 133
marks, so that marks that are not complete do not hold a command forever. When the shell of the user
sends the marks itself (a shell integration of the user), the end mark ends the command earlier and
gives the exit code, and a new prompt ends a command whose end mark got lost.

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TERM-23 | Ubiquitous | The system shall keep the result of the last command of each terminal: the command, the exit code, and the output. | User, 2026-11-12 |
| TERM-24 | Event      | When the user presses the insert-output key in the input of a chat, the system shall put into that input the result of the terminal that the user viewed last, without sending it. | User, 2026-11-12 |
| TERM-25 | Ubiquitous | The system shall give the output of a terminal to an agent only when the user puts it into the input, or when the user watches the command for that agent (TERM-55). | Design; TERM-2; User, 2026-11-17 |
| TERM-26 | Event      | When the shell sends the OSC 133 command-end mark before the command ends by TERM-27, the system shall end the command at the mark, take the exit code from the mark, and take the output from the output-start mark, or from the moment when the command went into the shell when no output-start mark came. | User, 2026-11-12; User, 2026-11-18 |
| TERM-27 | Event      | When no program of the command runs in the terminal and the terminal was quiet for 300 ms, the system shall end the command, also while the shell sends OSC 133 marks, take as the result the output from the moment when the command went into the shell, and show the exit code as unknown. | User, 2026-11-12; User, 2026-11-18 |

### Acceptance tests

The tests are in `tests/terminal_output.rs`. They use `bash` with a `.bashrc` that sends the marks
as the shell with marks, and `/bin/sh` as the shell without marks.

**`alt_o_puts_the_result_into_the_input_and_does_not_send_it`** (terminal check; verifies TERM-23, TERM-24, TERM-25, TERM-26)

- Given the `F5` terminal with `bash` and a `.bashrc` of the user that sends the marks, where
  `echo zendo-out; false` ended, and the user in the input of a chat.
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
| TERM-34 | —          | Retired in revision 3: zendo loads no script into the shell. Replaced by TERM-55. | User, 2026-11-18 |
| TERM-35 | Ubiquitous | The system shall not show the OSC 133 marks on the screen of the terminal. | Design |
| TERM-36 | Ubiquitous | The system shall take the insert-output key from the keymap, with `alt-o` as the default. | Design; TERM-19 |
| TERM-55 | Ubiquitous | The system shall start the shell of a terminal as it is: without a startup file of zendo, and without a change to the variables that select the startup files of the shell (`ZDOTDIR`, `ENV`, `BASH_ENV`). | User, 2026-11-18; ADR 0018 |
| TERM-74 | Unwanted   | If the OSC 133 prompt-start or command-line mark comes while a command runs, then the system shall end the command with the output up to the mark, and shall show the exit code as unknown. | User, 2026-11-18 |
| TERM-75 | Event      | When the command-end mark comes after the command ended by TERM-27 and before the next command, the system shall give the result the exit code of the mark. | Design, 2026-11-18 |
| TERM-76 | Ubiquitous | The system shall start the shell of a terminal without the variable `VTE_VERSION`, because the terminal of zendo is not a VTE terminal and `vte.sh` would add marks that a prompt of the user can break. | User, 2026-11-18 |

Tests that verify detailed requirements: `alt_o_again_goes_through_the_terminals`,
`a_command_that_runs_gives_output_that_is_not_complete`, `no_result_puts_nothing`,
`a_long_output_keeps_the_last_500_lines`, `bash_starts_as_it_is`, `zsh_starts_as_it_is` and
`fish_starts_as_it_is` in `tests/terminal_output.rs` (TERM-28 to TERM-33, TERM-55);
`a_c_mark_without_d_ends_when_the_terminal_is_quiet`, `a_b_mark_without_c_ends_when_the_terminal_is_quiet`,
`an_a_mark_in_the_output_ends_the_command`, `a_b_mark_in_the_output_ends_the_command` and
`marks_without_c_end_a_quiet_command_and_a_late_d_gives_the_exit_code` in
`src/session/output_tests.rs`, and `a_shell_gets_no_vte_version` in `src/session/pty_tests.rs`
(TERM-27, TERM-74 to TERM-76).

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| One result is kept for each terminal. Is a history of more than one command necessary? | TERM-23 | User |

Revision 2 changed TERM-34: the POSIX shells get the marks too, so that a command that prints
nothing for a long time does not end before its end.

Revision 3 retired TERM-34 and added TERM-55: CrowdStrike Falcon killed zendo together with the zsh
that zendo started with its own `ZDOTDIR` (ADR 0018). A command of the shell that prints nothing for
a long time and starts no program (a loop of the shell) can now end before its end.

Revision 4 made TERM-27 hold also with marks, and added TERM-74 to TERM-76: with `VTE_VERSION` of
the terminal of the user, `vte.sh` sent the output-start mark from `preexec`, the prompt of the user
dropped the end mark, and zendo waited for the end of each command forever.
