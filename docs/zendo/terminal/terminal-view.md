---
status: "approved"
approved: "2026-11-17"
revision: "6"
date: "2026-11-17"
parent: "terminal.md"
prev: "shell-blocks.md"
next: "terminal-output.md"
level: "the view of a terminal: the chat pane, the full screen, the keys to leave and to go to another session, the buttons"
note: "From the requests of the user on 2026-11-12, split from terminal.md; the user approved the requirements in the chat on 2026-11-14, with TERM-14 for the terminal buttons at the top of the right edge. Revision 5 (the user, 2026-11-17): TERM-15 marks the terminal in the chat pane as the active button, in the accent color. Revision 6 (agent draft from the request of the user on 2026-11-17, not reviewed): TERM-52 (`F5` goes through the terminals), TERM-53 and TERM-54 (the `[+]` button). These three IDs are proposals until the user reviews them."
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
`alt-]` goes to the next agent session, and after the last one through the terminals: `F5`, then the
`[>]` terminals from the top, then the first agent session again. `alt-[` (`alt-}` on macOS) goes the same way back.
`F5` in a terminal goes to the next terminal that is open: `F5`, then the `[>]` terminals from the
top, then `F5` again. It shows the next terminal in the chat pane, also from the full screen.
The keys keep the view: in the chat pane they show the next item in the chat pane; on the full
screen they show the next session or terminal that runs on the full screen.
`alt-n` is free for the new note in a [float](../float.md). All other keys and paste go to the
shell, except the keys that choose a session.

**Buttons.** Under the agent buttons `F6` to `F10` come a gap with a horizontal line, `F5`, and the
`[>]` buttons. Terminal buttons have the color of the text, whitish in a dark theme, so that they
differ from agent sessions at once. While a terminal shows in the chat pane, its button is the
active one, in the accent color, and no agent button is. Under the last terminal button comes
`[+]` while fewer than four `[>]` terminals are open: a click on it starts a new `[>]` terminal and
shows it in the chat pane.

```text
 F9
 F10
 ──
 F5
 [>]
 [+]
```

## Architecturally significant requirements

| ID      | Pattern | Requirement | Source |
| ------- | ------- | ----------- | ------ |
| TERM-37 | Event   | When the system shows a terminal, the system shall show it in the chat pane, and keep the other panes on the screen. | User, 2026-11-12 |
| TERM-38 | Event   | When the user presses `ctrl-o` while the chat pane shows a terminal, the system shall show that terminal on the full screen. | User, 2026-11-12 |
| TERM-41 | Event   | When the user presses `alt-]`, the system shall show the next item of this sequence: the agent sessions, the `F5` terminal, the `[>]` terminals from the top, and then the first agent session again. | User, 2026-11-12; User, 2026-11-13 |
| TERM-49 | Event   | Where the system runs on Linux, when the user presses `alt-[`, and where it runs on macOS, when the user presses `alt-}`, the system shall show the previous item of the sequence of TERM-41, and after the first agent session the last item. | User, 2026-11-13 |
| TERM-52 | Event   | When the user presses `F5` while a terminal is in the chat pane or on the full screen, the system shall show in the chat pane the next item of this sequence: the `F5` terminal, the open `[>]` terminals from the top, and then the `F5` terminal again. | User, 2026-11-17 |
| TERM-51 | State   | While a session or a terminal is on the full screen, when the user presses `alt-]` or the key of TERM-49, the system shall go to the next or the previous agent session or terminal that runs: it shall show one that runs on the full screen, and for an agent session without a Claude Code session it shall show the chat on that agent session. | User, 2026-11-13; User, 2026-11-14 |

### Acceptance tests

The tests are in `tests/terminal.rs`, with `/bin/sh` as the shell.

**`f5_goes_through_the_terminals`** (terminal check; verifies TERM-11, TERM-52)

- Given the `F5` terminal and two `[>]` terminals that run, and the focus on the note.
- When the user presses `F5` four times, then `ctrl-o`, and then `F5`.
- Then the chat pane shows `F5`, the first `[>]`, the second `[>]`, and `F5` again; after the last
  `F5` the chat pane shows the first `[>]`, and the full screen ends.

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

**`alt_bracket_goes_through_the_terminals`** (terminal check; verifies TERM-41, TERM-49)

