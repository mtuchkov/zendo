---
status: "draft"
revision: "1"
date: "2026-11-12"
parent: "terminal.md"
prev: "shell-blocks.md"
next: "terminal-output.md"
level: "the view of a terminal: the chat pane, the full screen, the keys to leave and to go to another session, the buttons"
note: "Agent draft from the requests of the user on 2026-11-12, split from terminal.md. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Terminal view

**The chat pane.** A terminal shows in the chat pane, in place of the transcript and the input; the
other panes stay on the screen. The user types into the shell there. The terminal has the size of
the chat pane.

**The full screen.** `ctrl-o` makes the terminal full screen, with the bottom line of the session
view (see [Sessions](../sessions.md)). `ctrl-o` is not a toggle: on the full screen it goes to the
shell. The full screen ends when the user leaves the terminal; the next time, the terminal shows in
the chat pane again.

**Leave and go on.** `alt-q` leaves the terminal from both modes, and the command continues.
`alt-n` goes to the next agent session, and after the last one through the terminals: `F5`, then the
`[>]` terminals from the top, then the first agent session again. All other keys and paste go to the
shell, except the keys that choose a session.

**Buttons.** Under the agent buttons `F6` to `F10` come a gap with a horizontal line, `F5`, and the
`[>]` buttons. Terminal buttons have the color of the text, whitish in a dark theme, so that they
differ from agent sessions at once.

```text
 F9
 F10
 ──
 F5
 [>]
```

## Architecturally significant requirements

| ID      | Pattern | Requirement | Source |
| ------- | ------- | ----------- | ------ |
| TERM-37 | Event   | When the system shows a terminal, the system shall show it in the chat pane, and keep the other panes on the screen. | User, 2026-11-12 |
| TERM-38 | Event   | When the user presses `ctrl-o` while the chat pane shows a terminal, the system shall show that terminal on the full screen. | User, 2026-11-12 |
| TERM-41 | Event   | When the user presses `alt-n`, the system shall show the next item of this sequence: the agent sessions, the `F5` terminal, the `[>]` terminals from the top, and then the first agent session again. | User, 2026-11-12 |

### Acceptance tests

The tests are in `tests/terminal.rs`, with `/bin/sh` as the shell.

**`a_terminal_shows_in_the_chat_pane`** (terminal check; verifies TERM-37, TERM-40)

- Given a screen of 120 columns and 40 lines, and an answer with a shell block.
- When the user presses `alt-F5` on the block, and runs `stty size`.
- Then the chat pane shows the terminal, the note is on the screen, and the shell has the width of
  the chat pane, not 120.

**`ctrl_o_makes_the_terminal_full_screen`** (terminal check; verifies TERM-38, TERM-39, TERM-40, TERM-42)

- Given the `F5` terminal in the chat pane.
- When the user presses `ctrl-o`, runs `stty size`, presses `ctrl-o` again, presses `alt-q`, and
  then `F5`.
- Then after the first `ctrl-o` the shell has the size `39 120`; the second `ctrl-o` goes to the
  shell; after `F5` the terminal is in the chat pane and the note is on the screen.

**`alt_n_goes_through_the_terminals`** (terminal check; verifies TERM-41)

- Given the `F5` terminal and one `[>]` terminal that run, and the chat on agent session 5.
- When the user presses `alt-n` three times.
- Then the chat pane shows `F5`, then the `[>]` terminal, then agent session 1.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TERM-13 | State      | While a terminal has the keys, the system shall send all keys and paste to the shell, except `alt-q`, `alt-n`, the keys that choose a session, and `ctrl-o` in the chat pane. | User, 2026-11-12; SESSIONS-9 |
| TERM-22 | Event      | When the user presses `alt-q` in a terminal, in the chat pane or on the full screen, the system shall show the screen from where the user opened the terminal, as SESSIONS-10 does for a Claude Code session. | User, 2026-11-12 |
| TERM-39 | State      | While a terminal is on the full screen, the system shall send `ctrl-o` to the shell. | User, 2026-11-12 |
| TERM-40 | Event      | When the area of a terminal changes, the system shall set the size of the terminal to the size of the new area. | Design |
| TERM-42 | Event      | When the user leaves a terminal that is on the full screen, or goes to a different terminal or session, the system shall stop the full screen mode of that terminal. | User, 2026-11-12 |
| TERM-14 | Ubiquitous | The system shall show the terminal buttons under the agent buttons, after a gap with a horizontal line: first `F5`, then a `[>]` button for each open `[>]` terminal. | User, 2026-11-12 |
| TERM-48 | Unwanted   | If the keymap in effect gives the first terminal another key than `F5`, then the system shall name it `[>]`, as the other terminals. | User, 2026-11-13 |
| TERM-15 | Ubiquitous | The system shall show the terminal buttons in the color of the text of the theme, and mark the session on the screen in the bottom line of the full screen. | User, 2026-11-12 |
| TERM-16 | State      | While a terminal runs a command, the system shall show a busy mark on its button. | Design |
| TERM-19 | Ubiquitous | The system shall take the keys of the terminals from the keymap, with `f5`, `alt-f5`, `alt-q`, `alt-n` and `ctrl-o` as the defaults. | PANES-17 |

`play_button_and_terminal_buttons` in [Shell blocks](shell-blocks.md) verifies TERM-14 and TERM-16.
`other_session_keys_name_the_buttons_1_to_5_and_the_terminal_by_its_mark` in `tests/keymap_hints.rs`
verifies TERM-48.

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| `F5` to `F10` choose a session, so programs such as `htop` and `mc` do not get them. Is that acceptable? | TERM-13 | User |
| In the chat pane, `ctrl-o` does not go to the shell, so `nano` (write the file) and bash (`operate-and-get-next`) get it only on the full screen. Is that acceptable? | TERM-13, TERM-39 | User |
| On the full screen of a Claude Code session, `alt-n` also leaves the full screen and goes to the next item. Is that acceptable? | TERM-41, TERM-42 | User |
