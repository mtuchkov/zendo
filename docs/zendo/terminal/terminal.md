---
status: "draft"
revision: "4"
date: "2026-11-13"
parent: "../zendo.md"
prev: "../chat.md"
next: "../attention.md"
children:
  - "shell-blocks.md"
  - "terminal-view.md"
  - "terminal-output.md"
  - "terminal-mouse.md"
level: "the terminals of zendo as a whole: what a terminal is, how many, who runs a command"
note: "Agent draft from the requests of the user on 2026-11-12. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document. Revision 3: split into children by topic (user, 2026-11-12). Depends on Sessions (sessions.md, draft) and ADR 0005."
---

# Terminal

An answer of the agent often holds a shell command. zendo shows a green play button `▶` on it. The
user presses `F5` on the command: zendo puts it into the shell of a terminal and does not run it.
The user reads the command, changes it if necessary, and presses `enter`. The agent never runs a
command; only the user does (CLAUDE-3 stays).

**Terminals.** A terminal is a shell that zendo runs in the root of the repository, in a terminal
that zendo owns, as a Claude Code session does (see [Sessions](../sessions.md) and
[ADR 0005](../../architecture/decisions/0005-sessions-in-own-terminals.md)). The shell keeps its
state, for example the folder after `cd`, until it exits. There are at most five terminals:

- The **`F5` terminal** takes the blocks of `F5` and the ad hoc commands of the user.
- A **`[>]` terminal** is an extra one, at most four. It goes away when its shell exits.

A command continues when the user leaves its terminal. If commands run when the user quits zendo,
zendo asks first. With `--offline` and with each agent, the `F5` terminal is available.

**The parts.**

| Document                                 | Content                                                            |
| ---------------------------------------- | ------------------------------------------------------------------ |
| [Shell blocks](shell-blocks.md)          | Shell blocks in the transcript, `▶`, `F5` and `alt-F5`, which terminal takes a block |
| [Terminal view](terminal-view.md)        | The terminal in the chat pane, `ctrl-o` for the full screen, `alt-q`, `alt-]` and `alt-[`, the keys, the buttons |
| [Terminal output](terminal-output.md)    | `alt-o` puts the result of the last command into the input of a chat |
| [Terminal mouse](terminal-mouse.md)      | The wheel, the click and the drag over a terminal: for the program or for zendo |

## Architecturally significant requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| TERM-2 | Ubiquitous | The system shall run the text of an agent message only when the user presses `alt-F5` on it, or `enter` in the terminal. | User, 2026-11-12; CLAUDE-3 |
| TERM-3 | Ubiquitous | The system shall have at most five terminals: one `F5` terminal and at most four `[>]` terminals. | User, 2026-11-12 |
| TERM-4 | Unwanted   | If five terminals are open and each one runs a command, then the system shall not put the block into a terminal, and shall tell the user. | User, 2026-11-12 |
| TERM-5 | Ubiquitous | The system shall run each terminal in a terminal that zendo owns, without tmux, a multiplexer, or a server process. | SESSIONS-3 |
| TERM-6 | Event      | When the user leaves a terminal, the system shall keep its command running until it ends or zendo quits. | SESSIONS-1 |

### Acceptance tests

The tests are in `tests/terminal.rs`. They put answers with ` ```sh ` blocks into the thread, and use
`/bin/sh` as the shell. `f5_puts_the_block_into_the_shell_and_does_not_run_it` and
`a_busy_f5_terminal_opens_a_new_one` in [Shell blocks](shell-blocks.md) also verify TERM-5 and
TERM-6.

**`a_shell_block_does_not_run_by_itself`** (terminal check; verifies TERM-2)

- Given an answer with a block `touch ran`.
- When the user moves the cursor over the block and leaves the chat.
- Then `ran` does not exist, and no terminal started.

**`five_busy_terminals_take_nothing`** (terminal check; verifies TERM-3, TERM-4, TERM-18)

- Given the `F5` terminal and four `[>]` terminals that each run `sleep 30`.
- When the user presses `alt-F5` on a block `touch ran`, and then quits.
- Then no new terminal opens, the chat tells the user why, `ran` does not exist, and the quit asks
  first.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TERM-12 | Event      | When a terminal is used the first time, the system shall start the shell of the user in the root of the repository. | Design |
| TERM-17 | Event      | When the shell of a terminal exits, the system shall remove a `[>]` terminal and its button, and start the `F5` shell again with the next use of `F5`. | Design |
| TERM-18 | Unwanted   | If the user quits zendo while a terminal runs a command, then the system shall ask the user before it stops the command. | SESSIONS-6 |
| TERM-20 | Optional   | Where zendo runs with `--offline` or with any agent, the system shall give the user the `F5` terminal. | Design |
