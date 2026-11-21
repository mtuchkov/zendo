---
status: "implemented"
approved: "2026-11-18"
revision: "3"
date: "2026-11-18"
parent: "text-input.md"
prev: "date-selector.md"
next: "undo-documents.md"
level: "undo and redo in every input: the keys, the loglets, where zendo keeps them"
note: "Written from the requests of the user on 2026-11-18; revision 2 moved the changes from outside to their own document; revision 3 moved all of the documents to undo-documents.md. Approved by the user on 2026-11-18. The code implements the requirements; the acceptance tests in tests/undo.rs pass."
---

# Undo and redo

Undo and redo are keys of every input of [Text input](text-input.md): the note in write mode, the
new note in the float, the chat input, and each text field.

**Keys.** `ctrl-z` undoes the last change. `ctrl-y`, and only `ctrl-y`, does the change again
(User, 2026-11-18). The keys are `undo` and
`redo` in `[write]`, `[chat]` and `[textbox]` of the keymap. Undo and redo move the cursor to the
end of the changed text and clear the selection.

**Loglets.** Each input keeps its changes as operations in a loglet: an ordered log of operations
with a head. The operations before the head are done; the operations after the head are undone.
Undo applies the inverse of the operation before the head and moves the head back. Redo applies the
operation after the head and moves the head forward. A new change after an undo trims the tail: the
operations after the head go, and redo has nothing to do.

**Documents.** A document records only what zendo saved, keeps its loglet in `.zendo/db.log.json`,
and takes the changes of its file from outside zendo: see [Undo in documents](undo-documents.md).

**Memory.** The chat input and the text fields keep their loglets in memory only; they go when
zendo stops. A sent question starts a new loglet in the chat input.

**Groups.** In the chat input and in a text field, the chars of one word that the user types one
after the other are one operation, with the space after the word. A run of `backspace` or of
`delete` is one operation. A move of the cursor ends the group. Every other change is an operation
of its own: a paste, a cut, a mark, `enter`.

## Architecturally significant requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| UNDO-1 | Event      | When the user presses `ctrl-z` in an input, the system shall undo the last change of that input; when the user presses `ctrl-y`, the system shall do the last undone change again. | User, 2026-11-18 |
| UNDO-2 | Ubiquitous | The system shall keep the changes of each input as operations in a loglet with a head, and undo and redo shall move the head. | User, 2026-11-18 |
| UNDO-3 | Event      | When the user makes a change other than redo after an undo, the system shall remove the operations after the head of the loglet. | User, 2026-11-18 |
| UNDO-5 | Ubiquitous | The system shall keep the loglets of the chat input and of the text fields only in memory. | User, 2026-11-18 |

### Acceptance tests

**`undo_and_redo_in_write_mode`** (mock agent; verifies UNDO-1, UNDO-2)

- Given a note in write mode where the user typed `alpha` and zendo saved it.
- When the user types ` beta`, presses `ctrl-z`, then `ctrl-y`, then `ctrl-z` twice.
- Then the paragraph and the file are `alpha` after the first `ctrl-z` and `alpha beta` after
  `ctrl-y`; after the last two, the note is as before write mode.

**`a_new_change_after_undo_trims_the_tail`** (mock agent; verifies UNDO-3)

- Given a note in write mode where the user typed `alpha`, zendo saved it, the user typed ` beta`
  and pressed `ctrl-z`.
- When the user types `x` and presses `ctrl-y`.
- Then the paragraph is `alphax`, and ` beta` does not come back.

**`undo_in_the_chat_input_and_a_field`** (mock agent; verifies UNDO-1, UNDO-5)

- Given `hello world` in the chat input, and `zen` in the find panel.
- When the user presses `ctrl-z` and `ctrl-y` in each of them.
- Then each input undoes and redoes its own change, and `.zendo/db.log.json` is not there.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| UNDO-7  | Ubiquitous | In the chat input and in the text fields, the system shall keep a word that the user types, with the space after it, as one operation, and a run of `backspace` or of `delete` as one operation. | Design |
| UNDO-8  | Event      | When undo or redo applies an operation, the system shall put the cursor at the end of the changed text and clear the selection. | Design |
| UNDO-9  | Unwanted   | If the loglet has no operation to undo or to redo, then the system shall not change the text. | Design |
| UNDO-10 | Ubiquitous | The system shall take the keys of undo and redo from `[write]`, `[chat]` and `[textbox]` of the keymap, and show them in the help. | Design |

Tests that verify detailed requirements: the tests of `src/text/ot_tests.rs`,
`src/text/loglet_tests.rs`, `src/text/editor/history_tests.rs` and `src/setup/logs_tests.rs`
(UNDO-2, UNDO-3, UNDO-7 to UNDO-9), and in `tests/undo.rs`: `undo_moves_the_cursor_to_the_change`
(UNDO-8), `undo_keys_are_in_the_keymap` (UNDO-10).

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| Does a pause in typing end a group, as in many editors? | UNDO-7 | User |
