---
status: "draft"
revision: "1"
date: "2026-11-18"
parent: "zendo.md"
prev: "board.md"
next: "chat.md"
level: "the watch of the message board: a command that prints the messages of a session while its turn runs, for the Monitor tool of Claude Code"
note: "Agent draft from the request of the user on 2026-11-18 in chat: a command that Claude can attach the Monitor tool to. Sibling of board.md, split by topic to keep each document within two pages. Design: ADR 0018. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Watch the board

A message for a session that works waits in its queue until the turn ends (BOARD-3). That is too
late when session 2 asked session 3 a question and waits for the answer, or when a turn runs for a
long time. The watch gives the messages to the agent during the turn.

**The Monitor of Claude Code.** Claude Code has the tool Monitor: it runs a command in the
background, and each line of the output of the command comes to the session as an event while
Claude works. The watch is a command for that tool.

**The command.** `zendo board watch --session 2` prints each message for session 2 as one JSON line:
the sender, the id of the message, the kind, the id that it answers, and the text. It first prints
the messages that wait for the session, then each new one. It runs until zendo quits for the
repository, or until the Monitor stops it.

**Wait for one answer.** `--reply-to m12` prints the answer to the message `m12` and exits. An
agent uses it in a foreground command when it needs one answer before it goes on, and an agent
without Monitor, such as Codex, uses it the same way. `--timeout <seconds>` ends the wait.

**A watched message is delivered.** When the watch prints a message, zendo does not send it again as
a question at the end of the turn. The chat of the target shows it with the label
`from 3 · <name> · watched`, in the order of the transcript.

**The end has a reason.** A Monitor that hears nothing cannot tell a quiet board from a dead watch.
So the watch prints one last line with the reason before it exits: zendo quit, the timeout, or an
error.

**How the agent learns the command.** The instructions of the MCP server `zendo` name the command of
the session with the full path of zendo, because zendo is not always on the `PATH`. The result of
`send` and `handoff` holds the id of the message and the command that waits for its answer.

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| BOARD-15 | Ubiquitous | The system shall have the command `zendo board watch --session <id>`, which prints each message for that session as one JSON line while it runs, first the messages that wait, then each new one. | User, 2026-11-18 |
| BOARD-16 | Event      | When the watch prints a message, the system shall not deliver it again as a question, and shall show it in the chat of the target with the label `from <id> · <name> · watched`. | Design |
| BOARD-17 | Unwanted   | If the watch ends for any reason, then it shall print one line with the reason before it exits. | Design |

### Acceptance tests

The tests are in `tests/board.rs`, with the mock agent of [Sessions](sessions.md). The test runs the
watch as a child process and reads its stdout.

**`a_watch_prints_a_message_during_the_turn`** (mock agent; verifies BOARD-15, BOARD-16)

- Given session 2 in a turn, one message for it in the queue, and a watch of session 2.
- When the watch starts, and session 3 sends a second message to session 2.
- Then the watch prints both messages in their order, one JSON line each. The chat of session 2
  shows both as `watched`, and after the turn the agent does not get them as questions.

**`a_watch_ends_with_a_reason`** (mock agent; verifies BOARD-17)

- Given a watch of session 2.
- When zendo quits.
- Then the last line of the watch names the reason, and the watch exits.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| BOARD-18 | Event      | When the watch gets `--reply-to <id>`, it shall print only the answer to that message and exit, or exit after `--timeout` seconds with a line that says so. | Design |
| BOARD-19 | Ubiquitous | The instructions of the MCP server shall name the command of the watch of the session with the full path of zendo, and tell Claude Code to attach the Monitor to it while it waits for an answer or works on a long turn. | User, 2026-11-18; Design |
| BOARD-20 | Ubiquitous | The result of `send` and `handoff` shall hold the id of the message and the command that waits for its answer. | Design |
| BOARD-21 | Unwanted   | If more than one watch of a session runs, then each watch shall print the message, and the system shall show it in the chat one time. | Design |
| BOARD-22 | Unwanted   | If the turn ends at the same time as the watch prints a message, then the system shall either deliver the message as a question or show it as watched, not both. | Design |

Tests of the detailed requirements: `a_watch_for_a_reply_ends_after_the_reply` (BOARD-18) and
`two_watches_show_a_message_once` (BOARD-21) in `tests/board.rs`; unit tests in
`src/mcp/mod_tests.rs` for the instructions (BOARD-19) and the result of the tools (BOARD-20); a
unit test of the board file for the race at the end of the turn (BOARD-22).

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| The watch runs in a shell of the agent, so the background line shows it as a background process (BG-5). Should it show, as a sign that the session listens, or should zendo leave it out? | BOARD-15; BG-5 | User |
| Does the Claude agent of the chat (ACP) have the Monitor tool, as Claude Code has? If not, the chat uses `--reply-to`. | BOARD-19 | Check with real Claude |
