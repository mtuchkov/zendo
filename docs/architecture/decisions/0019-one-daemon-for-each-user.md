---
status: "draft"
date: "2026-11-18"
sources:
  - "The user, 2026-11-18, in chat: sessions shall survive a restart of zendo, and a daemon shall carry the communication, so that more than one zendo app can communicate. The user accepted one daemon for each user, not one for each repository."
  - "The plan of zendo, phase 2 (project document, 2026-11-18): a server for each repository holds the PTYs, the screens and the agent; the UI is a client."
  - "ADR 0018 (messages through a journal of the user)"
---

# 0019 One zendo daemon for each user

## Context

Phase 2 of the plan splits zendo into a server and a UI: the server holds the terminals and the
agents, so they go on when the UI closes. The plan has one server for each repository.

The message board (ADR 0018) has posts about resources of the machine, such as the port 5220 or the
docker container `foo`. A session in another repository uses the same port. The user wants more
than one zendo app to communicate. With one server for each repository, the servers would need a
second channel between them for posts, and two UIs in one repository could still start two sets
of sessions.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------ | ------------------ |
| One server for each repository (the plan today) | Small failure domain; a crash stops one repository. | Posts and messages across repositories need another channel; discovery of many sockets. |
| One daemon for each user, with all repositories | One socket, one journal writer, one set of sessions for each repository, posts across repositories for free. | A crash stops the sessions of every repository; the daemon must isolate the repositories. |

## Decision

Draft of the agent, accepted by the user in chat on 2026-11-18: one daemon for each user.

- The daemon holds the sessions of every repository: the PTYs, the screens, the agents and the
  Claude Code sessions. Each UI connects to it through one socket in `~/.zendo/`, and shows the
  sessions of its repository.
- The daemon is the only writer of the journal of ADR 0018 when it runs. Before phase 2, the zendo
  processes write the journal themselves; the format does not change.
- When two UIs open one repository, they show the same sessions.
- Platforms stay as in the plan: a Unix socket on macOS and Linux, a named pipe on Windows.

## Consequences

- Phase 2 of the plan changes from a server for each repository to one daemon for each user.
- The open point of zendo-session.md, two zendo processes with two sessions 2, ends with phase 2.
- A fault in one repository, such as an agent that hangs, must not stall the daemon for the others:
  each repository needs its own tasks and limits inside the daemon.
- The board does not wait for this decision (ADR 0018).

## Verification

- The exit check of phase 2 runs with two repositories: the user closes both UIs during a turn of
  Claude Code in each, opens them again, and both turns go on.
- A post from repository A shows in a session of repository B through the daemon.

## Decision record

Draft of the agent, 2026-11-18. The user accepted the choice of one daemon for each user in chat on
2026-11-18; the record is not reviewed by the user.
