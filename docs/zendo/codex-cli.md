---
status: "draft"
revision: "1"
date: "2026-11-18"
parent: "zendo.md"
prev: "cli-activity.md"
next: "back-to-the-agent.md"
level: "the hand-off of a Codex thread to `codex resume` in a terminal of zendo, and what differs from a Claude Code session"
note: "Draft from the requirements of the user on 2026-11-18 in chat. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Codex in its terminal

With Codex as the agent of the chat ([Claude](claude.md)), `ctrl-o` gives the thread of the
active agent session to `codex resume`, in a terminal that zendo owns. The session view, its keys,
the five sessions at a time, the quit question and the end of the session are the same as for a
Claude Code session ([Sessions](sessions.md)). The ACP adapter of Codex gives each thread the
Codex thread ID as its ACP session ID, so the resume continues the same thread.

**The command line comes from the profile.** The profile of the agent names the tool and the
arguments before the thread ID: `claude --resume` for Claude, `codex resume` for Codex. Only
Claude Code gets the options of Claude Code: the hooks of zendo and the MCP server of the zendo
session ([zendo session](zendo-session.md)). Codex gets none of them.

**State from the terminal.** Codex sends zendo no hooks. Its state comes from the activity of its
terminal: *works* while it prints, *idle* when it is quiet. It never shows *waits*.

**What does not show.** Without hooks, the chat of the agent session does not show the prompts
and answers of the Codex session ([Claude Code messages in the chat](cli-messages.md)) or its
activity ([Activity of a Claude Code session](cli-activity.md)). When the session ends, zendo
opens the agent session again, and the chat shows the thread as the adapter loads it.

## Architecturally significant requirements

| ID         | Pattern    | Requirement                                                                                                                                          | Source           |
| ---------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| CODEXCLI-1 | Event      | When the user presses `ctrl-o` in a Codex session, the system shall start `codex resume <thread id>` in the terminal of that session.               | User, 2026-11-18 |
| CODEXCLI-2 | Ubiquitous | The system shall build the arguments of the command line from the profile of the agent, and shall give the Claude Code options only to Claude Code. | User, 2026-11-18 |

### Acceptance tests

The **mock command line** replaces `codex`. The tests are in `tests/sessions.rs`.

**`ctrl_o_resumes_the_codex_thread`** (mock command line; verifies CODEXCLI-1, CODEXCLI-2)

- Given Codex as the agent, and an agent session with a thread ID.
- When the user presses `ctrl-o` in its chat.
- Then the session view shows the command line with `resume` and the thread ID, without
  `--settings` and `--mcp-config`.

**`the_profile_builds_the_command_line_of_the_handoff`** (unit test; verifies CODEXCLI-1,
CODEXCLI-2)

- Given the profiles of Claude and Codex.
- When zendo builds the command line of the hand-off of each.
- Then Claude gets `claude --resume` with the options of Claude Code, and Codex gets
  `codex resume` without them.

## Detailed requirements

| ID         | Pattern | Requirement                                                                                               | Source           |
| ---------- | ------- | --------------------------------------------------------------------------------------------------------- | ---------------- |
| CODEXCLI-3 | State   | While a command-line session sends no hooks, the system shall take its state from the activity of its terminal. | User, 2026-11-18 |

Test of CODEXCLI-3: `a_session_without_hooks_takes_its_state_from_its_terminal` in
`tests/sessions.rs`.

## Open points

| Open question                                                                                          | Affects                | Answered by |
| ------------------------------------------------------------------------------------------------------ | ---------------------- | ----------- |
| Should Codex get the MCP server of the zendo session, through its own configuration options?           | CODEXCLI-2, SESSIONS-21 | User        |
| Should the chat show the prompts and answers of a Codex session, from the session files of Codex?      | CLIMSG-1               | User        |
| Should the end of a Codex turn ring the bell, from the activity of its terminal?                       | ATTN-1, CODEXCLI-3     | User        |
