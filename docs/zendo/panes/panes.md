---
status: "draft"
revision: "6"
date: "2026-11-17"
parent: "../zendo.md"
next: "../notes.md"
children:
  - "selection.md"
  - "transcript-selection.md"
  - "keymap.md"
  - "text-input/text-input.md"
  - "key-change.md"
  - "key-hints.md"
level: "the panes as a whole: layouts, widths, hiding, focus, scrolling, themes"
note: "Agent rewrite of the reference of zendo 0.9.1. Revision 5 moves this document to panes/panes.md, and the selection and the keys to its children selection.md and keymap.md, to keep each document within two pages; no requirement changed. Revision 6 adds PANES-46: a click in a pane moves the focus and the keys there, and PANES-47: a click off a panel over the screen closes it. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Panes and layouts

zendo has four panes: the file tree, the note, the chat with Claude and the delta. The note is the
main pane. The other panes open and close on the request of the user. Each pane has its own
document for what it shows: [Notes](../notes.md), [File tree](../files/files.md), [Claude](../claude.md),
[Work, delta and recaps](../work.md). Five topics that hold for all panes are child documents:
[Text selection](selection.md), [Selection in the transcript](transcript-selection.md), [Keys, hints and help](keymap.md), [Text input](text-input/text-input.md),
[Change a key](key-change.md) and [Key hints](key-hints.md).

The user has two layout presets and no other layout settings. In wide screen, the open panes stand
side by side. In standard, the note and the chat share the screen, and the delta takes the place of
the note. The top bar shows the file and the branch; each frequently used action has a key.

Keys in every pane: `tab` moves the focus to the next pane, `ctrl-shift-b` or `alt-b` opens or
closes the file tree, `ctrl-←` and `ctrl-→` move the edge of the pane in focus, `alt-=` and `alt--`
make the text column wider or narrower (kept for the repository). The font size is the terminal's
own: most terminals change it with `ctrl-+` and `ctrl--`. In read mode: `w` switches the layout and
`t` switches the theme.

## Architecturally significant requirements

| ID       | Pattern  | Requirement | Source |
| -------- | -------- | ----------- | ------ |
| PANES-1  | Unwanted | If the open panes do not fit into the terminal, then the system shall hide panes from the edges by turns: first the rightmost pane right of the note, then the leftmost pane left of the note, and again. | User, 2026-11-11 |
| PANES-2  | Unwanted | If the system hides panes, then it shall keep the pane in focus, and shall hide the note last.                                                                                                          | User, 2026-11-11 |

### Acceptance tests

**`panes_hide_from_the_right_then_the_left_by_turns`** (unit test; verifies PANES-1)

- Given the wide screen layout with the file tree, the note, the chat and the delta open.
- When the terminal gets narrower, step by step.
- Then the delta hides first, then the file tree, then the chat.

**`the_pane_in_focus_stays`** (unit test; verifies PANES-2)

- Given the same panes, with the focus on the delta.
- When the terminal is too narrow for all panes.
- Then the delta stays open, and another pane hides.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| PANES-4  | Ubiquitous | The system shall have two layouts: wide screen and standard.                                                                         | User, 2026-11-11 |
| PANES-5  | Ubiquitous | The system shall use wide screen when the user did not choose a layout.                                                              | User, 2026-11-11 |
| PANES-6  | Event      | When the user switches the layout, the system shall keep the choice for the repository.                                              | Design           |
| PANES-7  | State      | While the layout is wide screen, the system shall show the open panes side by side in this order: file tree, note, chat, delta.      | User, 2026-11-11 |
| PANES-8  | State      | While the layout is standard and the user reads, the system shall show the delta in the place of the note.                           | Design           |
| PANES-9  | Ubiquitous | The system shall show the file tree on the left in both layouts.                                                                     | Design           |
| PANES-10 | Event      | When the user drags the separator between two panes, or presses `ctrl-←` or `ctrl-→`, the system shall change the width of the pane. | User, 2026-11-11 |
| PANES-11 | Ubiquitous | The system shall keep a minimum width for the note, and give the note the width that the other panes do not use.                    | Design           |
| PANES-12 | Event      | When the user changes the width of a pane, the system shall keep the width for the repository.                                       | Design           |
| PANES-14 | Event      | When the user turns the mouse wheel, the system shall scroll the pane under the pointer.                                             | Design           |
| PANES-46 | Event      | When the user clicks in a pane, the system shall move the focus and the keys to that pane, as `tab` does; in the chat pane, to the transcript for a click in the transcript, with its cursor under the pointer, and to the chat input for a click in any other place of the pane. | User, 2026-11-17 |
| PANES-47 | Event      | When the user clicks off a panel that stands over the screen (the float of a new note, the find panel or the help), the system shall close the panel as `esc` does. | User, 2026-11-17 |
| PANES-20 | Event      | When scroll events come faster than one each 40 ms, as from a trackpad, the system shall scroll one line for each event and draw once for all the events that are waiting. | User, 2026-11-12 |
| PANES-15 | Event      | When the user presses `t`, the system shall show the next of the four themes, and keep the choice.                                   | User, 2026-11-11 |
| PANES-21 | Event      | When the user opens the delta and it does not fit beside the file tree, the system shall close the file tree. | User, 2026-11-12 |
| PANES-22 | Event      | When the user presses `alt-=` or `alt--`, the system shall make the column of the open file 4 chars wider or narrower, the code column for a code file and the text column for markdown, and keep the width for the repository. | User, 2026-11-12 |

Tests that verify detailed requirements: `the_delta_closes_the_tree_when_it_does_not_fit` (PANES-21),
`the_width_keys_change_the_text_column` and `the_width_keys_change_the_code_column_on_a_code_file`
(PANES-22), the tests of `tests/pane_click.rs`,
`a_click_puts_the_cursor_of_the_transcript_under_the_pointer` and
`col_at_finds_the_char_under_the_pointer` (PANES-46), `a_click_off_the_float_closes_it`,
`a_click_off_the_help_closes_it` and `a_click_off_the_find_panel_closes_it` (PANES-47).

The other `PANES` identifiers are in [Text selection](selection.md) (PANES-3, PANES-13, PANES-34 to
PANES-45), in [Selection in the transcript](transcript-selection.md) (PANES-50 to PANES-58) and in [Keys, hints and help](keymap.md) (PANES-16 to PANES-19, PANES-23 to PANES-33).
