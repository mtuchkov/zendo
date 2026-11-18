---
status: "draft"
revision: "1"
date: "2026-11-18"
parent: "zendo.md"
prev: "zendo-session.md"
next: "board-watch.md"
level: "messages between zendo sessions: the hand-off, the questions back, the list of sessions, and the limits"
note: "Agent draft from the request of the user on 2026-11-18 in chat: a session gives work to another session through zendo and the MCP server `zendo`, and the sessions ask each other for more information, so that the user does not copy a brief from one session to another. The user decided: a hand-off goes to one of the five sessions, after a question to the user when the agent names none; all messages go without a confirmation. Design: ADR 0018. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Message board

A zendo session ([zendo session](zendo-session.md)) often finds work that belongs to another
session, for example a finding of a review on the branch of session 2. Today the user copies the
brief from one session to the other. With the message board, the agents send messages to each other
through the MCP server `zendo`, and zendo delivers each one as a question of the user. Design:
[ADR 0018](../architecture/decisions/0018-messages-between-sessions-through-the-board.md).

**Tools.** `sessions` lists the five sessions with the branch of each worktree, so the agent finds
the owner of a branch itself. `handoff` gives a brief to another session; without a target, zendo
asks the user to choose one. `send` sends a short message, such as a question about a hand-off or
its answer.

**Delivery.** A message goes to the current thread of the target without a confirmation: to the
agent of the chat, or to Claude Code while it runs. While the target answers, the message waits in
its queue, or goes to the agent during the turn through the [watch](board-watch.md). The chat of
the target shows it with the label `from 3 · <name>`.

**Not the voice of the user.** A message from a session does not approve what only the user
approves, such as a merge into `main` (SESSIONS-21).

**Limit.** Two agents can answer each other without end, so zendo holds messages after a number of
them without a question of the user.

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| BOARD-1 | Ubiquitous | The system shall give the MCP server `zendo` of each session the tools `sessions`, `handoff` and `send`, which work for Claude Code and for the agent of the chat. | User, 2026-11-18 |
| BOARD-2 | Event      | When a session sends a message to another session, the system shall send it to the current thread of the target as a question of the user, without a confirmation of the user. | User, 2026-11-18 |
| BOARD-3 | State      | While the target session answers, the system shall keep the message in the queue of its chat, and send it when the turn ends. | Design |
| BOARD-4 | Event      | When `handoff` names no target, the system shall ask the user in the chat of the sender to choose one of the other sessions, and send the hand-off to the session that the user chooses. | User, 2026-11-18 |
| BOARD-5 | Unwanted   | If 20 messages go between sessions without a question of the user, then the system shall hold the next messages, show `!` and their number on the button of each target, and deliver them when the user asks a question in any session. | Design |
| BOARD-6 | Ubiquitous | The system shall keep each message in the repository until it is delivered, also over a restart of zendo. | Design; ADR 0003 |

### Acceptance tests

The tests are in `tests/board.rs`, with the mock agent and the mock command line of
[Sessions](sessions.md). The test calls `zendo mcp --session 3` over its stdin, as `tests/mcp.rs`
does.

**`a_handoff_reaches_the_other_session`** (mock agent; verifies BOARD-1, BOARD-2, BOARD-7)

- Given sessions 2 and 3 with threads, both idle.
- When session 3 calls `handoff` with the target 2 and a brief.
- Then the chat of session 2 shows the brief with `from 3`, and its agent gets it in the tag.

**`a_message_waits_while_the_session_answers`** (mock agent; verifies BOARD-3)

- Given session 2 in a turn.
- When session 3 calls `send` to session 2.
- Then the message shows as queued and goes to the agent after the turn.

**`a_handoff_without_a_target_asks_the_user`** (mock agent; verifies BOARD-4, BOARD-12)

- Given session 3 with a thread.
- When session 3 calls `handoff` without a target, and the user chooses session 4.
- Then the tool returns 4 and session 4 gets the brief; after `esc` the tool returns that the user
  did not choose.

**`a_message_reaches_claude_code`** (mock command line; verifies BOARD-8)

- Given a running Claude Code session for session 2.
- When session 3 calls `send` to session 2.
- Then Claude Code gets the message as a prompt.

**`messages_stop_at_the_limit`** (mock agent; verifies BOARD-5)

- Given two sessions that answer each message with a message.
- When the limit is reached.
- Then the targets show `!`, and the next question of the user delivers the held messages.

**`a_message_survives_a_restart`** (mock agent; verifies BOARD-6)

- Given a message that zendo did not deliver, and zendo quits.
- When zendo starts again.
- Then the target gets the message one time.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| BOARD-7  | Ubiquitous | The system shall show a message from a session in the chat of the target with the label `from <id> · <name>`, and give it to the agent in a tag that names the sender, the id of the message and its kind. | Design |
| BOARD-8  | State      | While a Claude Code session runs for the target, the system shall send the message to Claude Code. | SESSIONS; Design |
| BOARD-9  | Event      | When a message goes to a thread that has no running agent, the system shall open the thread and then send the message. | User, 2026-11-18 |
| BOARD-10 | Ubiquitous | The tool `sessions` shall return for each session its id, its name, its state, and the branch of its worktree. | Design |
| BOARD-11 | Unwanted   | If a message names the sender as the target, or a session that does not exist, then the tool shall return an error and the system shall not deliver the message. | Design |
| BOARD-12 | Event      | When the user cancels the choice of the target with `esc`, the system shall not deliver the hand-off, and the tool shall return that the user did not choose. | Design |
| BOARD-13 | Event      | When a message comes while the user is in another session, the system shall blink the button of the target, as at the end of a turn. | Design; Attention |
| BOARD-14 | Ubiquitous | The instructions of the MCP server shall tell the agent to use `handoff` for work that belongs to another session, and that a message from a session does not carry the approval of the user. | User, 2026-11-18; Design |

Tests of the detailed requirements: `a_message_opens_a_thread` (BOARD-9) and
`a_message_blinks_the_button` (BOARD-13) in `tests/board.rs`; unit tests in `src/mcp/mod_tests.rs`
for `sessions` (BOARD-10), the errors (BOARD-11) and the instructions (BOARD-14).

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| Should a hand-off be able to start a new thread in the target session, for a clean context, or does it always go to the current thread? | BOARD-2, BOARD-9 | User |
| Two zendo processes in one repository share `.zendo/board/` and both have a session 2. Does the board belong to one zendo process, or wait for the session server of the plan? | BOARD-6 | User |
| How long does `handoff` wait for the choice of the user before it returns? The MCP client can end a long tool call. | BOARD-4 | Check with real Claude and Codex |
| Is 20 the right limit, and does the count also stop a chain of `handoff` calls? | BOARD-5 | User |
