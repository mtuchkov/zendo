---
status: "draft"
revision: "4"
date: "2026-11-13"
parent: "zendo.md"
next: "notes.md"
level: "the panes as a whole: layouts, widths, hiding, focus, selection, themes"
note: "Agent rewrite of the reference of zendo 0.9.1. Not reviewed by the user. PANES-34 to PANES-45 (selection in tables, keys of [selection]) are agent drafts."
---

# Panes and layouts

zendo has four panes: the file tree, the note, the chat with Claude and the delta. The note is the
main pane. The other panes open and close on the request of the user. Each pane has its own
document for what it shows: [Notes](notes.md), [File tree](files.md), [Claude](claude.md),
[Work, delta and recaps](work.md).

The user has two layout presets and no other layout settings. In wide screen, the open panes stand
side by side. In standard, the note and the chat share the screen, and the delta takes the place of
the note. The top bar shows the file and the branch; there are no buttons, every action has a key.

Keys in every pane: `tab` moves the focus to the next pane, `ctrl-shift-b` or `alt-b` opens or closes the file
tree,
`ctrl-←` and `ctrl-→` move the edge of the pane in focus, `alt-=` and `alt--` make the
text column wider or narrower (kept for the repository). The font size is the terminal's own: most
terminals change it with `ctrl-+` and `ctrl--`. In read mode: `w` switches the layout,
`t` switches the theme, `alt-?` (or `f1`, `?`) shows all keys.

A selection that starts in a cell of a table with box borders (`┌ ─ ┬ │ ├ ┼ └`) keeps to the
columns of the table: a drag down a column copies only that column. The selection keys stop at the
edge of the cell; two quick presses in one direction go on to the next cell. A selection that
starts outside the table goes line by line, as before. A drag with Ctrl (or Shift, where the terminal
gives it to zendo) goes line by line also in a table. A release with Alt puts the selection into the
chat input as a chip instead of the clipboard. While the mouse selects, the status bar says
`MOUSE SELECTION` and names these keys. While text is selected in the note in write mode or in the
chat input, `c` copies and `x` cuts; the keys are in `[selection]` of the keymap.

## Architecturally significant requirements

| ID       | Pattern  | Requirement                                                                                                                                                                                              | Source           |
| -------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| PANES-1  | Unwanted | If the open panes do not fit into the terminal, then the system shall hide panes from the edges by turns: first the rightmost pane right of the note, then the leftmost pane left of the note, and again. | User, 2026-11-11 |
| PANES-2  | Unwanted | If the system hides panes, then it shall keep the pane in focus, and shall hide the note last.                                                                                                          | User, 2026-11-11 |
| PANES-3  | Event    | When the user drags the mouse, the system shall select text only inside the pane where the drag started.                                                                                                | User, 2026-11-11 |
| PANES-34 | Event    | When the user starts a selection in a cell of a table with box borders, with the mouse or with the keys, the system shall keep the selection to the rows of the table and to the columns from the cell of its start to the cell of its end. | User, 2026-11-13 |

### Acceptance tests

**`panes_hide_from_the_right_then_the_left_by_turns`** (unit test; verifies PANES-1)

- Given the wide screen layout with the file tree, the note, the chat and the delta open.
- When the terminal gets narrower, step by step.
- Then the delta hides first, then the file tree, then the chat.

**`the_pane_in_focus_stays`** (unit test; verifies PANES-2)

- Given the same panes, with the focus on the delta.
- When the terminal is too narrow for all panes.
- Then the delta stays open, and another pane hides.

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

