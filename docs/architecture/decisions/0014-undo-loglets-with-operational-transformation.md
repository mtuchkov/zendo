---
status: "draft"
date: "2026-11-18"
sources:
  - "The request of the user, 2026-11-18: \"support OT in every editor to support ctrl+z undo and ctrl+y redo; keep operational transformation state in loglets; keep loglets per document in zendo db.log.json; keep loglets of text editors and chat just in memory; after undo and any new action except redo trim the tail\""
  - "../../zendo/panes/text-input/undo.md (UNDO-1 to UNDO-11)"
  - "ADR 0003 (state in the repository)"
---

# 0014 Undo and redo with loglets of operations

## Context

Every input of zendo needs undo and redo ([Undo and redo](../../zendo/panes/text-input/undo.md)).
The note in write mode is a file that other writers change too: the agent (say it here, to spec),
git, and other editors. An undo must take back the change of the user, not the change of the agent.
The undo of a document must also live after write mode ends and after a restart.

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
- A loglet is the list of operations of one input and a head. After each key, paste and mouse
  event, the editor compares its text with the text at the head and records the difference as one
  operation. Typing of one word and runs of `backspace` or `delete` merge into one operation.
- Undo applies the inverse of the operation before the head; redo applies the operation after the
  head. A new operation after an undo trims the tail.
- A change of another writer is a diff of the text at the head and the new text. zendo transforms
  each operation of the loglet over it: the done operations from the head down, the undone
  operations from the head up. Then the loglet fits the new text.
- zendo keeps the loglets of documents in `.zendo/db.log.json`, apart from `.zendo/db.json`: a
  map from the repository path of the document to its operations, its head and the text at its
  head. The text at the head shows a change on disk while the document was closed; zendo transforms
  the loglet over that change when the document opens. zendo writes the file atomically after each
  save of a document.
- The chat input and the text fields keep their loglets in memory.
- Limits keep the file small: a number of operations for each loglet, and a number of documents,
  the most recent first.

## Consequences

- `.zendo/db.json` stays small and fast to write; `.zendo/db.log.json` grows with the edits.
- The text at the head doubles the text of each document in `.zendo/db.log.json`.
- A loglet that does not fit its text (a broken or old file) goes, and the document gets a new one.
- Undo of a change at two places takes back the text between them as one operation; OT keeps a
  change of another writer in that range when it came after the operation.

## Verification

`undo_works_after_a_restart` (UNDO-4) and `undo_keeps_the_change_of_the_agent` (UNDO-6) in
[Undo and redo](../../zendo/panes/text-input/undo.md), and the unit tests of `src/text/ot.rs` and
`src/text/loglet.rs`.

## Decision record

Proposed by the agent on 2026-11-18 from the request of the user; not reviewed by the user.
