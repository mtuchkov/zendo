---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "../internals.md"
prev: "../frame-and-budget-measurements.md"
next: "../document/document.md"
children:
  - "other-writers.md"
  - "write-conflicts.md"
level: "the write of user data by the runtime: the queue of a path, the write itself, a failure, the saved state, the exit"
note: "Draft from the chapter File writes and external consistency of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document. The write in place is the decision of the user of 2026-11-21 (the book takes priority, ADR 0029); failures.md follows it in its revision 2."
---

# File writes

Sources: the chapter [File writes and external consistency](../../../book/14-file-writes.md) of the
zendo book.

Parent: [Internals of the TUI](../internals.md). Children: [Other writers](other-writers.md), for a
file that a different program changes, and [Write conflicts](write-conflicts.md). The [failure default](../../failures.md) holds for each
write (FAIL-1 to FAIL-3, FAIL-7). The decision for the write in place is in
[ADR 0029](../../../architecture/decisions/0029-user-data-written-in-place-with-a-backup.md).

The write of notes and of the config is the most dangerous IO: an error can lose data of the user.
It has rules on top of the usual [commands](../commands.md).

## One queue for one path

The user edits a note fast, and the core asks to write it three times: A, B, C. Three writes in
parallel can finish in any order, and the disk can have B. So each path has its own queue: one
write runs, and of the waiting requests only the last one stays. A was written, B and C waited:
the runtime writes C and skips B.

Until a write is complete, the status bar shows it.

## How a file is written

1. The executor copies the file to a backup, `name~`.
2. It truncates the file and writes the new content in place. The write is synchronous: when it
   ends, the data is on the disk.
3. After a success, the executor deletes the backup. After a failure, it restores the file from
   the backup.

A write changes only the content of the one file that its command names (FAIL-3). The file keeps
its identity: its links, its owner and its permissions stay, and a watcher of the file sees the
change. The cost: a reader can see a partial file during the write, and the backup
stays on the disk if zendo stops during the write. That backup is not lost: zendo uses it for the
reconcile at the recovery (User, 2026-11-21).

## A failure, the saved state, the exit

If a write fails, the model keeps the data marked as not saved. The text stays in the editor, and
the user can try again.

The saved part of the model has a version. When the version changes, the runtime writes the file
after a small delay, so that ten changes in sequence give one write.

When the user exits, the runtime completes the waiting writes, with a time limit. During this
time, the screen draws and shows "saving…".

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| WRITE-1 | Ubiquitous | The runtime shall run at most one write for one path at one time. | User, 2026-11-20 |
| WRITE-2 | State | While a write of a path runs, the runtime shall keep only the last new request for that path, and shall write it when the running write ends. | User, 2026-11-20 |
| WRITE-3 | Unwanted | If a write fails, then the runtime shall send the message of the failure, the model shall keep the data marked as not saved, and the status bar shall show the failure. | User, 2026-11-20; FAIL-1 |
| WRITE-4 | State | While a write runs or waits, the status bar shall show it. | User, 2026-11-20 |
| WRITE-5 | Ubiquitous | A write shall keep the identity of the file: its links, its owner and its permissions. | User, 2026-11-20 |
| WRITE-6 | Unwanted | If the write of the new content fails, then the executor shall restore the file from its backup. | User, 2026-11-20; FAIL-2 |
| WRITE-7 | Event | When the user exits, the runtime shall complete the waiting writes, with a time limit, and the screen shall draw during that time. | User, 2026-11-20 |

### Acceptance tests

**`writes_to_one_path_run_one_at_a_time`** (scenario test with fake services, not written yet; verifies WRITE-1, WRITE-2)

- Given a fake disk that holds the write A of a note.
- When the core asks for the writes B and C of the same note, and the test lets the disk go.
- Then the disk got A and then C, never B, and never two writes of the note at one time.

**`a_failed_write_keeps_the_text_as_not_saved`** (scenario test with fake services, not written yet; verifies WRITE-3, WRITE-4)

- Given a note in write mode, and a fake disk that holds the write and then fails.
- When the user types a word.
- Then the status bar shows the write while it runs and the failure after it, and the editor has
  the word, marked as not saved.

**`a_write_keeps_the_identity_of_the_file`** (unit test, not written yet; verifies WRITE-5)

- Given a file with a second hard link and the permissions `0600`.
- When zendo writes new content to the file.
- Then the second link shows the new content, the permissions are `0600`, and no backup stays.

**`a_failed_write_restores_the_file`** (unit test, not written yet; verifies WRITE-6)

- Given a file, and a write that stops after half of the new content with an error.
- When zendo writes the file.
- Then the file has the content that it had before, and no backup stays.

**`the_exit_waits_for_the_writes`** (scenario test with fake services, not written yet; verifies WRITE-7)

- Given a fake disk that holds a write, and a second run with a disk that never answers.
- When the user exits.
- Then the screen shows "saving…"; the first run ends after the write, and the second at the time
  limit.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| WRITE-8 | Event | When the executor writes a file, it shall copy the file to the backup `name~`, write the new content in place and to the disk, and then delete the backup. | User, 2026-11-20 |
| WRITE-9 | Ubiquitous | The saved part of the model shall have a version; when the version changes, the runtime shall write the file after a debounce, and no component shall start the save. | User, 2026-11-20 |
| WRITE-23 | Event | When zendo finds the backup `name~` of a write that did not end, the system shall reconcile the file and the backup as the two versions of a conflict (WRITE-18 to WRITE-20). | User, 2026-11-21 |
| WRITE-24 | Ubiquitous | The file tree, find, the git status and the watch of the folder shall not show a backup `name~`, and shall not take it as a change from outside. | User, 2026-11-21 |
