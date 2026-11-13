---
status: "draft"
revision: "1"
date: "2026-11-12"
parent: "terminal.md"
next: "terminal-view.md"
level: "shell blocks in the transcript: what a shell block is, the play button, F5 and alt-F5, which terminal takes a block"
note: "Agent draft from the requests of the user on 2026-11-12, split from terminal.md. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Shell blocks

**Shell blocks.** A shell block is a fenced code block of a shell language, or a code block where
each line with text starts with the prompt `$ `. From a prompt block, zendo takes only the text
after each `$ `; the other lines are output. zendo shows a green play button `▶` on the first line
of each shell block in the transcript.

**`F5` and `alt-F5`.** The user moves the cursor of the transcript into a block and presses `F5`, or
clicks `▶`: zendo puts the command into the shell of a terminal, does not run it, and shows the
terminal (see [Terminal view](terminal-view.md)). `alt-F5` puts the command in and runs it at once.
`F5` outside a shell block, or in another pane, shows the `F5` terminal.

**Which terminal.** A block goes to the first terminal that does not run a command: `F5`, then the
`[>]` terminals from the top, then a new `[>]` terminal. If all five terminals run a command, the
block goes nowhere, and zendo tells the user why (TERM-4 in [Terminal](terminal.md)).

## Architecturally significant requirements

| ID      | Pattern | Requirement | Source |
| ------- | ------- | ----------- | ------ |
| TERM-1  | Event   | When the user presses `F5` while the cursor of the transcript is in a shell block, the system shall put the command of the block into the shell of a terminal that does not run a command, without running it, and show that terminal. | User, 2026-11-12 |
| TERM-21 | Event   | When the user presses `alt-F5` while the cursor of the transcript is in a shell block, the system shall put the command into a terminal as `F5` does, and run it. | User, 2026-11-12 |

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
| TERM-11 | Event      | When the user presses `F5` outside a shell block, or in another pane, the system shall show the `F5` terminal. | User, 2026-11-12 |

Tests that verify detailed requirements: `play_button_and_terminal_buttons` in `tests/terminal.rs`
(TERM-8, TERM-11, TERM-14, TERM-16), `shell_blocks_are_shell_fences_and_prompt_blocks` and
`a_prompt_block_gives_only_the_commands` in `src/text/shell_tests.rs` (TERM-7, TERM-9).
