---
status: "implemented"
approved: "2026-11-17"
revision: "2"
date: "2026-11-17"
parent: "files.md"
prev: "files-worktree-note.md"
level: "the last document of each agent session: zendo keeps it, with the place in it, and opens it again when the user goes back to the session and at start"
note: "Agent draft from the request of the user on 2026-11-17: remember the last opened document of a session and open it when the user switches back. The user approved FILES-50 to FILES-55 on 2026-11-17. Implemented in zendo 0.76.0. Revision 2: the user answered both open points with yes on 2026-11-17: FILES-54 (the place) and FILES-55 (the start), in zendo 0.77.0."
---

# File tree: the last document of a session

Parent: [File tree](files.md). Before: [Document of the session](files-worktree-note.md).

Each agent session (`F6` to `F10`) has a *last document*: the document that the note pane showed
last while the session was active. zendo keeps it as its path without the worktree, as the open
document ([Document of the session](files-worktree-note.md)).

**Going back.** When the user chooses an agent session, the note pane shows the last document of
that session, in the version of the checkout of the session, at the place where the user left it in
read mode: the same first row on the screen and the same cursor. Where the session has no last
document, or the last document does not exist in its checkout, the open document stays, as FILES-39
says.

**Start.** zendo starts on the last document of the active session, at its place. Where there is none,
zendo starts on the newest note, as before.

**What counts.** The note pane counts as it does for the recent files (FIND-6): a preview in the
tree, the float, and a session on the full screen do not change the last document. A document that
does not exist in the checkout of the session (FILES-40) does not change it either.

**Kept.** zendo keeps the last documents and their places in `.zendo/db.json`, with the active
session (CLAUDE-5), so they stay after zendo quits. zendo writes a place when the user leaves the
session and when zendo quits. A place is in rows of the screen: after a change of the width of the
column it is near the old place, not on it. A rename or a move in zendo changes the last document of each session
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
| FILES-54 | Event      | When the system shows the last document of an agent session again, the system shall show it in read mode with the first row on the screen and the cursor where the user left them in that session. | User, 2026-11-17 |
| FILES-55 | Event      | When zendo starts and the active agent session has a last document in its checkout, the system shall open that document at its place (FILES-54) in place of the newest note. | User, 2026-11-17 |

Tests that verify detailed requirements, in `tests/worktree.rs`:
`a_session_opens_its_last_document_again` (FILES-51),
`the_last_document_of_a_session_stays_after_a_restart` (FILES-52),
`the_last_document_of_a_session_follows_a_rename` (FILES-53),
`a_session_goes_back_to_the_place_in_its_last_document` (FILES-54), and
`zendo_starts_on_the_last_document_of_the_active_session` (FILES-52, FILES-55).
