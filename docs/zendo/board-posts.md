---
status: "draft"
revision: "3"
date: "2026-11-18"
parent: "zendo.md"
prev: "board.md"
next: "board-watch.md"
level: "posts of the message board: what a session tells all sessions of the machine about the shared resources that it uses, and the lease of a post"
note: "Agent draft from the discussion with the user on 2026-11-18 in chat. Sibling of board.md, split by topic. Design: ADR 0018, ADR 0019. Revision 2: posts survive a restart of zendo (BOARD-29). Revision 3: a post has a lease, and a session can ask how long the lease is expired (User, 2026-11-18); the end of a process no longer moves a post to the history. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Posts

A session often uses a resource outside its worktree: the docker container `foo`, the port 5220, a
local Kind cluster. A session in another repository can use the same resource. A *post* tells all
sessions of the machine about such a resource, for example "I use the port 5220 for the dev
server". See [Message board](board.md) for the messages between sessions.

**Not interactive.** A post starts no turn and gets no response. zendo does not check two posts
about the same resource against each other: a conflict is for the agents and the user to see, not
for zendo to decide.

**The resource.** Each post names its resource in free text, such as `docker:foo` or `port:5220`,
and can name the process that holds it.

**The lease.** Each live post has a lease, and zendo renews it while the holder is alive: the thread
of the author lives, and the process that the post names runs
([Background processes](background.md)). When the holder dies, zendo stops the renewal, and the
lease expires. zendo does not remove the post: it stays on the board with the time of its last
renewal. Another session asks how long the lease is expired and decides itself, for example that a
dev server that died ten minutes ago no longer holds the port. zendo cannot see the life of a
container under the docker daemon, so a post about it renews while the thread of the author lives.

**End.** A post goes to the history when the author calls `release` for the resource, or when the
thread of the author ends. The history keeps it for a fixed time.

**Restart.** A post and its lease survive a restart of zendo, and a restart does not end the thread
of the author. zendo keeps the id and the start time of the named process with the post, so after a
restart it sees whether that process still runs, and renews the lease or not. When the user quits
zendo and stops the background processes (BG-4), their leases expire; when the user leaves them
running, their leases go on after the restart. While no zendo runs, no lease renews, so the time of
the last renewal shows that gap.

**Where the agent sees posts.** zendo adds to the context of the next question each live post that
the session did not see yet, and each post whose lease expired since the session saw it (CLAUDE-27).
The [watch](board-watch.md) prints them during a turn. The tool `posts` returns the posts with their
leases, and the history.

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| BOARD-13 | Ubiquitous | The system shall show each post to every session of the user on the machine, in each repository. | User, 2026-11-18 |
| BOARD-14 | Ubiquitous | The system shall not start a turn and shall not accept a response for a post. | User, 2026-11-18 |
| BOARD-31 | State      | While the thread of the author lives and the process that the post names runs, the system shall renew the lease of the post. | User, 2026-11-18 |
| BOARD-32 | Event      | When the lease of a post expires, the system shall keep the post on the board, and the tool `posts` shall return how long the lease is expired. | User, 2026-11-18 |
| BOARD-29 | Ubiquitous | The system shall keep each post, its lease and the history over a restart of zendo, and shall not end a post because zendo restarts. | User, 2026-11-18 |

### Acceptance tests

The tests are in `tests/board.rs`, with the mock agent. Two repositories and the journal are in a
folder of the test.

**`a_post_reaches_another_repository`** (mock agent; verifies BOARD-13, BOARD-14, BOARD-17)

- Given session 2 of repository A and session 1 of repository B, both idle.
- When session 2 of A posts `port:5220`.
- Then no turn starts. The next question of session 1 of B carries the post in its context, and
  the question after it does not.

**`an_expired_lease_shows_its_age`** (mock agent; verifies BOARD-31, BOARD-32)

- Given a post that names a background process of the session.
- When the process ends, and a minute goes by.
- Then `posts` in another session returns the post with a lease that is expired for about a minute,
  and the next question of that session carries the post one time.

**`a_post_survives_a_restart`** (mock agent; verifies BOARD-29, BOARD-30)

- Given a post without a process, and a post whose process the user leaves running at the quit.
- When zendo quits and starts again.
- Then both posts are on the board with renewed leases, and the lease of the second expires when
  its process ends.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| BOARD-15 | Event      | When the author releases the resource, or the thread of the author ends, the system shall move the post to the history. | User, 2026-11-18 |
| BOARD-16 | Unwanted   | If two posts name the same resource, then the system shall show both and shall not refuse either. | User, 2026-11-18 |
| BOARD-17 | Event      | When a session gets a question, the system shall add to its context each live post that the session did not see yet, and each post whose lease expired since the session saw it. | Design; CLAUDE-27 |
| BOARD-18 | Unwanted   | If a post names a process that is not in the process tree of the session, then the system shall renew its lease while the thread of the author lives. | Design |
| BOARD-19 | Ubiquitous | The tool `posts` shall return each post with the author, the repository, the resource, the time of the post, the time of the last renewal of its lease, and whether the lease is expired, and the history. | User, 2026-11-18 |
| BOARD-30 | Event      | When zendo starts, the system shall check the process of each post by its id and its start time, and renew the lease only if that process runs. | Design; BOARD-29 |

Tests of the detailed requirements: `two_posts_on_one_resource_both_show` (BOARD-16),
`a_post_without_a_known_process_renews` (BOARD-18) and `release_moves_a_post_to_the_history`
(BOARD-15) in `tests/board.rs`; unit tests of the tool `posts` in `src/mcp/` (BOARD-19). BOARD-17
and BOARD-30 have their tests above.

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| How long does the history keep a post? | BOARD-15, BOARD-19 | User |
| Can the author give a post an end time, such as "until 18:00", after which the lease expires? | BOARD-31 | User |
| Should a post with an expired lease leave the board after a time, if its author does not release it? | BOARD-32 | User |