- Given the `F5` terminal and one `[>]` terminal that run, and the chat on agent session 5.
- When the user presses `alt-]` three times, and then the key of TERM-49 three times.
- Then the chat pane shows `F5`, then the `[>]` terminal, then agent session 1; and then the `[>]`
  terminal, `F5`, and agent session 5.

**`alt_bracket_keeps_the_full_screen`** (terminal check; verifies TERM-51)

- Given the `F5` terminal and one `[>]` terminal that run, and `F5` on the full screen.
- When the user presses `alt-]`, and then the key of TERM-49 two times.
- Then the full screen shows the `[>]` terminal, then `F5` again, and then zendo shows the chat on
  agent session 5, which has no Claude Code session.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TERM-13 | State      | While a terminal has the keys, the system shall send all keys and paste to the shell, except `alt-q`, `alt-]`, `alt-[` (`alt-}` on macOS), `alt-n`, `alt-/`, the keys that choose a session, and `ctrl-o` in the chat pane. | User, 2026-11-12; SESSIONS-9 |
| TERM-22 | Event      | When the user presses `alt-q` in a terminal, in the chat pane or on the full screen, the system shall show the screen from where the user opened the terminal, as SESSIONS-10 does for a Claude Code session. | User, 2026-11-12 |
| TERM-39 | State      | While a terminal is on the full screen, the system shall send `ctrl-o` to the shell. | User, 2026-11-12 |
| TERM-40 | Event      | When the area of a terminal changes, the system shall set the size of the terminal to the size of the new area. | Design |
| TERM-42 | Event      | When the user leaves a terminal that is on the full screen, or goes to a different terminal or session, the system shall stop the full screen mode of that terminal. | User, 2026-11-12 |
| TERM-14 | Ubiquitous | The system shall show the terminal buttons on the right edge of the chat, first `F5`, then a `[>]` button for each open `[>]` terminal: while the agent buttons are on the right edge (NAMES-8), under them after a gap with a horizontal line; while the agent buttons are in the row above the chat header, from the top of the right edge with no line. | User, 2026-11-12; User, 2026-11-14 |
| TERM-48 | Unwanted   | If the keymap in effect gives the first terminal another key than `F5`, then the system shall name it `[>]`, as the other terminals. | User, 2026-11-13 |
| TERM-15 | Ubiquitous | The system shall show the terminal buttons in the color of the text of the theme, the button of the terminal in the chat pane in the accent color, and mark the session on the screen in the bottom line of the full screen. | User, 2026-11-12; User, 2026-11-17 |
| TERM-53 | State      | While fewer than four `[>]` terminals are open, the system shall show a `[+]` button under the last terminal button. | User, 2026-11-17 |
| TERM-54 | Event      | When the user clicks the `[+]` button, the system shall start a new `[>]` terminal and show it in the chat pane. | User, 2026-11-17 |
| TERM-19 | Ubiquitous | The system shall take the keys of the terminals from the keymap, with `f5`, `alt-f5`, `alt-q`, `alt-]`, `alt-[` on Linux or `alt-}` on macOS, and `ctrl-o` as the defaults. | PANES-17 |

`play_button_and_terminal_buttons` in [Shell blocks](shell-blocks.md) verifies TERM-14.
`the_terminal_in_the_chat_pane_is_the_active_button` in `tests/session_names.rs` verifies TERM-15.
`the_plus_button_opens_a_new_terminal` in `tests/terminal.rs` verifies TERM-53 and TERM-54.

Retired in revision 3: TERM-16 (a busy mark on the button of a terminal that runs a command); the user removed the dots from the buttons on 2026-11-14. Revision 3 changed TERM-51: the step leaves no agent session out. Retired identifiers are never reused.
`other_session_keys_name_the_buttons_1_to_5_and_the_terminal_by_its_mark` in `tests/keymap_hints.rs`
verifies TERM-48. `the_linux_keymap_gives_alt_brackets_to_the_sessions` and
`the_mac_keymap_gives_alt_brace_to_the_previous_session` in `tests/keymap_hints.rs` verify TERM-41 and
TERM-49; `the_mac_and_linux_keymaps_differ_only_in_the_previous_session` verifies TERM-19.

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| `F5` to `F10` choose a session, so programs such as `htop` and `mc` do not get them. Is that acceptable? | TERM-13 | User |
| In the chat pane, `ctrl-o` does not go to the shell, so `nano` (write the file) and bash (`operate-and-get-next`) get it only on the full screen. Is that acceptable? | TERM-13, TERM-39 | User |
