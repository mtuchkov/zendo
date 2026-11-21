---
status: "draft"
date: "2026-11-17"
sources:
  - "The user, 2026-11-17: the question of the agent shall go to the user in the chat as a quick action, and `up` moves to the options."
  - "The user, 2026-11-17: rejecting Bash was a mistake. Ideally Bash runs in a terminal of zendo: if the user says yes, it runs at once; or the user says what to do instead."
  - "../../zendo/questions.md (ASK-1 to ASK-13)"
  - "../../zendo/claude.md (CLAUDE-3, CLAUDE-12 and CLAUDE-33, retired in revision 5)"
  - "ADR 0001 (Claude through ACP), ADR 0018 (command results to the agent)"
  - "Source of `@agentclientprotocol/claude-agent-acp` 0.106.0: `Monitor` has the ACP kind `other`; the shell tool runs in the process of Claude Code, and the adapter sends no `terminal/create` to the client."
  - "Claude Code permissions: the `deny` and `allow` rules and the permission mode decide first; only an open case goes to the permission prompt, which the adapter sends to the client as `session/request_permission`. Reads and searches need no approval; shell commands, file writes and web requests ask."
  - "Codex approvals: Codex reads, edits and runs routine commands in the workspace, and asks before it uses the internet or leaves the workspace."
  - "The user, 2026-11-17: do what Claude Code and Codex do. Keep the mode `auto`."
  - "ADR 0020 (answers to Claude through a hook)"
---

# 0019 Questions of the agent in the chat, and shell commands in the terminals of zendo

## Context

Claude Code checks each tool call against the permission rules and the mode of the user. A case
that they do not decide goes to the permission prompt. Through ACP, the prompt is a
`session/request_permission` to the client: zendo answers it in place of the user.

zendo now answers by the ACP kind of the call: `execute` gets *no* (CLAUDE-3), every other kind gets
*yes* at once (CLAUDE-33). Two problems:

- `Monitor` runs a shell command, but the adapter gives it the kind `other`. So zendo allows it,
  and the agent can run any command without the user. A new tool with a new kind passes the same
  way.
- The user cannot allow a useful command: zendo rejects each one in silence. The user said that
  this was a mistake.

The user wants each open question in the chat, as a quick action, and a shell command that the
user allows to run in a terminal of zendo, where the user sees it.

## Options

Which calls zendo answers itself:

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Allow all kinds but `execute` (now) | No questions. | `Monitor`, MCP tools and new tools run without the user. |
| Reject by tool name (`Monitor`, …) | A small change. | The next new tool passes again. |
| Allow the kinds read, search and edit, and the tools of `zendo`; ask for the rest | A new or unknown tool waits for the user. | A read that the agent asks about, for example outside the workspace, passes without the user. |
| Allow only edits and the tools of `zendo`; ask for the rest | The rules of the agent decide, as in Claude Code with `acceptEdits` and in Codex; zendo adds none. | A question for each open case of the agent. |

Where an allowed shell command runs:

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Allow the call; Claude Code runs it | The usual flow; the agent keeps its shell folder. | The command runs in a hidden process; the user sees only the tool line. |
| ACP terminal capability | Part of ACP. | The adapter does not run its shell tool through `terminal/create`. |
| A tool `run` of the MCP server `zendo` in place of the shell tool | A clean result to the agent. | The agent must learn a second shell tool; the shell tool stays and still needs an answer. |
| Answer *no*; zendo runs the command in its terminal and watches it (ADR 0018) | The user sees the command run; the output comes by push and pull. | The agent gets the fixed text "User refused permission to run tool"; it must check `command_result`. |
| Run the command in the terminal of zendo, and answer through a hook of Claude (ADR 0020) | The agent of Claude gets the output as a normal shell result. | Claude only. |

## Decision

Proposal: edits and the tools of `zendo` at once, a question for the rest, and shell commands in
the terminals of zendo.

- zendo shall allow at once only edits within the limits of the task and the tools of the MCP
  server `zendo` (ASK-1). The agent allows reads and searches itself. Each other question, also of a
  kind that zendo does not know, shall wait for the user in the chat, with the options of the agent
  (ASK-2).
- The question shall show the full command or the input of the tool (ASK-3), and the options under
  it in one row. `up` on the first line of the chat input moves to the options (ASK-5).
- For a shell command the options shall be `Run in zendo shell`, `No` and `Say what to do instead`. For
  another tool they are the options of the agent and `Say what to do instead`.
- zendo shall keep the permission mode of the agent, also `auto` (ASK-15). In `auto` the
  classifier of Claude Code decides first; only its open cases come to zendo.
- On `Run in zendo shell`, zendo shall run the command in a terminal of the session as a watched
  command (ASK-4, TERM-55, TERM-73). With Claude, zendo answers through its hook, so the agent gets
  the output as the result of its shell command (ADR 0020). With another agent, zendo answers *no*,
  and `command_result` tells the agent that the command ran (ASK-11).
- zendo shall answer *no* to each question of work without a user, such as a recap (ASK-7).

ADR 0001 stays: zendo gives the agent no ACP terminal capability. ADR 0018 gives the output of
an allowed command to the agent.

## Consequences

- CLAUDE-3, CLAUDE-12 and CLAUDE-33 of [Claude](../../zendo/claude.md) are retired; ASK-1 and ASK-2
  replace them. TERM-2 takes `Run in zendo shell`.
- `src/agent/acp.rs` allows only edits and the tools of `zendo`, in place of the check on `execute`, and sends the
  other questions to the app with the raw input of the call. The app gets a question item in the
  thread, and a focus on its options.
- With the hook, the agent of Claude sees a normal shell result for a command that ran in zendo.
  Without it (Codex, or the time limit of the hook), the agent sees a refusal; the push of ADR 0018
  wakes it with the result.
- The shell of the agent is not the shell of Claude Code. A terminal belongs to its session while
  the watch runs and for 10 min after (TERM-72), so a `cd` holds for the next commands of the
  session; after that the agent must use full paths or `cd` again.
- A command that the classifier of `auto`, an `allow` rule of the user (for example from "Yes, and
  don't ask again"), or the mode `bypassPermissions` allows runs in Claude Code, without a question
  to zendo and not in a terminal of zendo. zendo does not warn, and the chat shows it as a usual tool line
  (the user, 2026-11-17).
- Each new adapter version needs a check that its edits still have the kind `edit`.

## Verification

- `tests/questions.rs`: the acceptance tests of
  [Questions of the agent](../../zendo/questions.md).
- `tests/acp_mock.rs`: the mock agent requests `Monitor` with the kind `other` and a tool without
  a kind; both wait for the user.
- The adapter test of [ADR 0020](0020-answers-to-claude-through-a-hook.md): without
  `defaultMode` the adapter starts the session in `auto`.
- `zendo doctor --live` with real Claude: a shell command of the agent asks, `Run in zendo shell` runs
  it in the `F5` terminal, and the agent continues with the output.
