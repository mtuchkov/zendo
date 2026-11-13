---
status: "implemented"
approved: "2026-11-13"
revision: "2"
date: "2026-11-13"
parent: "zendo.md"
prev: "notes-list.md"
next: "find.md"
level: "the new note in a float: how it opens, what it shows, how it closes"
note: "Agent draft of the decisions of the user on 2026-11-13, approved by the user. Revision 2: the test names and kinds as in the code."
---

# New note in a float

A new note opens in a float: a window that stands over the open file. The float shows only the note.
There is no file tree, no chat, no delta and no key hints in it, so the user writes without
distraction. The screen behind the float stays as it was, in dim colors.

`alt-n` opens the float in every pane and every mode, also over the full screen of a Claude Code
session or a terminal. `n` in read mode and `n` in the notes list open the same float. The float is
the only way to write a new note. A note that is already there opens in the note pane, as every
other file. Notes, titles and file names are
in [Notes list and search](notes-list.md); write mode is in [Write mode](write.md).

## What the float shows

- The top line: the path of the note on the left, in dim colors; the branch and `[ esc close ]` on
  the right. Until the first save, the path is the notes folder and `new note`.
- The text of the note in write mode, with the head of the note: the title in bold, and under it the
  date and the time when the user opened the float, in dim colors.

Design: the float is as wide as the text column, with a margin of 4 chars on each side. It is as
tall as the screen, less one row at the top and one at the bottom. It stands over the note pane.
Where the note pane is narrower than the float, the float stands at the center of the screen.

## Keys and saving

The keys of write mode work in the float as in the note pane: move, select, bold, italic, cut and
paste, `say`. Keys that open a pane or move the focus to another pane do nothing. `alt-?` shows the
keys of the float.

The float saves as write mode does: after a pause, without a save key. At the first save, the file
gets its name from the date and the title (NOTE-7). A note with no text makes no file.

`esc` saves the note and closes the float. When the note has no `# ` title, the agent writes it, as
when the user leaves write mode. After the float closes, the screen is as it was before: the same
file, pane, mode, cursor and selection. The new note is in the notes list and in the recent files
([Find](find.md)); zendo does not open it in the note pane.

## Architecturally significant requirements

| ID      | Pattern  | Requirement                                                                                                                                             | Source           |
| ------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| FLOAT-1 | Event    | When the user presses `alt-n` in any pane or mode, the system shall open a new note in write mode in a float over the open file.                         | User, 2026-11-13 |
| FLOAT-2 | State    | While the float is open, the system shall show only the float with its note in normal colors, the screen behind in dim colors, and no key hints.         | User, 2026-11-13 |
| FLOAT-3 | Event    | When the user closes the float, the system shall save the note and show the screen as it was before the float: the same file, pane, mode, cursor and selection. | Design           |
| FLOAT-4 | Unwanted | If the user opens the float while the open file is in write mode, then the system shall save the open file first.                                      | Design           |

### Acceptance tests

**`alt_n_opens_a_float_from_every_pane`** (unit test; verifies FLOAT-1)

- Given a repository with one note, open in read mode.
- When the user presses `alt-n` in read mode, in write mode, in the chat, in the file tree and in the
  delta.
- Then each time a float opens in write mode with an empty note and the cursor after `# `.

**`the_float_hides_the_panes_and_the_hints`** (integration test on a test terminal of 120×40; verifies FLOAT-2)

- Given the wide screen layout with the file tree, the note and the chat open.
- When the user presses `alt-n`.
- Then the float shows the notes folder, `new note`, the branch and `[ esc close ]` on its top line,
  the panes behind show in dim colors, and the status bar shows no key hints.

**`esc_saves_the_float_and_restores_the_screen`** (unit test; verifies FLOAT-3)

- Given `docs/design.md` open in read mode with the cursor on line 12 and a selection.
- When the user presses `alt-n`, types `# Tea` and a line of text, and presses `esc`.
- Then `docs/notes/<date>-tea.md` holds the text, and `docs/design.md` is open in read mode with the
  cursor on line 12 and the same selection.

**`the_float_saves_the_open_file_first`** (unit test; verifies FLOAT-4)

- Given `docs/design.md` in write mode with text that is not saved yet.
- When the user presses `alt-n`.
- Then `docs/design.md` on disk has the text, before the user types in the float.

## Detailed requirements

| ID       | Pattern    | Requirement                                                                                                                                         | Source           |
| -------- | ---------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| FLOAT-5  | Event      | When the user presses `n` in read mode or in the notes list, the system shall open the new note in the float.                                       | User, 2026-11-13 |
| FLOAT-6  | Ubiquitous | The system shall show on the top line of the float the path of the note, the branch and `[ esc close ]`, and the notes folder and `new note` until the first save. | User, 2026-11-13 |
| FLOAT-7  | Ubiquitous | The system shall show under the title of the note in the float the date and the time when the user opened the float.                               | User, 2026-11-13 |
| FLOAT-8  | Ubiquitous | The system shall make the float as wide as the text column with a margin of 4 chars on each side, and as tall as the screen less one row at the top and at the bottom. | Design           |
| FLOAT-9  | Ubiquitous | The system shall put the float over the note pane, and at the center of the screen where the note pane is narrower than the float.                  | Design           |
| FLOAT-10 | State      | While the float is open, the system shall do the keys of write mode in the float, and do nothing for the keys that open a pane or move the focus.   | Design           |
| FLOAT-11 | State      | While the float is open, the system shall do nothing for `alt-n`.                                                                                   | Design           |
| FLOAT-12 | Unwanted   | If the user closes the float and the note has no text, then the system shall make no file.                                                          | Design           |
| FLOAT-13 | Event      | When the user closes the float, the system shall not open the new note in the note pane.                                                            | Design           |
| FLOAT-14 | State      | While a Claude Code session or a terminal is on the full screen, the system shall open the float over it when the user presses `alt-n`, and not send the key to the program. | User, 2026-11-13 |
| FLOAT-15 | Event      | When the user opens a note that is already there, from the notes list or elsewhere, the system shall show it in the note pane as every other file, not in the float. | User, 2026-11-13 |

Tests that verify detailed requirements: `n_opens_the_float` (FLOAT-5),
`the_top_line_shows_the_path_after_the_first_save` (FLOAT-6), `the_float_fits_the_text_column` and
`a_narrow_note_pane_centers_the_float` (FLOAT-8, FLOAT-9),
`pane_keys_do_nothing_in_the_float` (FLOAT-10, FLOAT-11), `an_empty_float_makes_no_file` (FLOAT-12),
`alt_n_opens_the_float_over_a_full_screen_session` in `tests/terminal.rs` (FLOAT-14), `n_opens_the_float`
(FLOAT-15).

