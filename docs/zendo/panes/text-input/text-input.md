---
status: "implemented"
approved: "2026-11-18"
revision: "4"
date: "2026-11-18"
parent: "../panes.md"
prev: "../keymap.md"
next: "../key-change.md"
children:
  - "text-formatting.md"
  - "date-selector.md"
  - "undo.md"
  - "undo-documents.md"
level: "the kinds of text input and the keys that every input has"
note: "Written from the requests of the user on 2026-11-17 and 2026-11-18; approved by the user on 2026-11-18. The code implements the requirements; the acceptance tests in tests/text_input.rs pass."
---

# Text input

zendo has two kinds of text input. A **rich editor** holds markdown on many lines: the note in
write mode, the new note in the float, and the chat input. A **text field** holds one line: the find
panel, the search of the notes, the file name filter of the tree, and the name or the folder of a
file. See [Panes and layouts](../panes.md) for the panes and [Keys, hints and help](../keymap.md) for
the keymap.

**Every input.** The arrows, `home` and `end`, the word moves, Shift and a move to select, copy, cut
and paste with the system clipboard, and the delete of the word before work in every input. The
text fields take these keys from `[textbox]` of the keymap; each rich editor has the same actions in
its own section, `[write]` or `[chat]`.

A rich editor also formats markdown as the user types: marks, lists, quotes, tasks and code blocks
are in [Formatting](text-formatting.md); `//` and the date selector are in
[Date selector](date-selector.md). Every input has undo and redo: see [Undo and redo](undo.md).

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TEXT-1  | Ubiquitous | The system shall give every one-line text field the same keys: the cursor by chars and by words, to the line start and to the line end, typing and deleting at the cursor. | User, 2026-11-17 |
| TEXT-2  | Ubiquitous | The system shall let the user select text in every input with Shift and a move, and copy, cut and paste it with the system clipboard. | User, 2026-11-17 |

### Acceptance tests

**`the_find_field_is_a_text_box`** (unit test; verifies TEXT-1, TEXT-2)

- Given the find panel with the query `docs zendo`.
- When the user moves a word left, deletes the space, types `/`, selects the line and presses
  `ctrl-x`.
- Then the query is empty and the clipboard holds `docs/zendo`.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TEXT-3  | Event      | When the user pastes into a one-line text field, the system shall put the text in at the cursor and make each new line a space. | Design |
| TEXT-4  | Event      | When the user presses `ctrl-x` with a selection in the chat input, the system shall cut the selection to the clipboard. | User, 2026-11-17 |
| TEXT-11 | Ubiquitous | The system shall take the keys of the text fields from `[textbox]` of the keymap, and show them in the help of the global keys and of the tree. | Design |

Tests that verify detailed requirements: the tests of `src/text/field_tests.rs` (TEXT-1, TEXT-2,
TEXT-3), and in `tests/text_input.rs`: `a_file_name_takes_a_paste_in_one_line` (TEXT-1, TEXT-3),
`ctrl_x_cuts_in_the_chat_input` (TEXT-4), `the_text_box_keys_are_in_the_keymap_and_the_help`
(TEXT-11).
