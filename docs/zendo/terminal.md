---
status: "draft"
revision: "2"
date: "2026-11-12"
parent: "zendo.md"
prev: "chat.md"
next: "terminal-output.md"
level: "terminals of zendo: shell blocks in the chat, the play button, F5 and [>] terminals"
note: "Agent draft from the request of the user on 2026-11-12. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document. Revision 2: the terminal shows in the chat pane, `ctrl-o` makes it full screen, `alt-n` goes through the terminals (user, 2026-11-12). Depends on Sessions (sessions.md, draft) and ADR 0005."
---

# Terminal

An answer of the agent often holds a shell command. zendo finds such a command in the transcript
and shows a green play button `▶` on it. The user moves the cursor into the command and presses
`F5`: zendo puts the command into the shell of a terminal, does not run it, and shows the terminal
in the chat pane.
The user reads the command, changes it if necessary, and presses `enter`. `alt-F5` puts the command
in and runs it at once. The agent never runs the command; only the user does (CLAUDE-3 stays).

**Shell blocks.** A shell block is a fenced code block of a shell language, or a code block where
each line with text starts with the prompt `$ `. From a prompt block, zendo takes only the text
after each `$ `; the other lines are output.

**Terminals.** A terminal is a shell that zendo runs in the root of the repository, in a terminal
that zendo owns, as a Claude Code session does (see [Sessions](sessions.md) and
[ADR 0005](../architecture/decisions/0005-sessions-in-own-terminals.md)). The shell keeps its state,
for example the folder after `cd`, until it exits. There are at most five terminals:

- The **`F5` terminal** takes the blocks of `F5` and the ad hoc commands of the user: `F5` outside a
  shell block opens it.
- A **`[>]` terminal** is an extra one, at most four. A block goes to the first terminal that does
  not run a command: `F5`, then the `[>]` terminals from the top, then a new `[>]` terminal. A
  `[>]` terminal goes away when its shell exits.

**The terminal view.** A terminal shows in the chat pane, in place of the transcript and the input;
the other panes stay on the screen. The user types into the shell there. `ctrl-o` makes the terminal
full screen, with the bottom line of the session view. `ctrl-o` is not a toggle: on the full screen
it goes to the shell. The full screen ends when the user leaves the terminal; the next time, the
terminal shows in the chat pane again. `alt-q` leaves the terminal from both modes, and the command
continues. `alt-n` goes to the next agent session, and after the last one through the terminals:
`F5`, then the `[>]` terminals from the top, then the first agent session again.

**Buttons.** Under the agent buttons `F6` to `F10` come a gap with a horizontal line, `F5`, and the
`[>]` buttons. Terminal buttons have the color of the text, whitish in a dark theme, so that they
differ from agent sessions at once.

```text
 F9
 F10
 ──
 F5
 [>]
 [>]
```

## Architecturally significant requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| TERM-1 | Event      | When the user presses `F5` while the cursor of the transcript is in a shell block, the system shall put the command of the block into the shell of a terminal that does not run a command, without running it, and show that terminal. | User, 2026-11-12 |
| TERM-21 | Event     | When the user presses `alt-F5` while the cursor of the transcript is in a shell block, the system shall put the command into a terminal as `F5` does, and run it. | User, 2026-11-12 |
| TERM-2 | Ubiquitous | The system shall run the text of an agent message only when the user presses `alt-F5` on it, or `enter` in the terminal. | User, 2026-11-12; CLAUDE-3 |
| TERM-3 | Ubiquitous | The system shall have at most five terminals: one `F5` terminal and at most four `[>]` terminals. | User, 2026-11-12 |
| TERM-4 | Unwanted   | If five terminals are open and each one runs a command, then the system shall not put the block into a terminal, and shall tell the user. | User, 2026-11-12 |
| TERM-5 | Ubiquitous | The system shall run each terminal in a terminal that zendo owns, without tmux, a multiplexer, or a server process. | SESSIONS-3 |
| TERM-6 | Event      | When the user leaves a terminal, the system shall keep its command running until it ends or zendo quits. | SESSIONS-1 |
| TERM-37 | Event     | When the system shows a terminal, the system shall show it in the chat pane, and keep the other panes on the screen. | User, 2026-11-12 |
| TERM-38 | Event     | When the user presses `ctrl-o` while the chat pane shows a terminal, the system shall show that terminal on the full screen. | User, 2026-11-12 |
| TERM-41 | Event     | When the user presses `alt-n`, the system shall show the next item of this sequence: the agent sessions, the `F5` terminal, the `[>]` terminals from the top, and then the first agent session again. | User, 2026-11-12 |

