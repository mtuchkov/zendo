---
status: "draft"
date: "2026-11-17"
sources:
  - "The user, 2026-11-17: the user clicks run, the command runs in the shell, and the agent gets its output through zendo: `▶` as today, and `▶▶` with an automatic watcher, also for intermediate output. Use push and MCP pull together."
  - "The user, 2026-11-17: the agent shall not run commands in the shells of zendo through an MCP tool."
  - "../../zendo/terminal/terminal-watch.md (TERM-55 to TERM-71)"
  - "../../zendo/terminal/terminal-output.md (TERM-25)"
  - "../../zendo/questions.md (ASK-2, ASK-4) and ADR 0019"
  - "ADR 0001 (Claude through ACP), ADR 0003 (state in the repository), ADR 0005 (sessions in own terminals)"
  - "Source of `@agentclientprotocol/claude-agent-acp` 0.106.0 and 0.108.0: `Monitor` is an agent control tool with the kind `other`; a turn that Claude Code starts without a prompt goes to the client only with the AIR capability `asyncTasks`."
  - "Claude Code tool reference: `Monitor` streams each stdout line of a command as a notification; a monitor ends after at most 10 minutes."
---

# 0018 Command results to the agent: push through a prompt, pull through MCP

## Context

The agent proposes shell commands, and the user runs them in the terminals of zendo. The agent does
not see the output: the user copies it with `alt-o` (TERM-24, TERM-25). The user wants the agent to
get the output of a command that the user ran with `▶▶`, at the end and while it runs.

Constraints:

- A command runs only when the user says so (TERM-2). ADR 0001 gives the agent no terminal
  capability. Each tool call of the agent that is not safe waits for the user (ADR 0019).
- ACP v1 is driven by the client: a turn starts with `session/prompt`. The agent cannot start a turn
  for an event. The Claude adapter sends such a turn only to a client with the AIR extension
  `asyncTasks`; zendo does not have it.
- zendo runs no server process (TERM-5, ADR 0005). The MCP server `zendo mcp --session N` is a
  child of the agent, not of the screen of zendo (SESSIONS-21).
- The same thread can run in Claude Code in a terminal (`ctrl-o`), where zendo has no ACP
  connection and must not type into the TUI.
- zendo works with Claude and with Codex (CLAUDE-13).

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| `alt-o` only (now) | No new path to the agent. | A manual step for each result; no progress. |
| An MCP tool that runs a command in a shell of zendo | The agent works alone. | Runs without a question, outside the sandbox and the permission rules of the agent, in the shell of the user, with its environment and its foreground program. Rejected by the user. |
| ACP terminal capability (`terminal/create`) | Part of ACP. | The agent starts the command: the same problem as the row above. |
| Claude `Monitor` on `zendo watch` | Wakes Claude; streams progress. | Claude only. A monitor lives at most 10 minutes. `Monitor` runs a command, so each monitor is a question to the user (ASK-2). Through ACP the wake needs `asyncTasks`. |
| Push: zendo sends the output as `session/prompt` | Plain ACP; Claude and Codex; no command for the agent; no time limit. | Each push is a turn of the agent. Not for Claude Code in a terminal. |
| Pull: MCP tool `command_result` | Works in the chat and in Claude Code; the agent gets more output when it needs it. | Works only in a turn: it does not wake the agent. |
| Push and pull with one position in the output | Push wakes the agent; pull serves it in a turn; no line comes twice. | Two paths to keep. |

## Decision

Proposal: push and pull with one position in the output.

- zendo shall watch only a command that the user started with `▶▶` or `alt-F5`, for the agent
  session of the answer that holds the block (TERM-55).
- zendo shall send the output to that thread as a `session/prompt` of its own, at the end of the
  command and after each progress interval with new output (TERM-56, TERM-57). While a turn runs,
  zendo shall keep the output and send one prompt at the end of the turn (TERM-58). While the
  thread runs in Claude Code in a terminal, the push shall wait for the chat (TERM-66).
- zendo shall put the output into the tag
  `<zendo_command_result id="…" exit="…" state="running|ended|stopped" lines="…">`, with
  `<command>` and `<output>` in it. zendo shall write each `</zendo_command_result` in the command
  and the output as `<\/zendo_command_result`, so the output cannot close the tag (TERM-61). The instructions of the MCP server `zendo` shall tell the
  agent that this tag is an event of zendo with data, not a request of the user.
- After each check-in interval of a watch, zendo shall push a check-in: the same tag with the
  state `running`, the run time, the time since the last output, and the question whether the run
  goes well (TERM-70). So the agent sees also a command that prints nothing.
- The MCP server `zendo` shall have the tool `command_result` with an optional wait (TERM-59,
  TERM-64), and the tool `end_watch` (TERM-71). `end_watch` ends only the watch, never the command. The wait shall be short, below the time limit of a tool call of the agent.
- zendo shall write the state of each watch, its output and its position, to
  `.zendo/watch/<pid>-<session>/<id>.json`, by a write to a temporary file and a rename (ADR 0003).
  The screen and `zendo mcp --session N --pid P` move the position under a file lock. So the screen
  and the MCP server share the position without a server process, and two zendo processes in one
  repository do not mix their watches.
- A shell command of the agent that the user allows runs as a watched command too (ADR 0019,
  ASK-4), so the agent gets its output the same way.

ADR 0001 stays: zendo gives the agent no terminal capability.

## Consequences

- TERM-21 and TERM-25 change; `▶▶` is a second button on each shell block (TERM-63).
- Each progress push is a turn of the agent and costs tokens. The progress interval is in the
  config, 30 s by default (TERM-69). A watch has no time limit (TERM-62): a long command, such as a
  dev server, sends a push every 30 s with new output, and a check-in every 10 min, until the user
  or the agent ends the watch.
- The output goes into the context of the model. It can hold secrets, for example of `env`, and
  text that asks the agent to act. Each command of the agent waits for the user (ASK-2), so such
  text can lead at most to file edits (ASK-1) and to questions that the user must answer.
- `zendo mcp` gets the argument `--pid`, and reads the folder of the watches. The instructions of
  the MCP server (`resources/mcp-instructions.md`) get the tag and the tool.
- The mock agent must accept prompts of zendo and call `command_result`.
- Codex gets push and pull the same way; no key turns off (CLAUDE-14).

## Verification

- `tests/terminal_watch.rs`: the acceptance tests of [Watched commands](../../zendo/terminal/terminal-watch.md).
- `src/mcp/mod_tests.rs`: `command_result` with and without a wait, and the shared position.
- A unit test: a command and an output with `</zendo_command_result>` stay inside the tag.
- `zendo doctor --live` with real Claude: a `▶▶` block wakes the agent, and the agent reads more
  with `command_result`.

## Decision record

Proposed by the agent on 2026-11-17 from the requests of the user in the chat. The user chose push
and pull together. The tag format, the folder of the watches and the file lock are proposals of the
agent. Not reviewed by the user.
