---
status: "implemented"
approved: "2026-11-15"
revision: "2"
date: "2026-11-17"
parent: "files.md"
prev: "files-worktree-recap.md"
next: "files-session-doc.md"
level: "the open document and the agent session: which version of it the note pane shows and the agent gets"
note: "From the request of the user on 2026-11-14: a question in agent session 2 named a file of .worktrees/1. The user approved the requirements in the chat on 2026-11-15, with `m` and `u` for the other versions. Implemented in zendo 0.71.0. FILES-49 (the user, 2026-11-15) came later. Revision 2 (approved by the user on 2026-11-17): FILES-39 yields to the last document of the session (FILES-50, FILES-51)."
---

# File tree: the document of the session

Parent: [File tree](files.md).

Each agent session from session 2 on changes code only in its own worktree, `.worktrees/<id>`
(SESSIONS-21 in [Sessions](../sessions.md)); session 1 works in the main checkout (SESSIONS-23). The note pane shows, and a question names and attaches, the version of
the active agent session: in its worktree, or in the main checkout while it has none.

**The document.** zendo keeps the open document as its path without the worktree:
`.worktrees/1/docs/a.md` and `docs/a.md` are one document in two versions.

**Choosing a session.** The note pane shows the version of the new session. Where the document does
not exist, the pane says "This document does not exist in this session." and a question goes without
a note file; the version shows as soon as it exists, and again when the user goes back. So a new
file of one agent shows only in its session. Choosing a terminal changes nothing. A session that has
a last document opens it ([The last document of a session](files-session-doc.md)).

**Opening a file.** A file opens from any view in the version of the active session, and a new
note is saved in its checkout; a notes repo outside the repository stays as it is.

**Other versions.** In read mode, while the user holds `m`, the note pane shows the version in the
last commit of `main`; while the user holds `u`, the version in the main checkout on disk, with its
changes that are not committed. When the key goes up, the pane shows the version of the session
again. Where the terminal does not report that a key goes
up, such as Terminal.app, the key is up when its auto-repeat stops.

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| FILES-38 | Event      | When a question goes to an agent session, the system shall name and attach only the version of the open document in the checkout of that session: its worktree, or the main checkout while the session has no worktree. | User, 2026-11-14 |
| FILES-39 | Event      | When the user chooses another agent session that has no last document (FILES-51), the system shall show in the note pane the version of the open document in the checkout of that session. | User, 2026-11-14 |

### Acceptance tests

The tests are in `tests/worktree.rs`.

**`a_question_names_the_document_of_its_session`** (mock agent; verifies FILES-38)

- Given a repository whose worktrees `.worktrees/1` and `.worktrees/2` both have `a.md`.
- When the user opens `.worktrees/1/a.md` in agent session 1, chooses session 2, and asks.
- Then the question names `.worktrees/2/a.md` and no other version.

**`choosing_a_session_shows_its_version_of_the_document`** (verifies FILES-39)

- Given the same repository, where each version of `a.md` has its own text.
- When the user opens `a.md` in session 1, then chooses session 2, then session 3, which has no
  worktree.
- Then the note pane shows the text of `.worktrees/1`, then of `.worktrees/2`, then of the main
  checkout.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| FILES-40 | Unwanted   | If the open document does not exist in the checkout of the chosen agent session, then the system shall show an empty note pane that says "This document does not exist in this session.", and send a question without a note file. | User, 2026-11-14 |
| FILES-41 | Event      | When the user goes back to an agent session where the open document exists, the system shall show its version of the document again. | User, 2026-11-14 |
| FILES-42 | Event      | When the user opens a file from any view while an agent session is active, the system shall open the version of the file in the checkout of that session. | User, 2026-11-14 |
| FILES-43 | Ubiquitous | The system shall keep the open document when the user chooses a terminal. | Design |
| FILES-44 | Event      | When the open document comes into being in the checkout of the active session while the note pane says that it does not exist, the system shall show the document. | Design |
| FILES-45 | State      | While the user holds `m` in read mode, the system shall show the version of the open document in the last commit of the branch `main`, and when the user lets go of `m`, the version of the active session. | User, 2026-11-15 |
| FILES-46 | Unwanted   | If the terminal does not report that a key goes up, then the system shall take the end of the auto-repeat of the key as the moment that it goes up. | Design |
| FILES-47 | State      | While the user holds `u` in read mode, the system shall show the version of the open document in the main checkout on disk, and when the user lets go of `u`, the version of the active session. | User, 2026-11-15 |
| FILES-48 | Unwanted   | If the open document does not exist in the version that the user holds `m` or `u` for, then the system shall say so in the note pane. | Design |
| FILES-49 | Event      | When the user saves a new note while an agent session is active, the system shall save it in the notes folder of the checkout of that session, or in the notes repo where the user turned it on (NOTESREP-1). | User, 2026-11-15 |

Tests that verify detailed requirements, in `tests/worktree.rs`:
`choosing_a_session_shows_its_version_of_the_document` (FILES-42),
`a_new_note_is_saved_in_the_checkout_of_the_session` and
`a_new_note_of_a_notes_repo_stays_in_the_notes_repo` (FILES-49),
`a_missing_document_says_so_and_comes_back` (FILES-40, FILES-41, FILES-44),
`choosing_a_terminal_keeps_the_document` (FILES-43), and
`holding_m_and_u_shows_main_and_the_disk_until_the_key_goes_up` (FILES-45 to FILES-48).
