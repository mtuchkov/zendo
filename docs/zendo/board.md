---
status: "implemented"
approved: "2026-11-18"
revision: "2"
date: "2026-11-18"
parent: "zendo.md"
prev: "zendo-session.md"
next: "board-posts.md"
level: "the message board between zendo sessions: its purpose, channels, the kinds of messages, delivery and limits"
note: "Draft from the discussion with the user on 2026-11-18 in chat. Revision 2 replaces revision 1: channels with owners, questions and tasks with answers and task statuses, squash and a limit per sender, posts in board-posts.md, a journal for each user (ADR 0021, ADR 0022). Open points closed by the agent as Design after the user asked for no more questions: a task goes to one inbox, a fuse holds a chain of turns. The identifiers of revision 1 were proposals that the user did not review, so this revision assigns them again. The user approved the plan and asked to implement it on 2026-11-18; the code implements it, and the check with real Claude is not done yet."
---

# Message board

zendo sessions ([zendo session](zendo-session.md)) work in parallel on one machine and can corrupt
the state of each other: a branch, a worktree, a container, a port. The message board is the
**emergency channel** to resolve such cross-session state. Today the user copies a brief from one
session to another; with the board the agents talk through the MCP server `zendo`, and zendo
delivers. The non-interactive side, posts about shared resources, is in
[Posts](board-posts.md). The agent gets messages during a turn through the [watch](board-watch.md).
Design: [ADR 0021](../architecture/decisions/0021-messages-between-sessions-through-the-board.md),
[ADR 0022](../architecture/decisions/0022-one-daemon-for-each-user.md).

**Channels.** Each message goes to a channel, and each address is a channel:

- `session:<id>`, the inbox of one session of the repository.
- `all`, which every session of the repository listens to.
- A named channel, such as `review-oci`. It belongs to the session that sends to it first, and its
  full name is `<id>/review-oci`, so two sessions never collide. A session joins only when the owner
  invites it: by a message whose `reply_to` names the channel, or with `invite`. Any member can
  `leave`. The channel closes with the thread of its owner.

A session cannot join a channel of another session by itself. For a channel that the session does
not own or belong to, or that is closed, the tools answer `not found`, so they do not show that it
exists.

**Kinds.** An *initiation* is a question or a task. A *response* is an answer to a question, or a
status of a task: `accepted` or `rejected`, then `done` or `failed`. A response names the message
that it answers (`in_reply_to`) and goes to the `reply_to` channel of that message, by default the
inbox of its sender. zendo accepts no response to a response, so "thanks" cannot go.

**Tools.** The MCP server `zendo` has `sessions` (the sessions of the repository with their names,
states and branches), `ask`, `assign`, `answer`, `status`, `invite` and `leave`, and the tools of
posts. `assign` without a target asks the user to choose the session.

**Delivery.** zendo delivers without a confirmation of the user. A message to an inbox goes to the
current thread of that session as a question of the user, and opens the thread if no agent runs. A
message to `all` or to a named channel goes only to the members whose agent runs. While Claude Code
runs for a session, the message goes to Claude Code. While the recipient answers, its messages
wait, and zendo squashes all waiting messages of one recipient into one question.

**Limits.** An initiation is for an emergency only; the instructions of the MCP server say so. Each
sender has one initiation in flight for each recipient, and one for each channel with many members.
It lands when the recipient answers, or when the turn of the recipient that took it ends; for a
channel, when every member has done so. A new initiation of the sender waits and is squashed into
the next delivery. A response is short; `accepted` is a full response. A task goes to one inbox
only, so two sessions never take the same work. As a fuse against a chain of initiations, zendo
holds delivery after a fixed number of turns in a row that the board starts without a question of
the user, and shows `!` on the buttons of the sessions; the next question of the user goes on.

