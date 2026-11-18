---
status: "draft"
date: "2026-11-18"
sources:
  - "The user, 2026-11-18: a session gives work to another session through zendo and the MCP server `zendo`, and the sessions ask each other for more information; today the user copies a brief from one session to another."
  - "The user, 2026-11-18: a hand-off goes to one of the five sessions, after a question to the user when the agent names none; all messages go without a confirmation."
  - "../../zendo/board.md (BOARD-1 to BOARD-14)"
  - "src/mcp/mod.rs: `zendo mcp --session N` is a stdio process that the agent starts; it has no connection to the zendo process."
  - "../../zendo/cli-messages.md (CLIMSG-2): the hooks of a Claude Code session append lines to a file in `.zendo/sessions/`, and zendo reads them on each tick."
  - "ADR 0003 (state in the repository), ADR 0009 (no UI stalls)"
---

# 0018 Messages between sessions go through a board in `.zendo/board/`

## Context

The agent of each zendo session starts its own MCP server `zendo` as a child process, over stdin
and stdout. The server knows only the id of its session. It has no connection to the zendo process,
which holds the threads, the chat inputs, the queues and the Claude Code terminals. An MCP server
also cannot put a user message into the session of its own client.

So a message from session 3 to session 2 must go from the server of session 3 to the zendo process,
and zendo must send it to session 2. For a hand-off without a target, the answer of the user must
go back from zendo to the server of session 3, which waits in its tool call.

The plan moves the sessions into a session server later (phase 2). The board must work before that
and move into the server without a change of the tools.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Files in `.zendo/board/`: the servers append lines, zendo reads them on each tick | The pattern of the hooks (CLIMSG-2). Messages survive a restart. Easy to test and to read. No socket on Windows. | One tick of delay. Two zendo processes in one repository read the same files. |
| A socket of the zendo process; the server connects to it | No delay. | A new protocol and a socket path before phase 2; messages are lost when zendo is down; a named pipe on Windows. |
| The agent of the sender writes into the other thread itself | No new part. | The agent has no access to the other thread, and it must not change state of another session. |

## Decision

Draft of the agent: the first option.

- The server of a session shall append each message as one JSON line, in one write, to
  `.zendo/board/messages.jsonl`. A line holds the id of the message, the sender, the target (or
  none), the kind (`handoff` or `message`), the id that it answers (or none), the text and the time.
- zendo shall read the new lines on each tick, off the UI thread (ADR 0009), and keep in
  `.zendo/db.json` the offset of the last line that it delivered.
- zendo shall send a message as a question of the user to the thread of the target. The text for
  the agent is `<zendo_message from="3" id="m12" kind="handoff" reply_to="m9">…</zendo_message>`.
  The chat shows the text without the tag, with the label `from 3 · <name>`.
- zendo shall write the state of the sessions (id, name, state, branch) to
  `.zendo/board/sessions.json` when it changes, with a write to a new file and a rename. The tool
  `sessions` reads this file.
- For a hand-off without a target, the server shall append a `handoff` line without a target and
  read the file until zendo appends a `choice` line with the same id. The `choice` line holds the
  target, or `cancelled`.
- zendo shall count the messages since the last question of the user, keep the count in
  `.zendo/db.json`, and hold the messages over the limit (BOARD-5).
- The checks of a message that need no zendo process (the sender as the target, an id out of 1 to
  5) shall run in the server, so the tool returns the error at once.

## Consequences

- A message takes one tick to arrive. That is fast enough for a hand-off.
- The board works when the user leaves a session and comes back. When zendo is down, no session
  runs (until phase 2), and the messages wait in the file.
- In phase 2 the session server reads the same file, or the servers connect to it; the tools stay
  the same.
- The file grows. zendo can start a new file when all lines are delivered; the size is small.
- Two zendo processes in one repository would deliver the same messages twice. This is an open
  point of board.md, together with the open point of zendo-session.md about the id of a session.
- The instructions of the MCP server (`resources/mcp-instructions.md`) get rules for the new tools
  (BOARD-14).

## Verification

- `tests/board.rs` runs the server of one session over stdin and the mock agent of another, and
  checks the delivery, the queue, the choice of the target, the limit and the restart.
- Unit tests in `src/mcp/` check the lines that the tools write and the errors of BOARD-11.
- A check with real Claude: session 3 finds that a branch belongs to session 2 with `sessions`, and
  gives it a hand-off.

## Decision record

Draft of the agent, 2026-11-18. Not reviewed by the user.
