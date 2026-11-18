---
status: "approved"
approved: "2026-11-18"
date: "2026-11-18"
sources:
  - "The user, 2026-11-18: make sure that ACP v2 is not abandoned; capture it as a decision record, and plan to support it as `--experimental`."
  - "ADR 0014 (the activity of the agent from ACP and from the hooks of Claude Code), ADR 0016 (background processes from the process tree of zendo)."
  - "The ACP v2 draft announcement, https://agentclientprotocol.com/announcements/acp-v2-draft (2026-07-20), and the v2 migration guide, https://agentclientprotocol.com/protocol/v2/migration, read on 2026-11-18."
  - "The repository agentclientprotocol/agent-client-protocol, read on 2026-11-18: 209 commits to schema/v2 and the v2 docs since 2026-10-18; tags schema-v2.0.0-alpha.10 (2026-10-27) to alpha.16 (2026-11-14)."
  - "The crate agent-client-protocol 5.2.0: the feature `unstable_protocol_v2`, `Client.v2()`."
  - "The adapters @agentclientprotocol/claude-agent-acp 0.106.0 (dist/v2: `state_update` with `background`, `terminal/cancel`, `toolCall/cancel`, subagents as child sessions) and @agentclientprotocol/codex-acp 2.7.0 (negotiates version 2 or 1)."
  - "../../zendo/background.md"
---

# 0015 ACP v2 behind `--experimental`

## Context

ADR 0014 keeps zendo on ACP v1. On v1, the Claude adapter tells about background work only to a
client that declares the `asyncTasks` capability of JetBrains AIR. That capability makes the
adapter treat zendo as JetBrains AIR: tool calls lose `rawInput` and `rawOutput`, `_meta` comes as
deltas in an AIR format, and other behavior changes. ADR 0016 shows the background processes and
stops one of them from the process tree of zendo on every version, but the agent knows more: the
name of the work, its live output, and subagents that run in the background.

ACP v2 gives these to every v2 client, without a vendor namespace: the list of background work in
`state_update`, `terminal/cancel` and `toolCall/cancel` for one item, subagents as child sessions,
and display-only terminals with live output. Both pinned adapters negotiate v2 and fall back to v1.

v2 is alive: its draft came on 2026-07-20, and the repository of the specification had 209 commits
to v2 in the last month and seven alpha tags since 2026-10-27. But v2 is still a draft. It has no
date for stabilization, its parts can change before then, and the maintainers ask to gate it behind
version negotiation and feature flags and not to turn it on by default. v1 stays: v1-only peers
will be common for some time. The Rust SDK has v2 only behind the crate feature
`unstable_protocol_v2`.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Stay on v1 only | Stable; no new code path. | Background processes only from the process tree: no live output, no subagents in the background. |
| Declare the `asyncTasks` capability of JetBrains AIR on v1 | The features on v1. | zendo reads the AIR format of a vendor; other behavior of the adapter changes. |
| v2 by default, v1 as a fallback | Every user gets the features. | Against the advice of the maintainers; a change of the draft can break zendo for every user. |
| v2 behind `--experimental`, v1 by default | The features for a user who asks; v1 users keep a stable zendo. | Two code paths in the ACP client while v2 is a draft. |

## Decision

Approved by the user on 2026-11-18: v2 behind `--experimental`.

- zendo shall have the start flag `--experimental`. It turns on the features that depend on a
  draft protocol. ACP v2 is the first of them.
- Where zendo starts with `--experimental`, its ACP client shall request protocol version 2 in
  `initialize`. Where the agent answers with version 1, zendo shall go on with v1 and show once
  that the agent has no v2.
- Where zendo starts without `--experimental`, zendo shall request version 1, as now (ADR 0014).
- On v2, zendo shall take the names and the live output of background work from the `background`
  list of `state_update` and from its display terminals, and shall stop one item with
  `terminal/cancel` or `toolCall/cancel` when the agent offers it. The process tree of ADR 0016
  stays the source of the life of a process (BG-1, BG-3).
- On v1, the background line shows the process tree only, and the rest of ADR 0014 stays as it
  is.
- zendo shall not declare the `_meta.jetbrains.air` namespace on either version.
- zendo shall build the crate `agent-client-protocol` with the feature `unstable_protocol_v2`, and
  shall keep the v2 path behind the flag at run time.

## Consequences

- The ACP client of zendo gets a second path. The events of zendo stay the same for both paths,
  so the chat does not depend on the version.
- v2 has no client terminals and no session modes as API. zendo does not use client terminals,
  and on v2 the mode comes from the session config options.
- The mock agent learns v2, so that the tests cover both paths.
- An upgrade of the pinned adapter or of the crate needs the v2 tests to pass. A change of the v2
  draft can break `--experimental` only, not the default zendo.
- When v2 becomes stable, a new decision record shall make v2 the default and retire this flag
  for ACP.

## Verification

- A test starts zendo with and without `--experimental` against the mock agent, and checks the
  protocol version in `initialize`.
- A test with a v1-only mock agent checks the fallback and the message.
- The acceptance tests of background.md run on both paths; on the v2 path they also check the
  names and the output from the agent.
- A check with real Claude and the pinned adapter confirms the background list and the stop of
  one item on v2.

## Decision record

Draft of the agent, 2026-11-18. Approved by the user on 2026-11-18 in chat, with PR #26.