| ID       | Pattern    | Requirement                                                                                                                         | Source           |
| -------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| PANES-4  | Ubiquitous | The system shall have two layouts: wide screen and standard.                                                                         | User, 2026-11-11 |
| PANES-5  | Ubiquitous | The system shall use wide screen when the user did not choose a layout.                                                              | User, 2026-11-11 |
| PANES-6  | Event      | When the user switches the layout, the system shall keep the choice for the repository.                                              | Design           |
| PANES-7  | State      | While the layout is wide screen, the system shall show the open panes side by side in this order: file tree, note, chat, delta.      | User, 2026-11-11 |
| PANES-8  | State      | While the layout is standard and the user reads, the system shall show the delta in the place of the note.                           | Design           |
| PANES-9  | Ubiquitous | The system shall show the file tree on the left in both layouts.                                                                     | Design           |
| PANES-10 | Event      | When the user drags the separator between two panes, or presses `ctrl-←` or `ctrl-→`, the system shall change the width of the pane. | User, 2026-11-11 |
| PANES-11 | Ubiquitous | The system shall keep a minimum width for the note, and give the note the width that the other panes do not use.                    | Design           |
| PANES-12 | Event      | When the user changes the width of a pane, the system shall keep the width for the repository.                                       | Design           |
| PANES-13 | Event      | When the user releases the mouse button after a drag, the system shall copy the selected text to the clipboard, also over SSH.       | User, 2026-11-11 |
| PANES-14 | Event      | When the user turns the mouse wheel, the system shall scroll the pane under the pointer.                                             | Design           |
| PANES-20 | Event      | When scroll events come faster than one each 40 ms, as from a trackpad, the system shall scroll one line for each event and draw once for all the events that are waiting. | User, 2026-11-12 |
| PANES-15 | Event      | When the user presses `t`, the system shall show the next of the four themes, and keep the choice.                                   | User, 2026-11-11 |
| PANES-16 | Event      | When the user presses `alt-?` or `f1` in any pane or mode, the system shall show the keys of the pane in focus, or of the mode, and the global keys. | User, 2026-11-12 |
| PANES-26 | Ubiquitous | The system shall show the key of the help first in the status bar, in every pane and mode.                                          | User, 2026-11-12 |
| PANES-27 | Ubiquitous | The system shall show the right side of the status bar whole: the last key hints give way to it, and the key of the help stays.     | User, 2026-11-12 |
| PANES-17 | Ubiquitous | The system shall take the keys of every action of every pane from the default keymap file that it ships, then from the keymap file that the config names, then from `[keys.<pane>]` in the config. | User, 2026-11-11 |
| PANES-28 | Ubiquitous | The system shall take the key in each key hint, message and help text from the keymap in effect, for each action that the keymap has. | User, 2026-11-13 |
| PANES-29 | Unwanted   | If an action has no key in the keymap in effect, then the system shall show no hint for that action. | User, 2026-11-13 |
| PANES-30 | Ubiquitous | The system shall show the actions of each pane in groups, in the help and in the default keymap file: navigation, text selection, editing, features, sessions and terminals, panes and view, zendo. | User, 2026-11-13 |
| PANES-31 | Event      | When the system shows the help, the system shall show the keymap file that the config names, and the version of zendo; the commit and the local copy only for a build with uncommitted changes. | User, 2026-11-13 |
| PANES-32 | Unwanted   | If the user presses `ctrl-c`, then the system shall not quit: `ctrl-c` copies. | User, 2026-11-13 |
| PANES-33 | State      | While the chat pane has the focus, the system shall name it `CHAT` in the badge of the status bar, and `SHELL` while a shell terminal shows in the chat pane. | User, 2026-11-13 |
| PANES-18 | Ubiquitous | The system shall have a key for every action, and no buttons.                                                                         | User, 2026-11-11 |
| PANES-19 | Unwanted   | If `[keys]` in the config names an unknown pane, action or key, then the system shall keep the default keys and tell the user.       | Design           |
| PANES-21 | Event      | When the user opens the delta and it does not fit beside the file tree, the system shall close the file tree. | User, 2026-11-12 |
| PANES-22 | Event      | When the user presses `alt-=` or `alt--`, the system shall make the column of the open file 4 chars wider or narrower, the code column for a code file and the text column for markdown, and keep the width for the repository. | User, 2026-11-12 |
| PANES-23 | Event      | When the user runs `zendo keymap`, the system shall print the default keymap file, so that the user can start a keymap of their own from it. | User, 2026-11-12 |
| PANES-24 | Unwanted   | If one key has two actions in one pane of the keymap, then the system shall tell the user, and the key shall do the first action. | User, 2026-11-12 |
| PANES-25 | Unwanted   | If the terminal has no kitty keyboard protocol and sends `ctrl-shift` and a letter as `ctrl` and the letter, then the system shall do the action of `ctrl-shift` and the letter when `ctrl` and the letter has no action. | User, 2026-11-12 |
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

zendo takes the mouse. To use the selection of the terminal, the user holds a modifier key, usually
Shift, or Option in iTerm2.

Tests that verify detailed requirements: `the_delta_closes_the_tree_when_it_does_not_fit` (PANES-21),
`the_width_keys_change_the_text_column` and `the_width_keys_change_the_code_column_on_a_code_file` (PANES-22),
`the_status_bar_asks_for_zendo_doctor_without_a_structure_file` (PANES-27), `the_default_keymap_file_names_every_action` (PANES-17),
`a_user_keymap_file_then_the_config_change_keys`, `zendo_keymap_prints_the_default_keymap` (PANES-17,
PANES-23), `a_key_of_two_actions_is_a_problem` (PANES-24), `without_the_kitty_protocol_ctrl_hjkl_select`
(PANES-25), the tests of `tests/keymap_hints.rs` (PANES-28, PANES-29),
`the_keymap_file_groups_the_actions_as_the_help_does` and `each_group_of_a_pane_has_one_place` (PANES-30),
`the_help_shows_groups_the_keymap_file_and_the_version` (PANES-30, PANES-31),
`the_help_shows_the_details_of_the_version_only_for_a_build_with_changes` (PANES-31),
`ctrl_c_does_not_quit` (PANES-32),
`alt_enter_is_a_new_line_and_the_status_bar_says_so` and `a_terminal_shows_in_the_chat_pane` (PANES-33),
the tests of `src/text/boxtable_tests.rs` (PANES-34, PANES-35, PANES-36, PANES-40, PANES-41),
`the_keys_of_read_mode_select_in_a_column`, `a_drag_from_before_a_table_goes_line_by_line` and
`a_selection_in_a_table_of_the_transcript_keeps_to_the_column` (PANES-34, PANES-36),
the tests of `src/text/softstop_tests.rs`, `the_keys_stop_at_the_edge_of_a_cell_and_two_quick_presses_go_on`
and `the_keys_of_the_transcript_stop_at_the_edge_of_a_cell` (PANES-42),
`ctrl_or_shift_selects_whole_lines_in_a_table` (PANES-43), `alt_marks_the_selection_for_the_chat` and
`alt_and_a_drag_put_the_selection_into_the_chat` (PANES-44, PANES-45),
`c_copies_and_x_cuts_the_selection_in_write_mode` and `x_cuts_the_selection_in_the_chat_input`
(PANES-37), `without_a_selection_c_and_x_are_text` (PANES-38),
`the_selection_keys_are_in_the_keymap_and_the_help` (PANES-39).

The keys of the search box, the theme list, the question of a file and the file name filter of the
tree are not in the keymap. Their hints show those fixed keys.
