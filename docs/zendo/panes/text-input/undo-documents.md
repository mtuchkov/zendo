---
status: "draft"
revision: "3"
date: "2026-11-18"
parent: "text-input.md"
prev: "undo.md"
level: "undo in documents: the saves in the loglet, .zendo/db.log.json, and the changes of the file from outside zendo"
note: "Written by the agent from the requests of the user on 2026-11-18; revision 2: the change is appended to the loglet, and the unsaved text stays out of it; revision 3 took the documents part of undo.md. Not reviewed by the user."
---

# Undo in documents

A document is the note in write mode or the new note in the float. Its loglet ([Undo and
redo](undo.md)) holds what zendo saved and the changes of its file from outside zendo: the agent,
git, another editor. See [ADR 0014](../../../architecture/decisions/0014-undo-loglets-with-operational-transformation.md).

**Documents.** The loglet of a document holds only what zendo saved: each save is one operation.
Write mode saves after a pause in typing, so one undo takes back the text of one burst of typing.
The text that is not saved yet is not in the loglet. `ctrl-z` and `ctrl-y` save first, and write
the text after the undo or redo to the file at once. zendo keeps the loglet of each document in
`.zendo/db.log.json`, with the file of the last save as its head (Design).
So undo works after the user leaves write mode, opens the document again, or starts zendo again.

**Changes from outside.** The agent, git or another editor can change the file of a document.
zendo looks at the file when the document opens in write mode, when the terminal gets the focus
again, when it sees a change on disk, and before each save. A change is appended to the loglet as
one external operation: the difference between the file of the last save and the file now. The
text that the user did not save yet stays out of the loglet: zendo moves it and the cursor over the
change (operational transformation, OT), and the next save writes it. So a save does not write over
the change. When the next undo takes back an external operation, zendo asks first: `y` undoes it;
`n` keeps it for good and undoes the change of the user before it; `esc` does nothing.

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| UNDO-4 | Ubiquitous | The system shall keep the loglet of each document in `.zendo/db.log.json`, so that undo works after the document opens again and after a restart. | User, 2026-11-18 |
| UNDO-16 | Ubiquitous | The system shall record in the loglet of a document only the text that it saved, one operation for each save, and keep the text that is not saved out of the loglet. | User, 2026-11-18 |
| UNDO-6 | Event      | When the file of a document changes outside zendo, the system shall record the change as one external operation of the loglet when it sees the change. | User, 2026-11-18 |
| UNDO-12 | Event     | When the user presses `ctrl-z` and the next undo takes back an external operation, the system shall ask the user whether to undo the external change before it changes the text. | User, 2026-11-18 |

### Acceptance tests

**`undo_works_after_a_restart`** (mock agent; verifies UNDO-4)

- Given a note where the user typed `added` in write mode and pressed `esc`.
- When zendo starts again, the user opens the note in write mode and presses `ctrl-z`.
- Then the text does not hold `added`, and `.zendo/db.log.json` holds the loglet of the note.

**`the_undo_of_an_external_change_asks_first`** (mock agent; verifies UNDO-6, UNDO-12)

- Given a note in write mode where the user typed `mine` and zendo saved it, and then the agent
  writes `theirs` at the start of the file.
- When the user presses `ctrl-z`, then `y`.
- Then after `ctrl-z` the question shows and the text does not change; after `y` the text does not
  hold `theirs` and holds `mine`.

**`undo_and_redo_in_write_mode`** in [Undo and redo](undo.md) also verifies UNDO-16.

**`undo_keeps_the_change_of_the_agent`** (mock agent; verifies UNDO-14)

- Given the note and the change of the agent of the test above.
- When the user presses `ctrl-z` and `esc`, then `ctrl-z` and `n`.
- Then after `esc` the text does not change; after `n` the text starts with `theirs` and does not
  hold `mine`, and the next `ctrl-z` asks nothing.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| UNDO-11 | Unwanted   | If the loglet of a document does not fit the text of the document, then the system shall start a new loglet for it and keep the text. | Design |
| UNDO-17 | Event      | When the user presses `ctrl-z` or `ctrl-y` in a document, the system shall save the text first, and write the text after the undo or redo to the file at once. | Design |
| UNDO-13 | Event      | When the file changes outside zendo while the editor has text that is not saved, the system shall keep that text out of the loglet, move it and the cursor over the external change with OT, and write it with the next save. | User, 2026-11-18 |
| UNDO-14 | Event      | When the user answers `n` to the question of UNDO-12, the system shall keep the external change, move the earlier operations over it, and undo the change of the user before it; on `esc`, the system shall not change the text. | Design |
| UNDO-15 | Event      | When a document opens in write mode, when the terminal gets the focus in write mode, and before each save, the system shall look at its file for a change from outside. | User, 2026-11-18 |

Tests that verify detailed requirements: the tests of `src/text/loglet_tests.rs`,
`src/text/editor/history_tests.rs` and `src/setup/logs_tests.rs` (UNDO-4, UNDO-6, UNDO-11,
UNDO-13, UNDO-14, UNDO-16), and in `tests/undo.rs`: `undo_and_redo_in_write_mode` (UNDO-17),
`an_external_change_merges_with_unsaved_text` (UNDO-13), `a_change_while_the_note_was_closed` and
`the_focus_brings_the_external_change` (UNDO-15).

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| Should the question name the writer (the agent of a session, git) when zendo knows it? | UNDO-12 | User |
| Is one undo for each autosave the right size, or should zendo save more often in write mode? | UNDO-16 | User |
