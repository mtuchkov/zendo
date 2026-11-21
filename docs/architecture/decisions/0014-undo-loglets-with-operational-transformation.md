---
status: "approved"
approved: "2026-11-18"
date: "2026-11-18"
sources:
  - "The request of the user, 2026-11-18: \"support OT in every editor to support ctrl+z undo and ctrl+y redo; keep operational transformation state in loglets; keep loglets per document in zendo db.log.json; keep loglets of text editors and chat just in memory; after undo and any new action except redo trim the tail\""
  - "The request of the user, 2026-11-18: \"keep in mind that file might be changed externally. we might want to add entire file update to loglet when we catch it on next focus on the doc. but when such operation undo we should ask use if he wants to undo external change.\""
  - "The request of the user, 2026-11-18: \"external change is just appended. unsaved changes should not be in the loglet. write mode is autosave on every few ms right? loglet add what has been flushed only.\""
  - "../../zendo/panes/text-input/undo.md (UNDO-1 to UNDO-3, UNDO-5, UNDO-7 to UNDO-10) and ../../zendo/panes/text-input/undo-documents.md (UNDO-4, UNDO-6, UNDO-11 to UNDO-17)"
  - "ADR 0003 (state in the repository)"
---

# 0014 Undo and redo with loglets of operations

## Context

Every input of zendo needs undo and redo ([Undo and redo](../../zendo/panes/text-input/undo.md)).
The note in write mode is a file that other writers change too: the agent (say it here, to spec),
git, and other editors. The user must see what an undo takes back: an undo of the change of the
agent only after a question. The undo of a document must also live after write mode ends and after
a restart.

The editor changes its text in many functions: typing, marks, lists, fences, dates, hard wrap,
paste. A change of each of them to make operations is a large change with a risk of gaps.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Snapshots of the whole text | Simple. | Much memory and disk for a long note; an undo after a change of the agent takes the change of the agent away too. |
| Operations, and a diff of the text after each key | Small records; the editor functions stay as they are; a change of another writer is one more diff. | A diff gives one range per change; a change at two places (a code fence) is one larger operation. |
| Operations from each editor function | Exact operations. | Each function changes; a new function without operations breaks undo. |

## Decision

Accepted: operations from a diff, kept in loglets, with operational transformation (OT).

- An operation is a list of parts: retain `n` chars, insert a text, delete a text. A delete keeps
  its text, so each operation has an inverse without the document.
- A loglet is the list of operations of one input and a head. In the chat input and the text
  fields, after each key, paste and mouse event, the editor compares its text with the text at the
  head and records the difference as one operation; typing of one word and runs of `backspace` or
  `delete` merge into one operation.
- The loglet of a document holds only what zendo flushed to the file: each save is one operation,
  and its head is the file of the last save. The text that is not saved is not in the loglet. Undo
  and redo save first, and write the file at once after the step, so the file is the head again.
- Undo applies the inverse of the operation before the head; redo applies the operation after the
  head. A new operation after an undo trims the tail.
- zendo looks at the file when write mode opens the document, when the terminal gets the focus,
  when the disk check sees a change, and before each save. A file that differs from the head is
  appended as one operation marked external: the difference of the two files. The unsaved text of
  the editor moves over that change with OT, outside the loglet, and the next save writes it.
- Undo of an external operation asks first. `y` undoes it. `n` keeps it: it leaves the loglet, and
  zendo transforms each earlier operation over it, from the head down; then the undo goes on with
  the change of the user before it.
- zendo keeps the loglets of documents in `.zendo/db.log.json`, apart from `.zendo/db.json`: a
  map from the repository path of the document to its operations, its head and the text at its
  head. zendo writes the file atomically after each save of a document.
- The chat input and the text fields keep their loglets in memory.
- Limits keep the file small: a number of operations for each loglet, and a number of documents,
  the most recent first.

## Consequences

- `.zendo/db.json` stays small and fast to write; `.zendo/db.log.json` grows with the edits.
- The text at the head doubles the text of each document in `.zendo/db.log.json`.
- A loglet that does not fit its text (a broken or old file) goes, and the document gets a new one.
- One undo of a document takes back one save: the text of one burst of typing between two pauses.
- Undo of a change at two places takes back the text between them as one operation.
- A change of the agent in write mode no longer waits for the save of the user, and the save of
  the user no longer writes over it: the editor takes it at once, and the save looks at the file
  first.
- Each change from outside that zendo sees is one operation: two quick writes of the agent can be
  two operations, and each asks before its undo.

## Verification

`undo_works_after_a_restart` (UNDO-4),
`the_undo_of_an_external_change_asks_first` (UNDO-6, UNDO-12) and
`undo_keeps_the_change_of_the_agent` (UNDO-14) in
[Undo in documents](../../zendo/panes/text-input/undo-documents.md); the unit tests of
`src/text/ot.rs` and `src/text/loglet.rs`.
