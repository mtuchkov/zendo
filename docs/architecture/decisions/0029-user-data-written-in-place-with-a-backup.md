---
status: "draft"
date: "2026-11-21"
note: "Agent draft. The decision is of the user (2026-11-21): the zendo book takes priority over the failure default of 2026-11-18. The record, its options and its consequences wait for the review of the user."
sources:
  - "The user, 2026-11-21: the book takes priority; this is a redesign."
  - "../../book/14-file-writes.md (How a file is written)"
  - "../../zendo/internals/file-writes/file-writes.md (WRITE-5, WRITE-6, WRITE-8)"
  - "../../zendo/failures.md (FAIL-2, FAIL-7, and its section Design)"
  - "../../zendo/internals/file-writes/write-conflicts.md (WRITE-19, WRITE-20)"
  - "../../../AGENTS.md (Code: the agent writes user data only with zendo::atomic)"
  - "ADR 0028 (a core that decides and a runtime that does)"
---

# 0029 User data is written in place, with a backup

## Context

Revision 1 of the [failure default](../../zendo/failures.md), approved on 2026-11-18, says: "A
write of user data is complete or does not happen: the file never stays half written." Its design
is one
function, `zendo::atomic`: the new content goes to a hidden temporary file in the folder of the
file, and a rename puts it in place. A file that is a link stays a link, and the file keeps its
permissions. `tests/user_data_writes.rs` checks that each write of user data uses this function.

The zendo book, the target architecture (User, 2026-11-20), describes a different write:

1. The executor copies the file to a backup, `name~`.
2. It truncates the file and writes the new content in place, synchronously.
3. After a success, it deletes the backup. After a failure, it restores the file from the backup.

The book gives the reason: the file keeps its identity. Its links, its owner and its permissions
stay, and a watcher of the file sees the change. It also names the cost: a reader can see a
partial file during the write, and the backup stays on the disk if zendo stops during the write.

The two documents cannot both hold. The decision needed: how zendo writes a file of the user, and
what FAIL-2 promises when zendo stops during a write.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| A. A temporary file and a rename (the code of zendo 0.126) | A reader sees the old file or the new file, never a part. If zendo stops, the file is whole. | The file on the disk is a new file after each write: a second hard link keeps the old content, and a watcher of the file itself loses it. The owner can change. |
| B. A write in place, with a backup `name~` (the book) | The file keeps its identity: hard links, owner, permissions. A watcher of the file sees a change. | A reader can see a partial file. If zendo stops during the write, the file is partial until someone restores `name~`. Each write copies the file one more time. `name~` shows in the folder of the user during the write. |
| C. Option A, and option B only for a file with more than one hard link or with a different owner | The whole file in the usual case; the identity where it matters. | Two paths in the code and in the tests. |

## Decision

The user decided on 2026-11-21: option B, as the book describes it. The book takes priority over
the failure default. The third rule is a proposal of the agent (Design); the book does not state
it.

- The executor shall write user data in place, with a backup, through one function. That function
  shall stay the only way to write user data.
- IF the write of the new content fails, THEN the executor shall restore the file from the backup
  and delete the backup. With this, FAIL-2 holds for a write that ends with an error.
- IF zendo finds a backup `name~` of a write that did not end, THEN zendo shall keep both files and
  shall ask the user to choose a version, as it does for a conflict (WRITE-19, WRITE-20). This is
  an open point of [File writes](../../zendo/internals/file-writes/file-writes.md).

## Consequences

- [Failure default](../../zendo/failures.md) has a revision 2 with this decision, as a draft: FAIL-2
  holds for a write that fails, FAIL-7 keeps the backup when zendo stops during a write, and the
  section "Design" describes option B. The code of zendo 0.126 still implements option A.
- `src/atomic.rs` changes its mechanism. The rule of `AGENTS.md` and the check of
  `tests/user_data_writes.rs` stay, with the name of the function that the user chooses.
- The acceptance test `a_failed_write_keeps_the_file` of FAIL-2 stays valid: after a failed write,
  the file has its old content, and no extra file stays in the folder.
- The watch of the folder (WRITE-16) sees `name~` come and go with each write. The model must not
  take these events as a change from outside.
- The file tree, find and `git status` can show `name~` during a write, and after a stop of zendo.
- `name~` gets a second use: in a conflict with a change from outside, it holds the text of zendo
  beside the version from outside (WRITE-19).
- The check of the hash before the write (WRITE-10) and the queue of the path (WRITE-1) do not
  depend on this decision.

## Verification

- `a_write_keeps_the_identity_of_the_file` (WRITE-5): a second hard link shows the new content,
  and the permissions stay.
- `a_failed_write_restores_the_file` (WRITE-6): a write that stops after half of the content
  leaves the old content and no backup.
- A test for a backup that a stopped zendo left, after the open point has an answer.

None is written yet.

## Decision record

Decided by the user on 2026-11-21 in the chat: "book takes priority". Record drafted by the agent
on 2026-11-21. The record and revision 2 of the failure default wait for the review of the user.
