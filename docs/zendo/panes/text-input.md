---
status: "draft"
revision: "2"
date: "2026-11-17"
parent: "panes.md"
prev: "keymap.md"
level: "how the user types: the rich editors and the one-line text fields, their keys and their formatting"
note: "Agent draft from the requests of the user on 2026-11-17. Revision 2: no alt-c key; `*`, `_`, backtick and `>` typed on a selection mark it; quotes go on; a click checks a task; alt-enter and ctrl-enter send from a code block. Not reviewed by the user. The requirement IDs TEXT-1 to TEXT-14 are proposals until the user reviews this document."
---

# Text input

zendo has two kinds of text input. A **rich editor** holds markdown on many lines: the note in
write mode, the new note in the float, and the chat input. A **text field** holds one line: the find
panel, the search of the notes, the file name filter of the tree, and the name or the folder of a
file. See [Panes and layouts](panes.md) for the panes and [Keys, hints and help](keymap.md) for the
keymap.

**Every input.** The arrows, `home` and `end`, the word moves, Shift and a move to select, copy, cut
and paste with the system clipboard, and the delete of the word before work in every input. The
text fields take these keys from `[textbox]` of the keymap; each rich editor has the same actions in
its own section, `[write]` or `[chat]`.

**Formatting.** In a rich editor, `ctrl-b` and `ctrl-i` make the selection, or the word at the
cursor, bold or italic, and take the marks away when they are there. On a selection, the typed
char `*` makes it bold, `_` italic, `` ` `` code and `>` a quote; the same char again takes the mark
away. A backtick on a selection of more than one line puts a code block around the lines. Without a
selection, these chars are text.

**Lists.** Enter on a list item starts the next item with the same indent and marker: `-`, `*`, `+`,
a task `- [ ]`, or a number with `.` or `)`. A number counts on from the number of the item, also
from a number with more digits; the whitespace after the marker stays as the user typed it. Enter
on an empty item ends the list. A quote goes on with `>` in the same way.

**Tasks.** Read mode and write mode show a task as a box: `☐` open, `☑` done. A click on the box
checks the task, or opens it again, in the file. Enter on a done task starts an open task.

**Code blocks.** Enter at the end of a fence that opens a code block puts the cursor on a new line
and puts the closing fence under it. In a code block, Enter keeps the indent and does not go on with
a list. In the chat, `enter` sends; in a code block, and at the end of its opening fence, `enter`
makes a new line, so the user does not send a half block. `alt-enter` and `ctrl-enter` send from
everywhere; `ctrl-j` and `shift-enter` make a new line.

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TEXT-1  | Ubiquitous | The system shall give every one-line text field the same keys: the cursor by chars and by words, to the line start and to the line end, typing and deleting at the cursor. | User, 2026-11-17 |
| TEXT-2  | Ubiquitous | The system shall let the user select text in every input with Shift and a move, and copy, cut and paste it with the system clipboard. | User, 2026-11-17 |
| TEXT-5  | Event      | When the user presses the bold or italic key in write mode, in the float or in the chat input, the system shall mark the selection, or the word at the cursor, and take the mark away when it is there. | User, 2026-11-17 |
| TEXT-13 | Event      | When the user types `*`, `_`, `` ` `` or `>` on a selection in a rich editor, the system shall mark the selection bold, italic, code or quote, and take the mark away when it is there. | User, 2026-11-17 |
| TEXT-14 | Event      | When the user clicks the box of a task in read mode or in write mode, the system shall check the task, or open it again, in the text. | User, 2026-11-17 |
| TEXT-7  | Event      | When the user presses Enter at the end of a fence that opens a code block that no fence closes, the system shall put the cursor on a new line and a closing fence with the same indent under it. | User, 2026-11-17 |
| TEXT-8  | Event      | When the user presses Enter on a bullet item, the system shall start the next item with the same indent and bullet; on an empty item, the system shall end the list. | User, 2026-11-17 |
| TEXT-9  | Event      | When the user presses Enter on an item that starts with a number of one or more digits, a period and whitespace, the system shall start the next item with the next number, the same indent and the same whitespace. | User, 2026-11-17 |

### Acceptance tests

**`the_find_field_is_a_text_box`** (unit test; verifies TEXT-1, TEXT-2)

- Given the find panel with the query `docs zendo`.
- When the user moves a word left, deletes the space, types `/`, selects the line and presses
  `ctrl-x`.
- Then the query is empty and the clipboard holds `docs/zendo`.

**`the_chat_input_marks_bold_italic_code_and_quote`** (unit test; verifies TEXT-5, TEXT-13)

- Given the chat input with `one two three`.
- When the user presses `ctrl-b` at the last word, types `` ` `` on a selected word, presses
  `ctrl-i` at the first word, and types `>` on the selected line.
- Then the input holds `> *one* two` and the code word.

**`a_click_on_a_task_box_checks_it`** (unit test; verifies TEXT-14)

- Given a note with the task `- [ ] milk`, in read mode.
- When the user clicks the box `☐`, and then clicks it again.
- Then the file holds `- [x] milk`, and then `- [ ] milk` again.

**`write_mode_closes_fences_and_counts_on`** (unit test; verifies TEXT-7, TEXT-8, TEXT-9)

- Given write mode at an empty line.
- When the user types `7. seven`, Enter, `eight`, Enter twice, ```` ```sh ````, Enter and `ls`.
- Then the note ends with `7. seven`, `8. eight`, ```` ```sh ````, `ls` and ```` ``` ````.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TEXT-3  | Event      | When the user pastes into a one-line text field, the system shall put the text in at the cursor and make each new line a space. | Design |
| TEXT-4  | Event      | When the user presses `ctrl-x` with a selection in the chat input, the system shall cut the selection to the clipboard. | User, 2026-11-17 |
| TEXT-6  | Event      | When the user types `` ` `` on a selection of more than one line, the system shall put fences around its lines; when the lines start and end with fences, the system shall take them away. | User, 2026-11-17 |
| TEXT-10 | State      | While the cursor of the chat input is in a code block, or at the end of the fence that opens one, the system shall make a new line when the user presses `enter`, and send when the user presses `alt-enter` or `ctrl-enter`. | User, 2026-11-17 |
| TEXT-12 | Event      | When the user presses Enter on a quote line, the system shall start the next line with `>`; on an empty quote line, the system shall end the quote. | User, 2026-11-17 |
| TEXT-11 | Ubiquitous | The system shall take the keys of the text fields from `[textbox]` of the keymap, and show them in the help of the global keys and of the tree. | Design |

Tests that verify detailed requirements: the tests of `src/text/field_tests.rs` (TEXT-1, TEXT-2,
TEXT-3), `src/text/editor/list_tests.rs` (TEXT-8, TEXT-9, TEXT-12, TEXT-13) and `src/text/editor/fence_tests.rs`
(TEXT-5, TEXT-6, TEXT-7), and in `tests/text_input.rs`: `a_file_name_takes_a_paste_in_one_line`
(TEXT-1, TEXT-3), `ctrl_x_cuts_in_the_chat_input` (TEXT-4), `a_backtick_on_lines_makes_a_code_block`
(TEXT-6), `a_click_on_a_task_box_in_write_mode` (TEXT-14), `the_chat_goes_on_with_lists` and `the_chat_closes_a_fence_and_enter_stays_in_the_block`
(TEXT-8, TEXT-9, TEXT-10), `the_text_box_keys_are_in_the_keymap_and_the_help` (TEXT-11).