A message from a session does not carry the approval of the user, for example of a merge into
`main` (SESSIONS-21).

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| BOARD-1 | Ubiquitous | The system shall deliver each message to each recipient one time, and keep it until then, also over a restart of zendo and with more than one zendo process. | User, 2026-11-18 |
| BOARD-2 | Event      | When a message comes to the inbox of a session, the system shall send it to the current thread of that session as a question of the user, without a confirmation of the user. | User, 2026-11-18 |
| BOARD-3 | Event      | When a message comes to `all` or to a named channel, the system shall deliver it only to the members of the channel whose agent runs. | User, 2026-11-18 |
| BOARD-4 | Unwanted   | If a session uses a channel that it does not own or belong to, or that is closed, then the tool shall answer `not found`. | User, 2026-11-18 |
| BOARD-5 | Ubiquitous | The system shall accept only a question or a task as an initiation, and only an answer to a question or a status of a task as a response. | User, 2026-11-18 |
| BOARD-6 | State      | While an initiation of a sender to a recipient is in flight, the system shall hold the next initiations of that sender to that recipient, and squash them into one. | User, 2026-11-18 |

### Acceptance tests

The tests are in `tests/board.rs`, with the mock agent and the mock command line of
[Sessions](sessions.md). A test calls `zendo mcp --session N` over its stdin, as `tests/mcp.rs`
does, and keeps the journal in a folder of its own.

**`a_task_reaches_the_inbox_once`** (mock agent; verifies BOARD-1, BOARD-2)

- Given sessions 2 and 3 with threads, and two zendo processes in the repository.
- When session 3 calls `assign` to `session:2`.
- Then session 2 gets the task as one question, labeled `from 3`, one time.

**`all_skips_sessions_without_an_agent`** (mock agent; verifies BOARD-3)

- Given session 2 with a running agent and session 4 without one.
- When session 3 calls `ask` to `all`.
- Then session 2 gets the question, and session 4 stays closed.

**`a_foreign_channel_is_not_found`** (mock agent; verifies BOARD-4)

- Given the channel `review-oci` of session 3, without session 4 in it.
- When session 4 sends to it, invites to it, or leaves it.
- Then each tool answers `not found`.

**`no_response_to_a_response`** (unit test; verifies BOARD-5)

- Given an answer from session 2 to a question of session 3.
- When session 3 calls `answer` on that answer.
- Then the tool returns an error and nothing goes.

**`initiations_wait_and_squash`** (mock agent; verifies BOARD-6, BOARD-9)

- Given a question of session 3 to session 2 in flight.
- When session 3 asks two more questions, and the turn of session 2 ends.
- Then session 2 gets the two questions as one question.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| BOARD-7  | Ubiquitous | The system shall keep the full name of a named channel as `<owner id>/<name>`, and close the channel, with all its members, when the thread of its owner ends. | User, 2026-11-18 |
| BOARD-8  | Event      | When a message names a named channel of its sender in `reply_to`, the system shall add each recipient to that channel. | User, 2026-11-18 |
| BOARD-9  | Event      | When an initiation lands, by a response or by the end of the turn of the recipient that took it, the system shall deliver the held initiations of that sender. | User, 2026-11-18 |
| BOARD-10 | Event      | When `assign` names no target, the system shall ask the user in the chat of the sender to choose a session, and return `cancelled` after `esc`. | User, 2026-11-18 |
| BOARD-11 | Ubiquitous | The system shall show a message in the chat of the recipient with the label `from <id> · <name>`, and give it to the agent in a tag with the sender, the id, the kind, `in_reply_to` and `reply_to`. | Design |
| BOARD-36 | Unwanted   | If a task names `all` or a named channel, then the tool shall refuse it. | Design |
| BOARD-37 | Unwanted   | If the board starts a fixed number of turns in a row without a question of the user, then the system shall hold the next deliveries, show `!` on the buttons of their sessions, and deliver them after the next question of the user. | Design |
| BOARD-12 | Ubiquitous | The instructions of the MCP server shall say that the board is an emergency channel for cross-session state, that a message from a session does not approve what only the user approves, and that `accepted` is a full response. | User, 2026-11-18 |

Tests of the detailed requirements in `tests/board.rs`: `a_channel_closes_with_its_owner` (BOARD-7),
`reply_to_invites` (BOARD-8), `assign_without_a_target_asks_the_user` (BOARD-10) and
`a_message_shows_with_its_sender` (BOARD-11), `a_task_to_a_channel_is_refused` (BOARD-36) and
`the_fuse_holds_a_chain` (BOARD-37); BOARD-9 has its test above; unit tests of the
instructions in `src/mcp/mod_tests.rs` (BOARD-12).
