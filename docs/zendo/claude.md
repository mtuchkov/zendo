---
status: "draft"
revision: "4"
date: "2026-11-14"
parent: "zendo.md"
prev: "work.md"
next: "threads.md"
level: "the connection to Claude: what zendo sends, what Claude may do, the agents, failures"
note: "Agent rewrite of the reference of zendo 0.9.1. Not reviewed by the user. Revision 4 moved the threads to threads.md."
---

# Claude

zendo is an Agent Client Protocol (ACP) client. It starts the Claude Code adapter in the root of the
repository. The adapter uses the Claude Code login of the user. zendo checks the connection before
it opens the screen. See [ADR 0001](../architecture/decisions/0001-claude-through-acp.md).

**What zendo sends.** With the first question of a thread, zendo sends the instructions of the user
from `.zendo/prompt.md`, when that file exists. With each question, zendo sends the name of the open
file and, when the user selected text, the text with its line numbers. When the open file did not
go with one of the last 20 questions, zendo also attaches the file and sends the rules of
`project-structure.toml` that cover it. A slash command (`/name …`) goes to the agent as the user
typed it, without context.

**What Claude may do.** Claude reads and edits files itself. When the agent asks for permission
for an edit, zendo allows it at once. The chat shows the edit as one line with the tool and the file, as for
every tool, without the diff; zendo does not wait for the user.
zendo rejects each shell command.

**When Claude is not available.** At the start, zendo does not open (ZENDO-2). While zendo runs, the
chat shows why the agent stopped, and zendo starts it again with the next question.

**Agents.** zendo works with Claude (the default) and with Codex (`zendo setup --agent codex`), each
through its pinned ACP adapter, and with any other ACP agent command. What an agent cannot do turns off
only its keys: with Codex there is no hand-off to the command line (`ctrl-o`); the rest works the same.

The five threads of the chat, and how zendo keeps and resumes them, are in [Threads](threads.md).
Claude Code sessions that run in the background while the user works in zendo are in
[Sessions](sessions.md). The chat input, its chips and attachments are in [Chat](chat.md).

## Architecturally significant requirements

| ID       | Pattern  | Requirement                                                                                                                                 | Source                 |
| -------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------- |
| CLAUDE-1 | Event    | When zendo starts without `--offline`, the system shall start the agent and finish the ACP handshake and the login check before it opens the screen. | ZENDO-2           |
| CLAUDE-33 | Event   | When the agent asks for permission for a tool call that is not a shell command, the system shall allow it at once, within the limits of the task (STRUCT-3, NOTESREP-4), and show it in the chat as one line with the tool and the file, without the diff. | User, 2026-11-12 |
| CLAUDE-3 | Unwanted | If Claude requests a shell command, then the system shall reject it without a question to the user.                                          | User, 2026-11-11       |

### Acceptance tests

**`start_check_fails_without_claude`** (mock agent, not written yet; verifies CLAUDE-1). See
[zendo](zendo.md).

**`edit_is_applied_without_a_question`** (mock agent; verifies CLAUDE-33)

- Given a thread with the mock agent.
- When the agent requests an edit.
- Then the file changes at once, the chat shows one line with the tool and the file and no diff, and
  the chat asks the user nothing.

**`shell_is_blocked_and_resume_replays`** (mock agent; verifies CLAUDE-3, and CLAUDE-6 of
[Threads](threads.md))

- Given a thread with the mock agent.
- When the agent requests a shell command, and later the user opens the thread again.
- Then the command is rejected without a question, and the chat shows the earlier messages.

## Detailed requirements

| ID        | Pattern    | Requirement                                                                                                                  | Source           |
| --------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| CLAUDE-4  | Unwanted   | If the agent stops while zendo runs, then the system shall show the reason in the chat and start the agent with the next question. | Design      |
| CLAUDE-9  | Event      | When the first question of a thread goes to Claude, the system shall send `.zendo/prompt.md`, if it exists. | Design |
| CLAUDE-26 | Event      | When a question goes to Claude and the open file did not go with one of the last 20 questions, the system shall attach the file and send the project rules for it. | User, 2026-11-12 |
| CLAUDE-13 | Ubiquitous | The system shall work with Claude and with Codex through their pinned ACP adapters, and with any ACP agent command.         | User, 2026-11-11 |
| CLAUDE-14 | Unwanted   | If the agent cannot do a feature, then the system shall turn off only the keys and hints of that feature.                   | User, 2026-11-11 |
| CLAUDE-11 | Event      | When the user presses `esc` during an answer, the system shall stop the answer.                                              | Design           |
| CLAUDE-12 | Ubiquitous | The system shall not tell the agent that zendo can read files or run commands for it.                                         | Design           |

The requirements of the threads (CLAUDE-5, CLAUDE-6, CLAUDE-10, CLAUDE-27, CLAUDE-31, CLAUDE-32,
CLAUDE-35, CLAUDE-40) are in [Threads](threads.md).

Retired in revision 1: CLAUDE-2 (wait for the decision of the user on an edit), CLAUDE-7 (show a proposed edit inline in the note) and CLAUDE-8 (`e` opens a proposed edit in the editor), because zendo does not wait for the user on an edit (User, 2026-11-12). Retired identifiers are never reused.

Tests: `one_thread_for_every_file_and_the_file_goes_once` (CLAUDE-26) in `tests/chat_flows.rs`.
