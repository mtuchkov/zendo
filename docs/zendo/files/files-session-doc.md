---
status: "draft"
revision: "1"
date: "2026-11-17"
parent: "files.md"
prev: "files-worktree-note.md"
level: "the last document of each agent session: zendo keeps it and opens it again when the user goes back to the session"
note: "Agent draft from the request of the user on 2026-11-17: remember the last opened document of a session and open it when the user switches back. Not reviewed by the user. FILES-50 to FILES-53 are proposals until the user reviews this document. Implemented in zendo 0.76.0."
---

# File tree: the last document of a session

Parent: [File tree](files.md). Before: [Document of the session](files-worktree-note.md).

Each agent session (`F6` to `F10`) has a *last document*: the document that the note pane showed
last while the session was active. zendo keeps it as its path without the worktree, as the open
document ([Document of the session](files-worktree-note.md)).

**Going back.** When the user chooses an agent session, the note pane shows the last document of
that session, in the version of the checkout of the session. Where the session has no last document,
or the last document does not exist in its checkout, the open document stays, as FILES-39 says.

**What counts.** The note pane counts as it does for the recent files (FIND-6): a preview in the
tree, the float, and a session on the full screen do not change the last document. A document that
does not exist in the checkout of the session (FILES-40) does not change it either.

**Kept.** zendo keeps the last documents in `.zendo/db.json`, with the active session (CLAUDE-5), so
they stay after zendo quits. A rename or a move in zendo changes the last document of each session
that had the file.

## Architecturally significant requirements

| ID       | Pattern | Requirement | Source |
| -------- | ------- | ----------- | ------ |
| FILES-50 | Event   | When the user chooses an agent session, the system shall show in the note pane the last document that the user had open in that session, in the version of the checkout of that session. | User, 2026-11-17 |

### Acceptance tests

The tests are in `tests/worktree.rs`.

**`a_session_opens_its_last_document_again`** (verifies FILES-50, FILES-51)

- Given a repository whose worktrees `.worktrees/1` and `.worktrees/2` both have `a.md` and `b.md`.
- When the user opens `a.md` in agent session 1, `b.md` in session 2, and then chooses session 1,
  session 2, session 3, and session 1 after `.worktrees/1/a.md` is deleted.
- Then the note pane shows `.worktrees/1/a.md`, then `.worktrees/2/b.md`, then `b.md` of the main
  checkout, then `.worktrees/1/b.md`.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ---------------- |
| FILES-51 | Unwanted   | If the chosen agent session has no last document, or its last document does not exist in the checkout of the session, then the system shall keep the open document (FILES-39). | Design |
| FILES-52 | Ubiquitous | The system shall keep the last document of each agent session in the local state of the repository, so that it stays after zendo quits. | Design; CLAUDE-5 |
| FILES-53 | Event      | When the user renames or moves a file in zendo, the system shall change the last document of each agent session that had the file to the new path. | Design; FIND-8 |

Tests that verify detailed requirements, in `tests/worktree.rs`:
`a_session_opens_its_last_document_again` (FILES-51),
`the_last_document_of_a_session_stays_after_a_restart` (FILES-52), and
`the_last_document_of_a_session_follows_a_rename` (FILES-53).

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| Should zendo also open the last document of the active session at start, in place of the newest note? | FILES-50 | User |
| Should the note pane also go back to the place in the document where the user left it? | FILES-50 | User |
