---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "file-writes.md"
next: "write-conflicts.md"
level: "a file that a different program changes: the base hash of a write, the watch of the folder, the lock"
note: "Agent draft from the part Other writers of the chapter File writes and external consistency of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Other writers

Sources: the part "Other writers" of the chapter
[File writes and external consistency](../../../book/14-file-writes.md) of the zendo book.

Parent: [File writes](file-writes.md). What zendo does when the editor has unsaved text is in
[Write conflicts](write-conflicts.md).

## The base of a write

A different program can change the file: a different editor, git, the agent. So, when the core
reads a file, the model keeps the hash of the file on the disk: the base. A write command holds the
base. Before the write, the executor reads the hash again. If the hash is not the base, the
executor does not write, and it sends the conflict message.

A short time remains between the check and the write. The check makes a race rare and visible, but
not impossible.

## A change on the disk

A [subscription](../subscriptions.md) to the file sends a message when the file changes. If the
document has no unsaved changes, the model reads the file again.

After its own write, the model sets the base to the new hash. So the event about its own write
changes nothing.

The subscription watches the folder of the file, not the file: a different program can replace the
file with a rename, and only the folder sees this.

An advisory lock prevents a race between two zendo processes. Other programs do not use the lock;
for them, only the check of the hash remains.

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| WRITE-10 | Unwanted | If the hash of the file on the disk is not the base of a write, then the executor shall not write, and shall send the conflict message. | User, 2026-11-20 |
| WRITE-11 | Event | When the file of a document without unsaved changes changes on the disk, the model shall read the file again. | User, 2026-11-20 |

### Acceptance tests

**`a_write_over_a_changed_file_does_not_happen`** (unit test, not written yet; verifies WRITE-10)

- Given a file, a write command with the hash of its content as the base, and a different program
  that changed the file after the read.
- When the executor does the command.
- Then the file has the content of the different program, and the conflict message comes.

**`a_change_from_outside_reloads_a_saved_document`** (scenario test with fake services, not written yet; verifies WRITE-11)

- Given an open note with no unsaved changes.
- When the fake disk sends the event that the file changed, with new content.
- Then the note shows the new content, and the base is the hash of the new content.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| WRITE-13 | Event | When the core reads a file, the model shall keep the hash of the file on the disk as the base, and each write command of that file shall hold the base. | User, 2026-11-20 |
| WRITE-14 | Event | When a write of zendo ends, the model shall set the base to the hash of the written content, so that the event about that write changes nothing. | User, 2026-11-20 |
| WRITE-15 | Ubiquitous | The executor shall read the hash of the file again directly before each write. | User, 2026-11-20 |
| WRITE-16 | Ubiquitous | The subscription to a file shall watch the folder of the file, so that it sees a file that a rename replaced. | User, 2026-11-20 |
| WRITE-17 | Ubiquitous | An advisory lock shall prevent two zendo processes from writing one file at the same time. | User, 2026-11-20 |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| UNDO-15 looks at the file at the open, at the focus of the terminal, and before each save. Does the watch of the folder take the place of the first two? | WRITE-16, UNDO-15 | User |
