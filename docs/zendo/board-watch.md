---
status: "implemented"
approved: "2026-11-18"
revision: "2"
date: "2026-11-18"
parent: "zendo.md"
prev: "board-leases.md"
next: "chat.md"
level: "the watch of the message board: a command that prints the messages and posts of a session while its turn runs, for the Monitor tool of Claude Code"
note: "Agent draft from the request of the user on 2026-11-18 in chat: a command that Claude can attach the Monitor tool to. Revision 2 follows revision 2 of board.md: channels, responses, posts. The identifiers of revision 1 were proposals that the user did not review, so this revision assigns them again. The user approved the plan and asked to implement it on 2026-11-18; the code implements it, and the check with real Claude is not done yet."
---

# Watch the board

A message for a session that works waits until its turn ends ([Message board](board.md)). That is
too late when session 2 asked session 3 and waits for the answer, or when a turn runs for a long
time. The watch gives the messages and the posts to the agent during the turn.

**The Monitor of Claude Code.** Claude Code has the tool Monitor: it runs a command in the
background, and each line of the output of the command comes to the session as an event while
Claude works. The watch is a command for that tool.

**The command.** `zendo board watch --session 2` prints, one JSON line each, every message for
session 2 (its inbox and the channels that it belongs to) and every new post
([Posts](board-posts.md)). It first prints what waits, then each new one. It runs until zendo ends
for the repository, or until the Monitor stops it.

**Wait for a response.** `--for m12` prints the responses to the message `m12` and exits after the
first answer, after the last status of a task (`done`, `failed` or `rejected`), or after
`--timeout` seconds. An agent uses it in a foreground command when it needs the response before it
goes on. An agent without Monitor, such as Codex, uses it the same way.

**A watched message is delivered.** When the watch prints a message, zendo does not send it again as
a question. The chat of the session shows it with the label `from 3 · <name> · watched`, in the
order of the transcript.

**The end has a reason.** A Monitor that hears nothing cannot tell a quiet board from a dead watch.
So the watch prints one last line with the reason before it exits: zendo ended, the timeout, or an
error. It also prints a line when a channel that the session belongs to closes, so an agent that
waits there knows that no response comes.

**In the background line.** The watch runs in a shell of the agent, so the background line shows it
(BG-5), as a sign that the session listens to the board.

**How the agent learns the command.** The instructions of the MCP server `zendo` name the command of
the session with the full path of zendo, because zendo is not always on the `PATH`. The result of
each initiation holds its id and the command that waits for its response.

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| BOARD-20 | Ubiquitous | The system shall have the command `zendo board watch --session <id>`, which prints each message for that session and each new post as one JSON line while it runs, first what waits, then each new one. | User, 2026-11-18 |
| BOARD-21 | Event      | When the watch prints a message, the system shall not deliver it again as a question, and shall show it in the chat of the session with the label `from <id> · <name> · watched`. | Design; BOARD-1 |
| BOARD-22 | Unwanted   | If the watch ends for any reason, then it shall print one line with the reason before it exits. | Design |

### Acceptance tests

The tests are in `tests/board.rs`, with the mock agent of [Sessions](sessions.md). The test runs the
watch as a child process and reads its stdout.

**`a_watch_prints_a_message_during_the_turn`** (mock agent; verifies BOARD-20, BOARD-21)

- Given session 2 in a turn, one message for it that waits, and a watch of session 2.
- When the watch starts, and session 3 sends a second message to session 2.
- Then the watch prints both messages in their order, one JSON line each. The chat of session 2
  shows both as `watched`, and after the turn the agent does not get them as questions.

**`a_watch_ends_with_a_reason`** (mock agent; verifies BOARD-22)

- Given a watch of session 2.
- When zendo ends.
- Then the last line of the watch names the reason, and the watch exits.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| BOARD-23 | Event      | When the watch gets `--for <id>`, it shall print only the responses to that message, and exit after the first answer, after the last status of a task, or after `--timeout` seconds with a line that says so. | Design |
| BOARD-24 | Event      | When a channel that the session belongs to closes, the watch shall print a line that names the channel. | User, 2026-11-18 |
| BOARD-25 | Ubiquitous | The instructions of the MCP server shall name the command of the watch of the session with the full path of zendo, and tell Claude Code to attach the Monitor to it while it waits for a response or works on a long turn. | User, 2026-11-18 |
| BOARD-26 | Ubiquitous | The result of `ask` and `assign` shall hold the id of the message and the command that waits for its response. | Design |
| BOARD-27 | Unwanted   | If more than one watch of a session runs, then each watch shall print the message, and the system shall show it in the chat one time. | Design |
| BOARD-28 | Unwanted   | If the turn ends at the same time as the watch prints a message, then the system shall either deliver the message as a question or show it as watched, not both. | Design; BOARD-1 |

Tests of the detailed requirements: `a_watch_for_a_response_ends_after_it` (BOARD-23),
`a_watch_names_a_closed_channel` (BOARD-24) and `two_watches_show_a_message_once` (BOARD-27) in
`tests/board.rs`; unit tests in `src/mcp/mod_tests.rs` for the instructions (BOARD-25) and the
result of the tools (BOARD-26); a unit test of the journal for the race at the end of a turn
(BOARD-28).

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| Does the Claude agent of the chat (ACP) have the Monitor tool, as Claude Code has? If not, the chat uses `--for`. | BOARD-25 | Check with real Claude |
