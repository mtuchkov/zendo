---
status: "draft"
approved: "2026-11-18"
revision: "3"
date: "2026-11-18"
parent: "terminal.md"
next: "terminal-view.md"
level: "shell blocks in the transcript: what a shell block is, the play buttons, F5 and alt-F5, which terminal takes a block"
note: "From the requests of the user on 2026-11-12, split from terminal.md. Revision 2 (the user, 2026-11-17): TERM-11 refers to TERM-52 for `F5` in a terminal; the user approved it and marked it implemented on 2026-11-18. Revision 3 (agent draft from the requests of the user on 2026-11-17, not reviewed): `alt-F5` and `▶▶` run and watch (terminal-watch.md); a terminal belongs to a session while its watch runs and 10 min after it."
---

# Shell blocks

**Shell blocks.** A shell block is a fenced code block of a shell language, or a code block where
each line with text starts with the prompt `$ `. From a prompt block, zendo takes only the text
after each `$ `; the other lines are output. zendo shows a green play button `▶` and a play-and-watch button `▶▶` on the
first line of each shell block in the transcript.

**`F5` and `alt-F5`.** The user moves the cursor of the transcript into a block and presses `F5`, or
clicks `▶`: zendo puts the command into the shell of a terminal, does not run it, and shows the
terminal (see [Terminal view](terminal-view.md)). `alt-F5`, or a click on `▶▶`, puts the command in, runs it at once, and gives its output to the
agent ([Watched commands](terminal-watch.md)).
`F5` outside a shell block, or in another pane, shows the `F5` terminal.

**Which terminal.** The terminals are one pool for all agent sessions. A block of a session goes
first to the terminal that belongs to that session, when it does not run a command. Else it goes to
the first free terminal: `F5`, then the `[>]` terminals from the top, then a new `[>]` terminal. If
no terminal is free, the block goes nowhere, and zendo tells the user why (TERM-4 in
[Terminal](terminal.md)).

**A terminal of a session.** A watched command of a session ([Watched commands](terminal-watch.md))
makes its terminal belong to that session until the watch ends, and for 10 min after. In that time
the terminal is not free for a block of another session: that block goes to another terminal, or to
a new one. The session gets the same shell again, with its folder after `cd`. The user can always
type into any terminal.

## Architecturally significant requirements

| ID      | Pattern | Requirement | Source |
| ------- | ------- | ----------- | ------ |
| TERM-1  | Event   | When the user presses `F5` while the cursor of the transcript is in a shell block, the system shall put the command of the block into the shell of a terminal that does not run a command, without running it, and show that terminal. | User, 2026-11-12 |
| TERM-21 | Event   | When the user presses `alt-F5` while the cursor of the transcript is in a shell block, the system shall put the command into a terminal as `F5` does, run it, and watch it (TERM-55). | User, 2026-11-12; User, 2026-11-17 |
| TERM-72 | State   | While a watch of a session runs, and for 10 min after it ends, the system shall keep its terminal for that session, and shall not put a block of another session into it. | User, 2026-11-17 |
| TERM-73 | Event   | When a block of a session goes to a terminal, the system shall choose the free terminal of that session first, then the first free terminal that belongs to no session, then a new `[>]` terminal. | User, 2026-11-17 |

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

**`a_terminal_stays_with_its_session`** (terminal check; verifies TERM-72, TERM-73)

- Given a watched block `cd src` of session 1 that ended 1 min ago in the `F5` terminal.
- When the user runs a block `pwd` of session 2, and then a block `pwd` of session 1.
- Then the block of session 2 runs in a new `[>]` terminal and shows the root of the repository;
  the block of session 1 runs in the `F5` terminal and shows `src`.

**`a_busy_f5_terminal_opens_a_new_one`** (terminal check; verifies TERM-1, TERM-3, TERM-6, TERM-22)

- Given the `F5` terminal that runs `sleep 30`.
- When the user presses `alt-q`, and `alt-F5` on a block `echo second`.
- Then a `[>]` terminal shows `second`, and `sleep` still runs.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TERM-7  | Ubiquitous | The system shall take as a shell block each fenced code block with the language `sh`, `bash`, `zsh`, `fish`, `shell` or `console`, and each code block where each line with text starts with `$ `. | Design |
| TERM-8  | Ubiquitous | The system shall show a green play button `▶` on the first line of each shell block in the transcript. | User, 2026-11-12 |
| TERM-9  | Event      | When the user takes a block with `$ ` prompts, the system shall put into the shell only the text after each prompt, one command on each line. | Design |
| TERM-10 | Event      | When the user clicks the play button of a shell block, the system shall do what `F5` does. | Design |
| TERM-11 | Event      | When the user presses `F5` outside a shell block, or in another pane, while no terminal is on the screen, the system shall show the `F5` terminal; in a terminal, `F5` goes to the next terminal (TERM-52). | User, 2026-11-12; User, 2026-11-17 |

Tests that verify detailed requirements: `play_button_and_terminal_buttons` in `tests/terminal.rs`
(TERM-8, TERM-11, TERM-14), `shell_blocks_are_shell_fences_and_prompt_blocks` and
`a_prompt_block_gives_only_the_commands` in `src/text/shell_tests.rs` (TERM-7, TERM-9).
