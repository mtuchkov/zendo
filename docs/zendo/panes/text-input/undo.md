---
status: "draft"
revision: "1"
date: "2026-11-18"
parent: "text-input.md"
prev: "date-selector.md"
level: "undo and redo in every input: the keys, the loglets, where zendo keeps them"
note: "Written by the agent from the request of the user on 2026-11-18; not reviewed by the user."
---

# Undo and redo

Undo and redo are keys of every input of [Text input](text-input.md): the note in write mode, the
new note in the float, the chat input, and each text field.

**Keys.** `ctrl-z` undoes the last change. `ctrl-y` does the change again. The keys are `undo` and
`redo` in `[write]`, `[chat]` and `[textbox]` of the keymap. Undo and redo move the cursor to the
end of the changed text and clear the selection.

**Loglets.** Each input keeps its changes as operations in a loglet: an ordered log of operations
with a head. The operations before the head are done; the operations after the head are undone.
Undo applies the inverse of the operation before the head and moves the head back. Redo applies the
operation after the head and moves the head forward. A new change after an undo trims the tail: the
operations after the head go, and redo has nothing to do.

**Groups.** The chars of one word that the user types one after the other are one operation, with
the space after the word. A run of `backspace` or of `delete` is one operation. A move of the cursor
ends the group. Every other change is an operation of its own: a paste, a cut, a mark, `enter`.

**Documents.** zendo keeps the loglet of each document in `.zendo/db.log.json`, with the text at
its head (Design, see [ADR 0014](../../../architecture/decisions/0014-undo-loglets-with-operational-transformation.md)).
So undo works after the user leaves write mode, opens the document again, or starts zendo again.
The chat input and the text fields keep their loglets in memory only; they go when zendo stops. A
sent question starts a new loglet in the chat input.

**Changes of other writers.** The agent, git or another editor can change a document. When zendo
sees such a change, it transforms the operations of the loglet over the change (operational
transformation). Then undo takes back only the changes of the user, and the change of the other
writer stays.

## Architecturally significant requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| UNDO-1 | Event      | When the user presses `ctrl-z` in an input, the system shall undo the last change of that input; when the user presses `ctrl-y`, the system shall do the last undone change again. | User, 2026-11-18 |
| UNDO-2 | Ubiquitous | The system shall keep the changes of each input as operations in a loglet with a head, and undo and redo shall move the head. | User, 2026-11-18 |
| UNDO-3 | Event      | When the user makes a change other than redo after an undo, the system shall remove the operations after the head of the loglet. | User, 2026-11-18 |
| UNDO-4 | Ubiquitous | The system shall keep the loglet of each document in `.zendo/db.log.json`, so that undo works after the document opens again and after a restart. | User, 2026-11-18 |
| UNDO-5 | Ubiquitous | The system shall keep the loglets of the chat input and of the text fields only in memory. | User, 2026-11-18 |
| UNDO-6 | Event      | When a document changes outside its editor, the system shall transform the operations of its loglet over the change, so that undo and redo keep the change of the other writer. | User, 2026-11-18 |

### Acceptance tests

**`undo_and_redo_in_write_mode`** (mock agent; verifies UNDO-1, UNDO-2)

- Given a note in write mode, the cursor in a new paragraph.
- When the user types `alpha beta`, presses `ctrl-z`, then `ctrl-y`.
- Then the paragraph is `alpha ` after `ctrl-z` and `alpha beta` after `ctrl-y`.

**`a_new_change_after_undo_trims_the_tail`** (mock agent; verifies UNDO-3)

- Given a note in write mode where the user typed `alpha beta` and pressed `ctrl-z`.
- When the user types `x` and presses `ctrl-y`.
- Then the paragraph is `alpha x`, and `beta` does not come back.

**`undo_works_after_a_restart`** (mock agent; verifies UNDO-4)

- Given a note where the user typed `added` in write mode and pressed `esc`.
- When zendo starts again, the user opens the note in write mode and presses `ctrl-z`.
- Then the text does not hold `added`, and `.zendo/db.log.json` holds the loglet of the note.

**`undo_in_the_chat_input_and_a_field`** (mock agent; verifies UNDO-1, UNDO-5)

- Given `hello world` in the chat input, and `zen` in the find panel.
- When the user presses `ctrl-z` and `ctrl-y` in each of them.
- Then each input undoes and redoes its own change, and `.zendo/db.log.json` is not there.

**`undo_keeps_the_change_of_the_agent`** (mock agent; verifies UNDO-6)

- Given a note in write mode where the user typed `mine` at the end and zendo saved it, and then
  the agent writes `theirs` at the start of the file.
- When the user presses `ctrl-z`.
- Then the text starts with `theirs` and does not hold `mine`.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| UNDO-7  | Ubiquitous | The system shall keep a word that the user types, with the space after it, as one operation, and a run of `backspace` or of `delete` as one operation. | Design |
| UNDO-8  | Event      | When undo or redo applies an operation, the system shall put the cursor at the end of the changed text and clear the selection. | Design |
| UNDO-9  | Unwanted   | If the loglet has no operation to undo or to redo, then the system shall not change the text. | Design |
| UNDO-10 | Ubiquitous | The system shall take the keys of undo and redo from `[write]`, `[chat]` and `[textbox]` of the keymap, and show them in the help. | Design |
| UNDO-11 | Unwanted   | If the loglet of a document does not fit the text of the document, then the system shall start a new loglet for it and keep the text. | Design |

Tests that verify detailed requirements: the tests of `src/text/ot_tests.rs`,
`src/text/loglet_tests.rs`, `src/text/editor/history_tests.rs` and `src/setup/logs_tests.rs`
(UNDO-2 to UNDO-9, UNDO-11), and in `tests/undo.rs`: `undo_moves_the_cursor_to_the_change`
(UNDO-8), `undo_keys_are_in_the_keymap` (UNDO-10).

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| Does `ctrl-shift-z` also redo? A terminal without the kitty keyboard protocol sends it as `ctrl-z`. | UNDO-1, UNDO-10 | User |
| Does a pause in typing end a group, as in many editors? | UNDO-7 | User |
