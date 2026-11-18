---
status: "draft"
revision: "1"
date: "2026-11-18"
parent: "text-input.md"
prev: "undo.md"
level: "changes of a document from outside zendo in the loglet, and their undo"
note: "Written by the agent from the request of the user on 2026-11-18; not reviewed by the user."
---

# External changes and undo

The loglet of a document ([Undo and redo](undo.md)) also holds the changes of its file from outside
zendo: the agent, git, another editor. See [ADR 0014](../../../architecture/decisions/0014-undo-loglets-with-operational-transformation.md).

**Changes from outside.** The agent, git or another editor can change the file of a document.
zendo looks at the file when the document opens in write mode, when the terminal gets the focus
again, and when it sees a change on disk. A change is one external operation of the loglet: the
whole change of the file, merged with the text that zendo did not save yet (operational
transformation, OT). The trim of a save is no change. When the next undo takes back an external
operation, zendo asks first: `y` undoes it; `n` keeps it for good and undoes the change of the user
before it; `esc` does nothing.

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| UNDO-6 | Event      | When the file of a document changes outside zendo, the system shall record the change as one external operation of the loglet when it sees the change. | User, 2026-11-18 |
| UNDO-12 | Event     | When the user presses `ctrl-z` and the next undo takes back an external operation, the system shall ask the user whether to undo the external change before it changes the text. | User, 2026-11-18 |

### Acceptance tests

**`the_undo_of_an_external_change_asks_first`** (mock agent; verifies UNDO-6, UNDO-12)

- Given a note in write mode where the user typed `mine` and zendo saved it, and then the agent
  writes `theirs` at the start of the file.
- When the user presses `ctrl-z`, then `y`.
- Then after `ctrl-z` the question shows and the text does not change; after `y` the text does not
  hold `theirs` and holds `mine`.

**`undo_keeps_the_change_of_the_agent`** (mock agent; verifies UNDO-14)

- Given the note and the change of the agent of the test above.
- When the user presses `ctrl-z` and `esc`, then `ctrl-z` and `n`.
- Then after `esc` the text does not change; after `n` the text starts with `theirs` and does not
  hold `mine`, and the next `ctrl-z` asks nothing.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| UNDO-13 | Event      | When the file changes outside zendo while the editor has text that is not saved, the system shall merge the external change into that text with OT and move the cursor with it. | User, 2026-11-18 |
| UNDO-14 | Event      | When the user answers `n` to the question of UNDO-12, the system shall keep the external change, move the earlier operations over it, and undo the change of the user before it; on `esc`, the system shall not change the text. | Design |
| UNDO-15 | Event      | When a document opens in write mode, or the terminal gets the focus in write mode, the system shall look at its file for a change from outside. | User, 2026-11-18 |

Tests that verify detailed requirements: the tests of `src/text/loglet_tests.rs` and
`src/text/editor/history_tests.rs` (UNDO-6, UNDO-13, UNDO-14), and in `tests/undo.rs`:
`an_external_change_merges_with_unsaved_text` (UNDO-13), `a_change_while_the_note_was_closed` and
`the_focus_brings_the_external_change` (UNDO-15).

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| Should the question name the writer (the agent of a session, git) when zendo knows it? | UNDO-12 | User |
