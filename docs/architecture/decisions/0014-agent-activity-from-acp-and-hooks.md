---
status: "approved"
approved: "2026-11-18"
date: "2026-11-18"
sources:
  - "The user, 2026-11-18: the chat shall show the current state of the agent, so that the user is calm and does not switch to Claude Code; the same detail for a Claude Code session; answers to Claude Code in the chat; the latest Claude Code."
  - "../../zendo/activity.md, ../../zendo/checklist.md, ../../zendo/live-status.md, ../../zendo/interactivity.md, ../../zendo/cli-activity.md"
  - "The pinned adapter @agentclientprotocol/claude-agent-acp 0.106.0 (src/setup/config/profile.rs), read on 2026-11-18: dist/v1/acp-agent.js, dist/async-tasks.js, dist/tool-calls/client-capabilities.js, dist/context-compaction.js, dist/prompt-suggestions.js, dist/elicitation.js."
  - "The crate agent-client-protocol 5.2.0 with agent-client-protocol-schema 1.17.0: v1 SessionUpdate, elicitation."
  - "Claude Code 2.1.327: the hook events and the decision of the PermissionRequest hook, read from its binary on 2026-11-18."
  - "ADR 0001 (Claude through ACP), ADR 0005 (sessions in own terminals)"
---

# 0014 The activity of the agent comes from ACP and from the hooks of Claude Code

## Context

The chat shows the text of the agent, a sign that it thinks, and one line for each tool call.
Users open Claude Code (`ctrl-o`) only to see that the agent works. The new specifications ask
for the step that runs, its state and result, the plan, the questions of the agent, the commands,
the thought, the waits, steering, and the mode, also for a Claude Code session.

The pinned adapter already sends more than zendo reads: the plan, the state, the content and the
locations of each tool call, the text of the thought, the use of the context, and the mode. Some
of it comes only when the client says that it can show it:

- The question tool of Claude is off unless the client has `elicitation.form`.
- The output of a command comes only to a client with `_meta.terminal_output`.
- Compaction updates come only to a client with `session.compaction`.

Some of it is an extension of this adapter, not of ACP. The adapter advertises
`_session/steering` in `_meta.steering`. It sends the rate limit in the `_meta` of
`usage_update`, and the suggested next question in `_meta.promptSuggestions` of
`session_info_update`. The adapter always steers at priority `now`: it stops the text that the
agent writes and moves a running tool into the background.

The life of a background task, and its stop through `_session/async_task/stop`, go only to a
client that declares the `asyncTasks` capability in `_meta.jetbrains.air`. A client that declares
that namespace counts as JetBrains AIR, and the adapter then changes other behavior for it.

A Claude Code session has no ACP. Its hooks (SESSIONS-18) in 2.1.327 have events before and after
each tool, for a failed tool, for subagents, for compaction, for tasks, for a failed turn, and for
permission requests. A `PermissionRequest` hook can answer `allow` with an `updatedInput`, which
also carries the answers to a question, or `deny`.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Read only standard ACP v1 | Works the same with each agent. | No steering, no rate limit, no suggestion, no output of commands. |
| Standard ACP v1, the client capabilities of the features, and the extensions of the adapter where it advertises them | Almost all of the specifications; each extension turns off alone (CLAUDE-14). | Extensions can change with the adapter; the pin and the tests guard them. |
| Also declare the `asyncTasks` capability of JetBrains AIR | Background tasks and the stop of one task. | zendo would count as JetBrains AIR, and other behavior of the adapter would change. |
| Run Claude Code for each chat and read its screen | One source for both kinds of chat. | Two writers of a thread, five processes, and a screen format that is not an interface. |

## Decision

Approved by the user on 2026-11-18: the second option.

- In the handshake, zendo shall declare `elicitation.form`, `_meta.terminal_output`, and
  `session.compaction`.
- zendo shall read the plan, the state, content and locations of tool calls, the text of the
  thought, `usage_update` with its rate limit, the mode, the compaction updates, and the suggested
  next question.
- zendo shall send `_session/steering` only when the adapter advertises it. zendo shall hold a
  question until the step that runs ends, and send it at once only on `steer_now` (INTER-1,
  INTER-7).
- zendo shall not declare the `_meta.jetbrains.air` namespace. The background processes come from
  the process tree of zendo on every version (ADR 0016).
- For a Claude Code session, zendo shall read the activity only from the hooks of its run and from
  its transcript (CLIACT-2), and shall answer a permission or a question through a blocking
  `PermissionRequest` hook (CLIACT-10).
- zendo shall work with the latest Claude Code at the date of its release (CLIACT-12).

## Consequences

- The ACP client of zendo reads more updates; the events of zendo grow by the plan, the tool
  content, the thought text, the usage, the mode, the compaction, and the suggestion.
- The mock agent learns these updates and the steering request.
- An upgrade of the pinned adapter needs a check of the extensions that zendo uses.
- The hooks of a Claude Code session get the events above. The `PermissionRequest` hook waits for
  zendo, so a lost answer must not hang Claude Code: the hook gives up when zendo ends.
- With Codex, the features of the extensions are off.

## Verification

- The acceptance tests of activity.md, checklist.md, live-status.md and interactivity.md run with
  the mock agent; the tests of cli-activity.md run with the mock command line.
- A test checks the capabilities that zendo sends in the handshake.
- A check with real Claude confirms the steering, the question tool, and the output of a command
  with the pinned adapter.
