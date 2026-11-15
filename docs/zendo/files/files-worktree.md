---
status: "draft"
revision: "1"
date: "2026-11-14"
parent: "files.md"
prev: "files-code.md"
next: "files-worktree-recap.md"
level: "the two views of the worktree of the agent session: what they list, how they follow the session, what opening a file does"
note: "Agent draft from the requests of the user on 2026-11-14. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document. The recap is in files-worktree-recap.md (split by the user, 2026-11-14). Depends on Sessions (../sessions.md, draft)."
---

# File tree: worktree views

Parent: [File tree](files.md).

Each agent session of zendo changes code only in its own worktree, `.worktrees/<id>`
(SESSIONS-21 in [Sessions](../sessions.md)). The other views of the tree show the main checkout,
so the user does not see what the agent session in the chat does. Two more views show it:

- **worktree**: every text file of the worktree of the active agent session, as the text view
  shows the main checkout;
- **worktree changed**: only the files of that worktree that differ from `main`.

At the bottom, both views show a recap of the changes of the worktree; see
[Worktree recap](files-worktree-recap.md).

`v` shows the two views after the folders view. Both views follow the chat: when the user chooses
another agent session, the tree shows the worktree of that session. A session has no worktree until
its agent changes code the first time; then `v` skips the two views.

**What "changed" means.** A file is changed when it differs from the merge base of the branch of
the worktree and `main`: a changed file, a new file (also one that git does not track yet), or a
deleted file. The merge base is the commit of `main` that the branch started from, or the last
commit of `main` that was merged into the branch. The agent commits in its worktree, so a comparison with the `HEAD` of the worktree
would show an empty list after each commit (Design). A deleted file shows as a struck row and does
not open.

**Opening a file.** `enter` opens the file of the worktree in the note pane, with read, write, delta
and Claude (FILES-3). The same path can exist in the main checkout, so the head of the note names
the worktree.

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| FILES-26 | State      | While the active agent session has a worktree, the system shall have a worktree view that lists every text file of that worktree. | User, 2026-11-14 |
| FILES-27 | State      | While the active agent session has a worktree, the system shall have a worktree changed view that lists only the files of that worktree that differ from the merge base of its branch and `main`: changed, new and deleted files. | User, 2026-11-14; Design (the merge base) |

### Acceptance tests

Neither test is written yet.

**`the_worktree_view_lists_the_files_of_the_session_worktree`** (unit test; verifies FILES-26)

- Given a test repository with `a.md`, and a worktree `.worktrees/1` that also has `src/new.rs`.
- When agent session 1 is active and the user presses `v` until the worktree view shows.
- Then the tree lists `a.md` and `src/new.rs` of `.worktrees/1`, and no file of the main checkout.

**`the_changed_view_lists_what_differs_from_main`** (unit test; verifies FILES-27)

- Given a worktree whose branch has one commit that changes `a.md`, one new file that git does not
  track, one deleted file, and one file that did not change.
- When the worktree changed view shows.
- Then the view lists the changed file, the new file and the deleted file, and not the fourth file.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| FILES-30 | Event      | When the user presses `v` in the folders view, the system shall show the worktree view, then the worktree changed view, then the structure view. | Design |
| FILES-31 | Unwanted   | If the active agent session has no worktree, then the system shall skip both worktree views on `v`, and shall show one line that says so in a worktree view that is open. | Design |
| FILES-32 | Event      | When the user chooses another agent session, the system shall show the worktree of that session in both worktree views. | User, 2026-11-14 |
| FILES-33 | Event      | When a file or the branch of the worktree changes, the system shall show the change in both worktree views without a restart. | Design |
| FILES-34 | Event      | When the user opens a file from a worktree view, the system shall open the file of the worktree like a note, and name the worktree in the head of the note. | Design |
| FILES-35 | Ubiquitous | The system shall show a deleted file in the worktree changed view as a struck row that does not open. | Design |
| FILES-37 | State      | While a worktree view shows, the system shall not rename, move, delete or make files from the tree. | Design |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| FILES-4 (approved) names five views in the `v` cycle. Does FILES-30 replace its cycle, or does FILES-4 get a new revision? | FILES-4, FILES-30 | User |
| May the user rename, move, delete and make files in the worktree of an agent that works (this draft: no)? | FILES-37 | User |
