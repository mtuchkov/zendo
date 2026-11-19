---
status: "implemented"
approved: "2026-11-18"
revision: "1"
date: "2026-11-18"
parent: "zendo.md"
prev: "board-posts.md"
next: "board-watch.md"
level: "the lease of a post: its renewal, its age, the restart of zendo, and the tombstone of a post whose owner is dead"
note: "Agent draft from the discussion with the user on 2026-11-18 in chat: a post has a lease, other sessions ask how long it is dead, posts survive a restart, and another session can tombstone a post whose owner is dead. Split from board-posts.md, revision 4, by topic. Design: ADR 0018. The user approved the plan and asked to implement it on 2026-11-18; the code implements it, and the check with real Claude is not done yet."
---

# Leases

A [post](board-posts.md) says that a session uses a shared resource. The session can die and leave
the post behind: its thread ends, its dev server stops, zendo quits. A lease shows whether the
holder of a post is alive, and since when it is not.

**Renewal.** Each post has a lease, and zendo renews it while the holder is alive: the thread of the
author lives, and the process that the post names runs
([Background processes](background.md)). zendo cannot see the life of a container under the docker
daemon, so a post about it renews while the thread of the author lives.

**Expiry.** When the holder dies, zendo stops the renewal, and the lease expires. zendo does not
remove the post: it stays on the board with the time of its last renewal. Another session asks how
long the lease is expired and decides itself, for example that a dev server that died ten minutes
ago no longer holds the port.

**Restart.** A post and its lease survive a restart of zendo, and a restart does not end the thread
of the author. zendo keeps the id and the start time of the named process with the post, so after a
restart it sees whether that process still runs, and renews the lease or not. When the user quits
zendo and stops the background processes (BG-4), their leases expire; when the user leaves them
running, their leases go on after the restart. While no zendo runs, no lease renews, so the time of
the last renewal shows that gap.

**Tombstone.** When the lease of a post is expired, any session of the machine can call `tombstone`
on it: the owner is dead, and the post goes to the history with the tombstone and the session that
set it. While the lease is live, `tombstone` is refused. A tombstone is final: when the holder lives
again, as after a restart, zendo does not renew that post, and the next question of the author tells
it that its post was tombstoned and by whom. The author posts again if it still holds the resource. The session that sets a tombstone decides
alone how old the expired lease must be, and a post has no end time of its own: its lease is the
only clock.

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| BOARD-31 | State      | While the thread of the author lives and the process that the post names runs, the system shall renew the lease of the post. | User, 2026-11-18 |
| BOARD-32 | Event      | When the lease of a post expires, the system shall keep the post on the board, and the tool `posts` shall return how long the lease is expired. | User, 2026-11-18 |
| BOARD-29 | Ubiquitous | The system shall keep each post, its lease and the history over a restart of zendo, and shall not end a post because zendo restarts. | User, 2026-11-18 |
| BOARD-33 | Event      | When a session tombstones a post whose lease is expired, the system shall move the post to the history with the tombstone and the session that set it, and shall not renew it again. | User, 2026-11-18 |

### Acceptance tests

The tests are in `tests/board.rs`, with the mock agent. Two repositories and the journal are in a
folder of the test.

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

**`a_dead_post_can_be_tombstoned`** (mock agent; verifies BOARD-33, BOARD-34, BOARD-35)

- Given a post of session 2 of repository A, and session 1 of repository B.
- When session 1 of B calls `tombstone` while the lease is live, then again after the process of
  the post ends, and then the holder lives again after a restart of zendo.
- Then the first call is refused, the second moves the post to the history with the tombstone, the
  lease is not renewed, and the next question of session 2 of A says who tombstoned its post.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| BOARD-18 | Unwanted   | If a post names a process that is not in the process tree of the session, then the system shall renew its lease while the thread of the author lives. | Design |
| BOARD-30 | Event      | When zendo starts, the system shall check the process of each post by its id and its start time, and renew the lease only if that process runs. | Design; BOARD-29 |
| BOARD-34 | Unwanted   | If a session calls `tombstone` on a post whose lease is live, or the lease is renewed before the tombstone, then the system shall refuse the tombstone. | User, 2026-11-18; Design |
| BOARD-35 | Event      | When the author of a tombstoned post gets its next question, the system shall add to its context the post, the tombstone and the session that set it. | Design |

Tests of the detailed requirements: `a_post_without_a_known_process_renews` (BOARD-18) in
`tests/board.rs`; a unit test of the journal for a renewal and a tombstone at the same time
(BOARD-34). BOARD-30 and BOARD-35 have their tests above.