### Acceptance tests

The tests are in `tests/terminal.rs`. They put answers with ` ```sh ` blocks into the thread, and use
`/bin/sh` as the shell.

**`f5_puts_the_block_into_the_shell_and_does_not_run_it`** (terminal check; verifies TERM-1, TERM-2, TERM-5, TERM-12)

- Given an answer with a block `touch ran`, and the cursor of the transcript in it.
- When the user presses `F5`, and then `enter`.
- Then after `F5` the `F5` terminal shows `touch ran` on its command line and `ran` does not exist;
  after `enter`, `ran` exists. No tmux runs.

**`alt_f5_runs_the_block`** (terminal check; verifies TERM-21)

- Given an answer with a block `echo zendo-$((40+2))`, and the cursor of the transcript in it.
- When the user presses `alt-F5`.
- Then the `F5` terminal shows `zendo-42`.

**`a_shell_block_does_not_run_by_itself`** (terminal check; verifies TERM-2)

- Given an answer with a block `touch ran`.
- When the user moves the cursor over the block and leaves the chat.
- Then `ran` does not exist.

**`a_busy_f5_terminal_opens_a_new_one`** (terminal check; verifies TERM-1, TERM-3, TERM-6, TERM-22)

- Given the `F5` terminal that runs `sleep 30`.
- When the user presses `alt-q`, and `alt-F5` on a block `echo second`.
- Then a `[>]` terminal shows `second`, and `sleep` still runs.

**`five_busy_terminals_take_nothing`** (terminal check; verifies TERM-4, TERM-18)

- Given the `F5` terminal and four `[>]` terminals that each run `sleep 30`.
- When the user presses `alt-F5` on a block `touch ran`.
- Then no new terminal opens, the chat tells the user why, and `ran` does not exist.

**`a_terminal_shows_in_the_chat_pane`** (terminal check; verifies TERM-37, TERM-40)

- Given a screen of 120 columns and 40 lines, and an answer with a block `tput cols`.
- When the user presses `alt-F5` on the block.
- Then the chat pane shows the terminal, the note pane is on the screen, and the terminal shows the
  width of the chat pane, not `120`.

**`ctrl_o_makes_the_terminal_full_screen`** (terminal check; verifies TERM-38, TERM-39, TERM-40, TERM-42)

- Given the `F5` terminal in the chat pane.
- When the user presses `ctrl-o`, runs `tput cols`, presses `ctrl-o` again, presses `alt-q`, and
  then `F5`.
- Then after the first `ctrl-o` no other pane is on the screen and the terminal shows `120`; the
  second `ctrl-o` goes to the shell; after `F5` the terminal is in the chat pane and the note pane is
  on the screen.

**`alt_n_goes_through_the_terminals`** (terminal check; verifies TERM-41)

- Given the `F5` terminal and one `[>]` terminal that run, and the chat on agent session 5.
- When the user presses `alt-n` three times.
- Then the chat pane shows `F5`, then the `[>]` terminal, then agent session 1.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TERM-7  | Ubiquitous | The system shall take as a shell block each fenced code block with the language `sh`, `bash`, `zsh`, `fish`, `shell` or `console`, and each code block where each line with text starts with `$ `. | Design |
| TERM-8  | Ubiquitous | The system shall show a green play button `▶` on the first line of each shell block in the transcript. | User, 2026-11-12 |
| TERM-9  | Event      | When the user takes a block with `$ ` prompts, the system shall put into the shell only the text after each prompt, one command on each line. | Design |
| TERM-10 | Event      | When the user clicks the play button of a shell block, the system shall do what `F5` does. | Design |
| TERM-11 | Event      | When the user presses `F5` outside a shell block, or in another pane, the system shall show the `F5` terminal. | User, 2026-11-12 |
| TERM-12 | Event      | When a terminal is used the first time, the system shall start the shell of the user in the root of the repository. | Design |
| TERM-13 | State      | While a terminal has the keys, the system shall send all keys and paste to the shell, except `alt-q`, `alt-n`, the keys that choose a session, and `ctrl-o` in the chat pane. | User, 2026-11-12; SESSIONS-9 |
| TERM-22 | Event      | When the user presses `alt-q` in a terminal, in the chat pane or on the full screen, the system shall show the screen from where the user opened the terminal, as SESSIONS-10 does for a Claude Code session. | User, 2026-11-12 |
| TERM-39 | State      | While a terminal is on the full screen, the system shall send `ctrl-o` to the shell. | User, 2026-11-12 |
| TERM-40 | Event      | When the area of a terminal changes, the system shall set the size of the terminal to the size of the new area. | Design |
| TERM-42 | Event      | When the user leaves a terminal that is on the full screen, or goes to a different terminal or session, the system shall stop the full screen mode of that terminal. | User, 2026-11-12 |
| TERM-14 | Ubiquitous | The system shall show the terminal buttons under the agent buttons, after a gap with a horizontal line: first `F5`, then a `[>]` button for each open `[>]` terminal. | User, 2026-11-12 |
| TERM-15 | Ubiquitous | The system shall show the terminal buttons in the color of the text of the theme, and mark the session on the screen in the bottom line of the terminal view. | User, 2026-11-12 |
| TERM-16 | State      | While a terminal runs a command, the system shall show a busy mark on its button. | Design |
| TERM-17 | Event      | When the shell of a terminal exits, the system shall remove a `[>]` terminal and its button, and start the `F5` shell again with the next use of `F5`. | Design |
| TERM-18 | Unwanted   | If the user quits zendo while a terminal runs a command, then the system shall ask the user before it stops the command. | SESSIONS-6 |
| TERM-19 | Ubiquitous | The system shall take the keys of the terminals from the keymap, with `f5`, `alt-f5`, `alt-q`, `alt-n` and `ctrl-o` as the defaults. | PANES-17 |
| TERM-20 | Optional   | Where zendo runs with `--offline` or with any agent, the system shall give the user the `F5` terminal. | Design |

Tests that verify detailed requirements: `play_button_and_terminal_buttons` (TERM-8, TERM-11,
TERM-14, TERM-16), `shell_blocks_are_shell_fences_and_prompt_blocks` and
`a_prompt_block_gives_only_the_commands` in `src/text/shell_tests.rs` (TERM-7, TERM-9).

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| In the terminal view, `F5` to `F10` choose a session, so programs such as `htop` and `mc` do not get them. Is that acceptable? | TERM-13 | User |
| In the chat pane, `ctrl-o` does not go to the shell, so `nano` (write the file) and bash (`operate-and-get-next`) do not get it there; they get it on the full screen. Is that acceptable? | TERM-13, TERM-39 | User |
| In the session view of a Claude Code session, `alt-n` leaves the full screen and goes to the next item, as in a terminal. Is that acceptable? | TERM-41, TERM-42 | User |
| This document has about 2,000 words, over the limit of about 1,000. Split it into a `terminal/` directory with children (a change of the project structure)? | AGENTS.md | User |
