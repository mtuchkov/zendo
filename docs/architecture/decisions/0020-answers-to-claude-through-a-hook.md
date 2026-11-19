---
status: "draft"
date: "2026-11-17"
sources:
  - "The user, 2026-11-17: why does zendo answer *no* to a command that it runs? Could the answer carry instructions?"
  - "../../zendo/questions.md (ASK-4, ASK-6, ASK-11)"
  - "ADR 0018 (command results to the agent), ADR 0019 (questions of the agent), ADR 0014 (the activity of the agent from ACP and the hooks of Claude Code; CLIACT-10 answers a Claude Code session through a blocking `PermissionRequest` hook)"
  - "Source of `@agentclientprotocol/claude-agent-acp` 0.106.0: the adapter reads only the `optionId` of the ACP answer, and turns each reject into the text \"User refused permission to run tool\"; it passes `_meta.claudeCode.options.settings` of `session/new` to Claude Code."
  - "Claude Code docs: the `PermissionRequest` hook returns `decision.behavior` `allow` with `updatedInput`, or `deny` with a `message` for Claude."
  - "The test of 2026-11-17, below."
---

# 0020 Answers to Claude through a PermissionRequest hook

## Context

For a shell command that the user runs in zendo (`Run in zendo shell`, ASK-4), the agent must get the
output, and for `No` or `Say what to do instead` (ASK-6) it should get a text from zendo. An ACP
answer to a permission request holds only the id of an option. The Claude adapter gives the model
the fixed text "User refused permission to run tool" for each reject. So through ACP the agent
gets a bare refusal, and must find the result itself with `command_result`.

Claude Code has two hooks before a tool call. `PreToolUse` runs before the permission rules, for
each call, also for `ls`, which Claude Code allows itself. `PermissionRequest` runs when Claude Code
would ask the user, so it fits the rules of ADR 0019. It can answer *allow* with a new input, or
*deny* with a text for the model.

The test of 2026-11-17 used the adapter 0.106.0 (Claude Code 2.1.322), a mock Messages API that
always asks for `Bash`, and a minimal ACP client in place of zendo, in the mode `default`:

| Case | Result |
| ---- | ------ |
| No hook; the client answers *reject* | The ACP request came; the model got "User refused permission to run tool". |
| A `PermissionRequest` hook in `_meta.claudeCode.options.settings`; the client answers at once | The hook ran with the tool input, and the ACP request came too; the answer of the client won. |
| The same; the client answers after 8 s; the hook answers *deny* with a text | The model got the text of the hook. |
| The same; the hook answers *allow* with a new command | The new command ran, the old one did not; the model got a normal shell result. |

The hook and the ACP request start together, and the first answer wins. ADR 0014 already answers
a Claude Code session in a terminal through the same blocking hook (CLIACT-10); this decision uses
it also for a chat of Claude through ACP.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| ACP answer only | One path. | A bare refusal; the agent must call `command_result`. |
| `PreToolUse` hook | Runs before every rule. | Fires for each shell call, also for those that Claude Code allows itself. |
| `PermissionRequest` hook that waits for the user, with the ACP answer as the fallback | A normal shell result, or a refusal with a text. | Claude only; a time limit; zendo must hold the ACP request. |
| A change of the adapter: read a message and a new input from `_meta` of the ACP answer | No hook. | A change in a project of another owner; zendo depends on its release. |

## Decision

Proposal: a `PermissionRequest` hook that waits for the user, with the ACP answer as the fallback.

- zendo shall start each Claude session with a `PermissionRequest` hook for the shell tool in
  `_meta.claudeCode.options.settings` of `session/new`, not in a file of the user (SESSIONS-18).
  The hook runs `zendo hook permission --session N --pid P`, with a time limit of 600 s.
- For a shell command, zendo shall show the ACP request as the question (ASK-2) and hold it. The
  answer of the user goes to the hook through a file in `.zendo/watch/<pid>-<session>/`:
  - `Run in zendo shell`: zendo runs the command as a watched command (ASK-4), and the hook answers
    *allow* with the command `zendo await <id>`. `zendo await` prints the output of the terminal and
    exits with its exit code. Before the time limit of the shell tool, it prints the output so far
    and says that the rest comes as `zendo_command_result` (ADR 0018).
  - `No`: the hook answers *deny* with "The user said no to this command."
  - `Say what to do instead`: the hook answers *deny* with "The user will say what to do instead.
    Wait for the next message."
- When the hook has answered, zendo shall drop the held ACP request.
- If the hook ends without an answer, or the agent is not Claude, zendo shall answer the ACP
  request as ADR 0019 says.
- Claude Code in a terminal keeps the choices of CLIACT-10: zendo does not run its commands
  (the user, 2026-11-18).

## Consequences

- New: `zendo hook permission` and `zendo await`. A question in the app keeps the id of its hook
  file.
- `zendo await` runs as a shell command of Claude Code: in its sandbox, if the user has one, and
  only reads the folder of the watches.
- A `PermissionRequest` hook of the user runs beside the hook of zendo; the first answer wins.
- An answer of the user after 600 s goes through ACP: the agent gets a bare refusal, then the push.
- The test must run again with each new adapter version, because the race of the hook and the ACP
  request is not documented.

## Verification

- The test of the Context, as `tests/permission_hook.rs`, with the pinned adapter and a mock
  Messages API: `run_in_zendo_shell_gives_the_output_as_a_shell_result`,
  `no_gives_the_text_of_zendo`, `a_late_answer_goes_through_acp`.
- `zendo doctor --live` with real Claude: `Run in zendo shell` on `echo zendo-42` gives the agent
  `zendo-42` as the shell result.

## Decision record

Proposed by the agent on 2026-11-17 after the question of the user and the test above. Not
reviewed by the user.
