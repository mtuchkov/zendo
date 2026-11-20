---
status: "draft"
revision: "1"
date: "2026-11-14"
parent: "panes.md"
next: "transcript-selection.md"
level: "how the user selects and copies text in the panes: the mouse, tables, the selection keys"
note: "Agent draft, split from panes.md on 2026-11-14; no requirement changed. PANES-34 to PANES-45 are agent drafts. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Text selection

The user selects text with a drag of the mouse or with the selection keys. A selection stays inside
the pane where it started. When the user releases the mouse button, zendo copies the selection to
the clipboard, also over SSH. See [Panes and layouts](panes.md) for the panes.

**The mouse.** zendo takes the mouse. To use the selection of the terminal, the user holds a
modifier key, usually Shift, or Option in iTerm2. While the mouse selects, the status bar says
`MOUSE SELECTION` and names the keys of the drag. A release with Alt puts the selection into the
chat input as a chip instead of the clipboard.

**Tables.** A selection that starts in a cell of a table with box borders (`┌ ─ ┬ │ ├ ┼ └`) keeps
to the columns of the table: a drag down a column copies only that column. The selection keys stop
at the edge of the cell; two quick presses in one direction go on to the next cell. A selection
that starts outside the table goes line by line. A drag with Ctrl (or Shift, where the terminal
gives it to zendo) goes line by line also in a table.

**Copy and cut.** While text is selected in the note in write mode or in the chat input, `c` copies
and `x` cuts; the keys are in `[selection]` of the [keymap](keymap.md).

## Architecturally significant requirements

| ID       | Pattern  | Requirement | Source |
| -------- | -------- | ----------- | ------ |
| PANES-3  | Event    | When the user drags the mouse, the system shall select text only inside the pane where the drag started.                                                                                                | User, 2026-11-11 |
| PANES-34 | Event    | When the user starts a selection in a cell of a table with box borders, with the mouse or with the keys, the system shall keep the selection to the rows of the table and to the columns from the cell of its start to the cell of its end. | User, 2026-11-13 |

### Acceptance tests

**`selection_stays_inside_its_panel`** (unit test; verifies PANES-3)

- Given the note and the chat side by side.
- When the user drags from the note into the chat.
- Then the selection holds only lines of the note.

**`a_drag_in_a_table_copies_the_column`** (integration test; verifies PANES-34)

- Given a note with a table of two columns, in read mode.
- When the user drags from a cell of the second column down two rows.
- Then the clipboard holds only the text of the second column, one row for each line, without the
  lines of the table.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| PANES-13 | Event      | When the user releases the mouse button after a drag, the system shall copy the selected text to the clipboard, also over SSH.       | User, 2026-11-11 |
| PANES-35 | Ubiquitous | The system shall look for the borders of a table at most 100 rows up and down from the start of the selection. Where the rows of a pane end before a border, the border on the other side shall be enough. | User, 2026-11-13 |
| PANES-36 | Unwanted   | If a selection starts outside a table, or in a table drawn with `\|`, `-` and `+`, then the system shall select the text line by line. | User, 2026-11-13 |
| PANES-40 | Event      | When the user copies a selection in a table, the system shall put a tab between the cells of a row, leave out the lines and the rules of the table, and leave out the rows with no text. | Design |
| PANES-41 | Event      | When the end of a selection in a table goes above or below the table, the system shall select the column of the start to its end; when it goes left or right of the table, the system shall stop it at the first or the last column. | Design |
| PANES-42 | State      | While a selection that started in a table grows by the keys, the system shall stop the cursor at the edge of its cell, and shall move it on to the next cell in the direction of the key when the user presses the key again 60 to 400 ms after the stop. | User, 2026-11-13 |
| PANES-43 | State      | While the user holds Ctrl or Shift during a drag of the mouse, the system shall select whole lines, also in a table. | User, 2026-11-13 |
| PANES-44 | Event      | When the user releases the mouse button with Alt held during the drag, the system shall put the selection into the chat input as a chip with its file and lines, move the focus to the chat input, and leave the clipboard as it is. | User, 2026-11-13 |
| PANES-45 | State      | While the user drags the mouse to select, the system shall show `MOUSE SELECTION` in the badge of the status bar, and the hints of the release, Alt, Ctrl and the cancel. | User, 2026-11-13 |
| PANES-37 | State      | While text is selected in the note in write mode or in the chat input, the system shall copy the selection when the user presses `c`, and cut it when the user presses `x`, before the other keys of the pane. | User, 2026-11-13 |
| PANES-38 | Unwanted   | If no text is selected, then the system shall type `c` and `x` as text. | User, 2026-11-13 |
| PANES-39 | Ubiquitous | The system shall take the keys of PANES-37 from `[selection]` of the keymap, and show them in the help of write mode and of the chat as a section of their own. | User, 2026-11-13 |

Tests that verify detailed requirements: the tests of `src/text/boxtable_tests.rs` (PANES-34,
PANES-35, PANES-36, PANES-40, PANES-41), `the_keys_of_read_mode_select_in_a_column`,
`a_drag_from_before_a_table_goes_line_by_line` and
`a_selection_in_a_table_of_the_transcript_keeps_to_the_column` (PANES-34, PANES-36), the tests of
`src/text/softstop_tests.rs`, `the_keys_stop_at_the_edge_of_a_cell_and_two_quick_presses_go_on` and
`the_keys_of_the_transcript_stop_at_the_edge_of_a_cell` (PANES-42),
`ctrl_or_shift_selects_whole_lines_in_a_table` (PANES-43), `alt_marks_the_selection_for_the_chat` and
`alt_and_a_drag_put_the_selection_into_the_chat` (PANES-44, PANES-45),
`c_copies_and_x_cuts_the_selection_in_write_mode` and `x_cuts_the_selection_in_the_chat_input`
(PANES-37), `without_a_selection_c_and_x_are_text` (PANES-38),
`the_selection_keys_are_in_the_keymap_and_the_help` (PANES-39).

The identifiers keep the topic `PANES`: they came from [Panes and layouts](panes.md).
