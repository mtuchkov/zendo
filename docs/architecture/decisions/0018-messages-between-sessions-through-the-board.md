---
status: "draft"
date: "2026-11-18"
sources:
  - "The user, 2026-11-18, in chat: the sessions hand off work and ask each other through zendo and the MCP server `zendo`; channels for one, many and all sessions; questions and tasks with answers and statuses; posts about shared resources with a lease that another session can ask about, and a tombstone when the owner is dead; persistence is good; the board shall work with more than one zendo process."
  - "../../zendo/board.md, board-posts.md, board-leases.md, board-watch.md (BOARD-1 to BOARD-35)"
  - "src/mcp/mod.rs: `zendo mcp --session N` is a stdio process that the agent starts; it has no connection to the zendo process."
  - "../../zendo/cli-messages.md (CLIMSG-2): the hooks of Claude Code append lines to a file, and zendo reads them on each tick."
  - "ADR 0003 (state in the repository), ADR 0009 (no UI stalls), ADR 0016 (process tree), ADR 0019 (one daemon for each user)"
---

# 0018 Messages between sessions go through a journal of the user

## Context

The agent of each zendo session starts its own MCP server `zendo` over stdin and stdout. The server
knows only its session and has no connection to the zendo process, which holds the threads, the
queues and the Claude Code terminals. An MCP server cannot put a user message into the session of
its own client. So a message must go from the server of the sender to a zendo process, and that
process delivers it.

Posts are about resources of the machine, such as ports and containers, so they cross
repositories. More than one zendo process can run, in one repository or in many. The board must
work before the daemon of ADR 0019 exists, and keep its tools when the daemon comes.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------ | ------------------ |
| A journal file of the user that every writer appends to and every zendo process reads | The pattern of the hooks (CLIMSG-2). Works now, over restarts, with many zendo processes. The daemon later takes the same file. | Polling; a lock for the appends; zendo must compact the file. |
| A socket of each zendo process | No polling. | No delivery while zendo is down; many processes need discovery; a named pipe on Windows. |
| Wait for the daemon of ADR 0019 | One writer, no polling. | The board waits for the riskiest phase of the plan. |

## Decision

Draft of the agent, from the discussion with the user on 2026-11-18: the first option, before the
daemon.

- **The journal is the contract.** `~/.zendo/board/journal.jsonl` holds one JSON event on each
  line: a message, a post, a release, an invite, a leave, a close, a choice of the user, and the
  claims `delivered` and `watched`. Each writer appends a line in one write under a file lock.
- **Scope.** A repository is named by the path of its main checkout, so its worktrees share it.
  Inboxes, `all` and named channels belong to a repository; posts belong to the machine.
- **Channels.** A named channel is stored as `<repository>/<owner id>/<name>`. The server answers
  `not found` for a channel that the session does not own or belong to, or that is closed, and does
  so before it writes anything.
- **Exactly once.** A delivery is a pair of a message and a recipient. Before zendo sends a message
  as a question, it appends `delivered` for the pair; before the watch prints it, the watch appends
  `watched`. Each reads the file again after its append, and the first claim of the pair in the file
  wins. The same rule settles two zendo processes in one repository.
- **Limits.** zendo derives from the journal which initiations are in flight, holds the next ones of
  the same sender, and squashes the waiting messages of a recipient into one question.
- **Kinds.** The server checks the kind and `in_reply_to` against the journal, and refuses a
  response to a response.
- **Posts and leases.** A renewal of a lease writes no line, so the journal does not grow with time.
  zendo appends `expired` with the time when the holder of a post dies (the thread ends or ADR 0016
  sees the process end), and `renewed` when the holder lives again, as after a restart. While the
  zendo process of the author is down, the time of the last renewal is its last heartbeat. Another
  session appends `tombstone` for an expired post; the first of `renewed` and `tombstone` after the
  expiry wins, and a tombstoned post is not renewed again. zendo adds
  the unseen and the newly expired posts to the context of the next question.
- **Life.** Each zendo process writes a heartbeat to `~/.zendo/board/zendo-<pid>.json` each second;
  the watch exits with a reason when the heartbeat of its repository stops.
- **Compaction.** When the file is large, the process that holds the lock writes a new file with the
  undelivered messages, the open channels, the live posts and the history within its time, and
  renames it over the old one.
- zendo reads the journal off the UI thread (ADR 0009).

## Consequences

- The board does not need the daemon of the plan; the daemon becomes the only writer later.
- A message arrives within one tick. The watch reads about twice a second.
- Two zendo processes in one repository both have a session 2; the claims stop a double delivery,
  but either process can take the message. ADR 0019 removes this.
- `.zendo/board/` of revision 1 of the draft is not used; the board leaves the repository, unlike
  the state of ADR 0003, because posts cross repositories.
- `resources/mcp-instructions.md` gets the rules of BOARD-12 and BOARD-25.

## Verification

- `tests/board.rs` runs the servers over stdin with the mock agent and the mock command line, with a
  journal in a folder of each test, and checks delivery, channels, kinds, limits, posts and the
  watch.
- A unit test appends from many threads under the lock and checks that no line breaks.
- A check with real Claude: session 3 finds with `sessions` that a branch belongs to session 2, and
  assigns it a task.

## Decision record

Draft of the agent, 2026-11-18. Not reviewed by the user.
