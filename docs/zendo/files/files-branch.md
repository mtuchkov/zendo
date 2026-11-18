---
status: "implemented"
revision: "1"
date: "2026-11-17"
parent: "files.md"
prev: "files-session-doc.md"
level: "the name of the git branch in the top line and in the float: which checkout it names in each context"
note: "Agent draft from the request of the user on 2026-11-17: the branch name in the top line depends on the context. Approved by the user on 2026-11-17; implemented in zendo 0.89.0. On 2026-11-18 the user moved the branch to the left end of the top line, where the worktree was (FILES-80)."
---

# File tree: the branch of the context

Parent: [File tree](files.md). Before: [The last document of a session](files-session-doc.md).

The top line shows the name of a git branch (ATTN-7). zendo has more than one checkout: the main
checkout, the worktree of each agent session, `.worktrees/<id>` (SESSIONS-21 in
[Sessions](../sessions.md)), and a notes repo outside the repository (NOTESREP-1). One name for the
whole app does not say which checkout the user looks at. So the branch follows the context:

- **The session.** By default, the top line names the branch of the checkout of the active agent
  session: its worktree, or the main checkout while it has none.
- **The document.** While the note pane has the focus, in read mode or in write mode, the top line
  names the branch of the checkout of the open document. The float names the same branch
  (FLOAT-6). While the user holds `m`, the branch is `main`; while the user holds `u`, it is the
  branch of the main checkout (FILES-45, FILES-47).
- **The file tree.** While the file tree has the focus, the top line names the branch of the
  checkout that its view lists: the worktree of the active session in the two worktree views, else
  the main checkout ([Views](files-views.md)).

zendo reads the branch from the files of git and does not start git, so the name changes without a
restart and the screen does not wait (ADR 0009).

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| FILES-59 | Ubiquitous | Except where FILES-60 or FILES-61 applies, the system shall show in the top line the branch of the checkout of the active agent session: its worktree, or the main checkout while the session has no worktree. | User, 2026-11-17 |
| FILES-60 | State      | While the note pane has the focus in read mode or in write mode, the system shall show in the top line the branch of the checkout of the open document. | User, 2026-11-17 |
| FILES-61 | State      | While the file tree has the focus, the system shall show in the top line the branch of the checkout that the view of the tree lists: the worktree of the active agent session in a worktree view, else the main checkout. | User, 2026-11-17 |

### Acceptance tests

The tests are in `tests/branch.rs`.

**`the_top_line_names_the_branch_of_the_session`** (verifies FILES-59)

- Given a repository on `main` whose worktree `.worktrees/1` is on the branch `agent`.
- When the chat has the focus and the user chooses agent session 1, then agent session 2, which
  has no worktree.
- Then the top line shows `agent`, then `main`.

**`the_top_line_names_the_branch_of_the_document`** (verifies FILES-60)

- Given the same repository, agent session 2 active, and a notes repo on the branch `notes`.
- When the user opens a note of the notes repo and the note pane has the focus.
- Then the top line shows `notes`; when the chat has the focus, it shows `main`.

**`the_top_line_names_the_branch_of_the_view`** (verifies FILES-61)

- Given the same repository and agent session 1 active.
- When the file tree has the focus in the text view, then in the worktree view.
- Then the top line shows `main`, then `agent`.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| FILES-62 | Ubiquitous | The system shall show on the top line of the float the branch of the checkout of the open document. | User, 2026-11-17 |
| FILES-63 | State      | While the user holds `m` in read mode, the system shall show `main` as the branch of the document; while the user holds `u`, the branch of the main checkout. | Design |
| FILES-64 | Unwanted   | If a worktree view has the focus and the active agent session has no worktree, then the system shall show no branch in the top line. | Design |
| FILES-65 | Ubiquitous | The system shall read the branch from the files of git at each frame, without a git process. | Design (ADR 0009) |
| FILES-80 | Ubiquitous | The system shall show the branch at the left end of the top line, before the path of the document, with ` · ` between them; the buttons stay at the right end. | User, 2026-11-18 |

Tests that verify detailed requirements, in `tests/branch.rs`:
`the_float_names_the_branch_of_the_document` (FILES-62),
`holding_m_and_u_names_their_branch` (FILES-63),
`a_worktree_view_without_a_worktree_names_no_branch` (FILES-64),
`the_top_line_names_the_branch_of_the_session` (FILES-65: a new branch shows without a restart), and
`the_branch_stands_left_of_the_path` (FILES-80).
