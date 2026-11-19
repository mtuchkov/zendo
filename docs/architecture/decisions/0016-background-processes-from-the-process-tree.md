---
status: "approved"
approved: "2026-11-18"
note: "ADR 0023 (draft, 2026-11-18) changes it in part: no `ps` fallback and no signal 0; libproc on macOS; scans only around commands."
date: "2026-11-18"
sources:
  - "The user, 2026-11-18: show that background processes still run, and which ones; much asked for."
  - "../../zendo/background.md (BG-1 to BG-10)"
  - "ADR 0014 and ADR 0015: on ACP v1 the adapter gives the life of a background task only to JetBrains AIR; ACP v2 is a draft behind `--experimental`."
  - "Claude Code 2.1.327, read from its binary on 2026-11-18: the Bash tool spawns its shell with `detached: true`; a background terminal runs as `claude bg-pty-host`."
  - "src/session/pty.rs: zendo already reads the foreground process group of its own terminals."
  - "ADR 0005 (sessions in own terminals)"
---

# 0016 Background processes come from the process tree of zendo

## Context

Users ask much to see that background processes of the agent still run, and which ones. ADR 0014
and ADR 0015 get them from the agent: on ACP v1 the Claude adapter gives them only to a client
that declares the JetBrains AIR capability, and on ACP v2 only behind `--experimental`. A Claude
Code session tells about them in no hook. Codex has its own adapter.

zendo starts every process that can start such a command: the agent process, and the process of
each Claude Code session in a terminal of its own. Claude Code starts each Bash command with
`detached: true`, so each command is a process group of its own under the Claude Code process, and
stays one when it goes to the background. zendo already reads process groups for its own terminals.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Only the agent tells (ACP v2, behind `--experimental`) | The names and the output come from the agent. | Off on ACP v1, in a Claude Code session, and for most users. |
| The tool calls tell: a step with `run_in_background`, and its later end | No process code. | The end and the stop are not known on v1; a Claude Code session has no such data. |
| zendo reads the process tree of the processes that it started | The same on v1 and v2, in a Claude Code session, and with each agent; the end and the stop are exact. | Code for each platform; zendo must tell a command from a process that the agent needs, such as an MCP server. |

## Decision

Approved by the user on 2026-11-18: the third option, with the data of the agent on top.

- zendo shall read, about once per second, the processes under the agent process and under each
  Claude Code process. A background process is a process group under them that runs a command of
  the agent or of Claude Code while no step runs it.
- zendo shall not count the processes that the agent needs for itself, such as its MCP servers.
- zendo shall name a process with the text of its step when a tool call started it, and with its
  command line when not.
- zendo shall stop one process by a signal to its process group, and by force after 5 s.
- zendo shall keep the processes that it saw after their agent or their Claude Code session ended,
  until they end.
- Where the agent tells about its background work (ACP v2 with `--experimental`), zendo shall use
  that data for the names and the output, and the process tree for the life and the stop.
- zendo shall not declare the `_meta.jetbrains.air` namespace for this.

## Consequences

- A new part of zendo reads the process tree on Linux and macOS. It may use a crate for the list of
  processes, or the platform APIs.
- The background line, the stop of one process, and the quit question of background.md work
  without `--experimental`. ACT-12, INTER-4 and INTER-12 are retired; ADR 0015 keeps ACP v2 for
  the other features.
- A process that leaves the tree on purpose, such as a daemon that starts again under the system,
  is out of the tree and does not show.
- The read of the process tree must not stall the screen (ADR 0009): it runs off the UI thread.

## Verification

- The acceptance tests of background.md start real child processes with the mock agent and the
  mock command line, and check the line, the end, the stop, and the quit question.
- A test checks that an MCP server of the mock agent does not show.
- A check with real Claude on Linux and on macOS confirms a dev server that the agent starts in
  the background.

## Decision record

Draft of the agent, 2026-11-18. Approved by the user on 2026-11-18 in chat, with PR #26.
