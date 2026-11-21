---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "file-writes.md"
prev: "other-writers.md"
level: "unsaved text and a change of the file from outside: the silent merge, the conflict with its two versions, and the later reconcile"
note: "Draft from the decisions of the user of 2026-11-21 on conflicts, and from the part Other writers of the chapter File writes and external consistency of the zendo book. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Write conflicts

Sources: the decisions of the user on 2026-11-21; the part "Other writers" of the chapter
[File writes and external consistency](../../../book/14-file-writes.md) of the zendo book.

Parent: [File writes](file-writes.md). The base hash and the watch of the folder are in
[Other writers](other-writers.md). The merge with OT and its undo are in
[Undo in documents](../../panes/text-input/undo-documents.md).

## The silent merge

The file of a document changes on the disk while the editor has unsaved text. OT moves the unsaved
text over the change from outside (UNDO-13). Where OT can do this, zendo is silent: the user gets
no question.

## A conflict

OT cannot prevent all conflicts. When the unsaved text and the change from outside conflict, zendo
loses neither text:

- the file keeps the version from outside;
- `name~` gets the text of zendo;
- zendo handles them as two separate versions and suggests that the user choose one;
- zendo shows which changes of `name~` are lost with the version from outside, and Claude explains
  them.

A conflict that the executor finds before a write ([WRITE-10](other-writers.md)) goes the same way
(Design). So does the backup `name~` that a stopped write left: it is for the reconcile at the
recovery ([WRITE-23](file-writes.md)).

## The later step

The user planned a reconcile for a later step (User, 2026-11-21). It makes the choice of the user
smaller:

- For a file under git, zendo gives the conflict to the conflict resolution of git. How it does
  this is parked (User, 2026-11-21).
- For each other file, zendo counts the change from outside as made at the modified time of the
  file. It splits its own operations into those before and those after that time, applies as many
  as it can, shows the edits that can be lost, and asks the user.
- A three-way merge view shows the versions.

Until that step, each conflict gets the choice between the two versions.

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| WRITE-12 | Unwanted | If the file of a document with unsaved changes changes on the disk, and OT can move the unsaved text over the change, then the system shall merge the two and shall ask no question. | User, 2026-11-21; UNDO-13 |
| WRITE-18 | Unwanted | If the unsaved text of a document and a change of its file on the disk conflict, then the system shall lose neither text, and the user shall decide. | User, 2026-11-21 |

### Acceptance tests

**`a_change_from_outside_merges_with_no_question`** (scenario test with fake services, not written yet; verifies WRITE-12)

- Given a note in write mode with an unsaved word in its last paragraph.
- When the fake disk sends the event that the first paragraph changed.
- Then the editor has both changes, and the screen shows no question.

**`a_conflict_keeps_both_versions`** (scenario test with fake services, not written yet; verifies WRITE-18)

- Given a note in write mode with an unsaved change of its first line.
- When the fake disk sends the event that the same line changed.
- Then the file has the version from outside, `name~` has the text of zendo, and the screen asks
  the user to choose a version and shows the changes of `name~` that can be lost.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| WRITE-19 | Event | When a conflict occurs, the system shall keep the version from outside in the file and the text of zendo in `name~`, as two separate versions, and shall suggest that the user choose one. | User, 2026-11-21 |
| WRITE-20 | Event | When the system asks the user to choose a version, the system shall show the changes of `name~` that are lost with the version from outside, and Claude shall explain them. | User, 2026-11-21 |
| WRITE-21 | Optional | Where the file is under git, the system shall give a conflict to the conflict resolution of git. | User, 2026-11-21 |
| WRITE-22 | Optional | Where the file is not under git, the system shall count the change from outside as made at the modified time of the file, shall split its own operations into those before and those after that time, shall apply as many as it can, and shall show the edits that can be lost before it asks the user. | User, 2026-11-21 |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| WRITE-19 is the first step, and WRITE-21 and WRITE-22 come later. Which milestone document names the steps? | WRITE-19, WRITE-21, WRITE-22 | User |
| `name~` is also the backup of a write (WRITE-8). Can a conflict and a backup need the name at the same time? | WRITE-8, WRITE-19 | User |
| What does the three-way merge view show, and which key opens it? | WRITE-22 | User |
| What does the user see when Claude is off and cannot explain the lost changes? | WRITE-20 | User |
