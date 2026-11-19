---
status: "draft"
revision: "1"
date: "2026-11-18"
parent: "zendo.md"
prev: "board.md"
next: "board-watch.md"
level: "posts of the message board: what a session tells all sessions of the machine about the shared resources that it uses"
note: "Agent draft from the discussion with the user on 2026-11-18 in chat. Sibling of board.md, split by topic. Design: ADR 0018, ADR 0019. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Posts

A session often uses a resource outside its worktree: the docker container `foo`, the port 5220, a
local Kind cluster. A session in another repository can use the same resource. A *post* tells all
sessions of the machine about such a resource, for example "I use the port 5220 for the dev
server". See [Message board](board.md) for the messages between sessions.

**Not interactive.** A post starts no turn and gets no response. zendo does not check two posts
about the same resource against each other: a conflict is for the agents and the user to see, not
for zendo to decide.

**Where the agent sees posts.** zendo adds the live posts that the session did not see yet to the
context of its next question, as it adds other context (CLAUDE-27). The
[watch](board-watch.md) prints a new post during a turn. The tool `posts` returns the live posts and
the history.

**The resource.** Each post names its resource in free text, such as `docker:foo` or `port:5220`.

**Life of a post.** A post is live until one of these comes first:

- the author calls `release` for the resource;
- the thread of the author ends;
- the process that the post names ends, when that process is in the process tree of the session
  ([Background processes](background.md)). A dev server that stops ends its post.

zendo cannot see the life of a container under the docker daemon, so such a post ends by `release`
or with the thread.

**History.** A post that ended goes to the history, and the history keeps it for a fixed time.

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| BOARD-13 | Ubiquitous | The system shall show each live post to every session of the user on the machine, in each repository. | User, 2026-11-18 |
| BOARD-14 | Ubiquitous | The system shall not start a turn and shall not accept a response for a post. | User, 2026-11-18 |
| BOARD-15 | Event      | When the author releases the resource, the thread of the author ends, or the process that the post names ends, the system shall move the post to the history. | User, 2026-11-18; Design |

### Acceptance tests

The tests are in `tests/board.rs`, with the mock agent. Two repositories and the journal are in a
folder of the test.

**`a_post_reaches_another_repository`** (mock agent; verifies BOARD-13, BOARD-14, BOARD-17)

- Given session 2 of repository A and session 1 of repository B, both idle.
- When session 2 of A posts `port:5220`.
- Then no turn starts. The next question of session 1 of B carries the post in its context, and
  the question after it does not.

**`a_post_ends_with_its_process`** (mock agent; verifies BOARD-15)

- Given a post that names a background process of the session.
- When the process ends.
- Then `posts` shows the post in the history, and the next question carries it no more.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| BOARD-16 | Unwanted   | If two live posts name the same resource, then the system shall show both and shall not refuse either. | User, 2026-11-18 |
| BOARD-17 | Event      | When a session gets a question, the system shall add to its context the live posts that the session did not see yet. | Design; CLAUDE-27 |
| BOARD-18 | Unwanted   | If a post names a process that is not in the process tree of the session, then the system shall keep the post live until `release` or the end of the thread. | Design |
| BOARD-19 | Ubiquitous | The tool `posts` shall return the live posts and the history, each with the author, the repository, the resource and the time. | User, 2026-11-18 |

Tests of the detailed requirements: `two_posts_on_one_resource_both_show` (BOARD-16) and
`a_post_without_a_known_process_stays` (BOARD-18) in `tests/board.rs`; unit tests of the tool
`posts` in `src/mcp/` (BOARD-19). BOARD-17 has its test above.

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| How long does the history keep a post? | BOARD-19 | User |
| Should a post in a repository that the user has not opened for a long time still show in other repositories? | BOARD-13 | User |
