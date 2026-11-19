---
status: "draft"
revision: "4"
date: "2026-11-18"
parent: "zendo.md"
prev: "board.md"
next: "board-leases.md"
level: "posts of the message board: what a session tells all sessions of the machine about the shared resources that it uses, and where the agents see posts"
note: "Agent draft from the discussion with the user on 2026-11-18 in chat. Sibling of board.md, split by topic. Design: ADR 0018, ADR 0019. Revision 3: a post has a lease (User, 2026-11-18). Revision 4: the lease, the restart and the tombstone moved to board-leases.md, to keep this document within two pages. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Posts

A session often uses a resource outside its worktree: the docker container `foo`, the port 5220, a
local Kind cluster. A session in another repository can use the same resource. A *post* tells all
sessions of the machine about such a resource, for example "I use the port 5220 for the dev
server". See [Message board](board.md) for the messages between sessions, and
[Leases](board-leases.md) for how long a post holds.

**Not interactive.** A post starts no turn and gets no response. zendo does not check two posts
about the same resource against each other: a conflict is for the agents and the user to see, not
for zendo to decide.

**The resource.** Each post names its resource in free text, such as `docker:foo` or `port:5220`,
and can name the process that holds it.

**End.** A post goes to the history when the author calls `release` for the resource, when the
thread of the author ends, or when another session tombstones it ([Leases](board-leases.md)). The
history keeps it for a fixed time.

**Where the agent sees posts.** zendo adds to the context of the next question each post that the
session did not see yet, and each post whose lease expired since the session saw it (CLAUDE-27).
The [watch](board-watch.md) prints them during a turn. The tool `posts` returns the posts with their
leases, and the history.

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| BOARD-13 | Ubiquitous | The system shall show each post to every session of the user on the machine, in each repository. | User, 2026-11-18 |
| BOARD-14 | Ubiquitous | The system shall not start a turn and shall not accept a response for a post. | User, 2026-11-18 |

### Acceptance tests

The tests are in `tests/board.rs`, with the mock agent. Two repositories and the journal are in a
folder of the test.

**`a_post_reaches_another_repository`** (mock agent; verifies BOARD-13, BOARD-14, BOARD-17)

- Given session 2 of repository A and session 1 of repository B, both idle.
- When session 2 of A posts `port:5220`.
- Then no turn starts. The next question of session 1 of B carries the post in its context, and
  the question after it does not.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| BOARD-15 | Event      | When the author releases the resource, or the thread of the author ends, the system shall move the post to the history. | User, 2026-11-18 |
| BOARD-16 | Unwanted   | If two posts name the same resource, then the system shall show both and shall not refuse either. | User, 2026-11-18 |
| BOARD-17 | Event      | When a session gets a question, the system shall add to its context each post that the session did not see yet, and each post whose lease expired since the session saw it. | Design; CLAUDE-27 |
| BOARD-19 | Ubiquitous | The tool `posts` shall return each post with the author, the repository, the resource, the time of the post, the time of the last renewal of its lease, and whether the lease is expired, and the history. | User, 2026-11-18 |

Tests of the detailed requirements: `release_moves_a_post_to_the_history` (BOARD-15) and
`two_posts_on_one_resource_both_show` (BOARD-16) in `tests/board.rs`; unit tests of the tool
`posts` in `src/mcp/` (BOARD-19). BOARD-17 has its test above.
