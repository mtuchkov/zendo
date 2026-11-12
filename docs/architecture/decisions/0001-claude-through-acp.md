---
status: "draft"
date: "2026-11-11"
sources:
  - "../../zendo/claude.md"
  - "The decision of the user to work with Claude Code from zendo, 2026-11-11"
---

# 0001 Connect to Claude through the Agent Client Protocol

## Context

zendo must let the user work with Claude on the open file: ask, get answers as a stream, and allow or
discard each edit. Claude must use the Claude Code login that the user already has. The user must
be able to continue a thread in Claude Code. The connection is in `src/agent/`; the rest of zendo
sees only commands and events.

## Options

| Option                                                                 | Benefit                                                                                                                       | Cost or constraint                                                                       |
| ---------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| ACP client, with the adapter `@agentclientprotocol/claude-agent-acp`  | A published protocol with a Rust crate. Edits come as permission requests. The session ID is a Claude Code session ID. Other ACP agents work with `--agent`. | Needs Node.js and the adapter. zendo must pin a version of the adapter that it tested.    |
| Run the Claude Code CLI in print mode and read its JSON stream         | No extra package.                                                                                                             | A format of one program. Edits that wait for the user need a separate path.              |
| Own Node.js program on the Claude Agent SDK                            | Full control of the session.                                                                                                  | zendo must write and keep a second program in another language.                           |

## Decision

Proposal: zendo is an ACP client. By default it starts the Claude Code adapter, at a pinned version,
from the data folder of zendo. `zendo setup` installs it there.

zendo shall not offer file system or terminal capabilities to the agent; the agent edits files itself
after the user allows each edit.

## Consequences

- zendo needs Node.js and a one-time install of the adapter. When the adapter is installed, no
  network is necessary (ZENDO-5).
- A new adapter version is a change of zendo: it needs a test with the mock agent and a manual check
  with real Claude.
- `ctrl-o` can give the thread to `claude --resume`.
- The mock agent `zendo-mock-agent` speaks ACP, so `cargo test` covers the client without Claude.

## Verification

`edit_is_proposed_and_applied_only_after_allow` and `shell_is_blocked_and_resume_replays` in
`tests/acp_mock.rs`; `zendo doctor --live` with real Claude.

## Decision record

Not reviewed by the user.
