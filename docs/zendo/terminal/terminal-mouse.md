---
status: "draft"
revision: "1"
date: "2026-11-12"
parent: "terminal.md"
prev: "terminal-output.md"
level: "the mouse over a terminal or a Claude Code session: who gets the wheel, the click and the drag"
note: "Agent draft from the bug report of the user on 2026-11-12: the wheel and the selection did not work in a terminal. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document. A child of terminal.md."
---

# Terminal mouse

zendo takes the mouse of the terminal of the user (see [Panes](../panes.md)). A terminal of zendo and
a Claude Code session run a program in a terminal that zendo owns, so zendo decides who gets each
mouse event over that screen: the program or zendo. The rules are the same in the chat pane and on
the full screen, and the same for a terminal and for a Claude Code session.

**The program reports the mouse.** Programs such as Claude Code with its full screen display, `vim`
and `htop` ask the terminal for the mouse events (xterm mouse tracking). zendo sends them the events
over their screen, with the cell on that screen: the press, the release, the drag and the wheel, as
the program asked for them. The program scrolls and selects by itself. A program that got the press
of a button also gets the release, also when the pointer is then outside its screen.

**The program is on the alternate screen and does not report the mouse.** Programs such as `less` and
`man` have no lines that left the screen. The wheel sends them the arrow-up or arrow-down key, one
for each line, as most terminals do.

**All other programs, such as the shell at its prompt.** The wheel scrolls through the lines that
left the screen. A drag selects text of the screen, and the release copies it, as in each pane
(PANES-3, PANES-13). A key ends the selection.

To select text with the terminal of the user and not with the program, the user holds the modifier
key of that terminal, usually `shift` or `option`, as for zendo itself.

## Architecturally significant requirements

| ID      | Pattern | Requirement | Source |
| ------- | ------- | ----------- | ------ |
| TERM-43 | State   | While the program of a terminal or of a Claude Code session reports the mouse, the system shall send each mouse event over the screen of that program to the program, in the mode and the encoding that the program set. | User, 2026-11-12 |

### Acceptance tests

The tests are in `tests/terminal_mouse.rs`, with `/bin/sh` as the shell. Each test runs two times:
with the terminal in the chat pane, and on the full screen.

**`a_program_in_the_chat_pane_gets_the_mouse`**, **`a_program_on_the_full_screen_gets_the_mouse`**
(terminal check; verifies TERM-43)

- Given a program in the `F5` terminal that turns on the mouse reports with the SGR encoding, and
  prints each byte that it gets.
- When the user presses the left button on the top left cell of the program, drags, releases, and
  turns the wheel up and down.
- Then the program gets the press, the drag, the release and the two wheel events, with cells
  counted from the top left cell of its screen; zendo selects no text.

## Detailed requirements

| ID      | Pattern | Requirement | Source |
| ------- | ------- | ----------- | ------ |
| TERM-44 | Event   | When the user turns the wheel over a program that is on the alternate screen and does not report the mouse, the system shall send the arrow-up or the arrow-down key to the program, one for each line of the scroll. | Design |
| TERM-45 | Event   | When the user turns the wheel over a program that is on the main screen and does not report the mouse, the system shall scroll through the lines that left the screen of the program. | User, 2026-11-12; PANES-14 |
| TERM-46 | Event   | When the user drags the mouse over a program that does not report the mouse, in the chat pane or on the full screen, the system shall select the text of its screen, and copy it when the user releases the button. | User, 2026-11-12; PANES-3, PANES-13 |
| TERM-47 | Event   | When the program of a terminal in the chat pane gets the press of a mouse button, the system shall give the keys to that terminal. | Design |

`the_wheel_scrolls_a_program_…` verifies TERM-44, `the_wheel_scrolls_a_terminal_…` verifies TERM-45,
and `a_drag_copies_from_a_terminal_…` verifies TERM-46, each `…in_the_chat_pane` and
`…on_the_full_screen`. `a_program_in_the_chat_pane_gets_the_mouse` also verifies TERM-47. The unit tests of `src/session/mouse.rs` verify the modes and the encodings
of TERM-43.

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| A program that reports the mouse copies its selection by itself. When it uses only the OSC 52 sequence, as Claude Code does over SSH, zendo drops the sequence and nothing gets to the clipboard. Shall zendo pass OSC 52 to the terminal of the user? | TERM-43 | User |
| The wheel over the main screen scrolls three lines for each event, also for a trackpad. Shall PANES-20 apply here too? | TERM-45 | User |
