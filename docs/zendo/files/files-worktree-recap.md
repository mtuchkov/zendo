---
status: "implemented"
approved: "2026-11-14"
revision: "1"
date: "2026-11-14"
parent: "files.md"
prev: "files-worktree.md"
next: "files-worktree-note.md"
level: "the recap of the changes of a worktree at the bottom of the worktree views, and when it is written again"
note: "Agent draft from the requests of the user on 2026-11-14, split from files-worktree.md. Approved by the user on 2026-11-14. Implemented in zendo 0.62.0. Depends on Work (../work.md, draft)."
---

# File tree: worktree recap

Parent: [File tree](files.md). The views that show the recap are in
[Worktree views](files-worktree.md).

**The recap.** In both views, the bottom of the tree shows a recap of the changes of the worktree,
in place of the `about` and the rules (FILES-8). It is one recap of the whole diff of the worktree,
against the same merge base as the worktree changed view (FILES-27), not a recap of the selected
file. The recap says what the changes of the worktree do. zendo looks every 10 minutes and writes the recap again only when the changes differ from
those of the last recap. A recap after each turn of the agent would cost too much (User,
2026-11-14). As for every recap, Claude writes it in a
separate session without tools and it does not go into the chat (WORK-3 in [Work](../work.md));
zendo sends the changes with the request. The idle rule of WORK-2 does not apply, because the
worktree changes while the agent works.

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| FILES-28 | Ubiquitous | The system shall show one recap of the whole diff of the worktree against the merge base of FILES-27 at the bottom of both worktree views, in place of the `about` and the rules of the selected item. | User, 2026-11-14 |
| FILES-29 | State      | While a worktree has changes that differ from those of its last recap, the system shall write the recap of the worktree again at most every 10 minutes, and show it without a restart. | User, 2026-11-14 |

### Acceptance tests

The test is in `tests/worktree.rs`.

**`the_worktree_recap_refreshes_every_10_minutes_when_it_changed`** (mock agent, with a test
clock; verifies FILES-28, FILES-29)

- Given a worktree with one changed file, its recap, and the worktree changed view.
- When the agent changes another file, and the clock goes 5 minutes, then 10 minutes, then 10
  more minutes without a new change.
- Then zendo asks for no recap at 5 minutes, the bottom of the tree shows a new recap at 10
  minutes, and zendo asks for no recap at 20 minutes. The chat has no recap.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| FILES-36 | Ubiquitous | The system shall write the recap of the worktree in a separate Claude session without tools, from the changes that zendo sends, and shall not show it in the chat. | WORK-3 |

Tests that verify detailed requirements: `the_worktree_recap_refreshes_every_10_minutes_when_it_changed`
(FILES-36).
