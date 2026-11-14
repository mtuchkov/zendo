---
status: "draft"
revision: "2"
date: "2026-11-13"
parent: "zendo.md"
prev: "work.md"
next: "sessions.md"
level: "the connection to Claude: what zendo sends, what Claude may do, threads, failures"
note: "Agent rewrite of the reference of zendo 0.9.1. Not reviewed by the user."
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

**Threads.** zendo keeps five agent sessions for the repository, each one thread for all files.
`F6` to `F10` (or `alt` and the F key, where the terminal takes the F key) choose a session, and
`alt-]` goes to the next one, after the last through the terminals of [Terminal](terminal/terminal.md),
then to the first; `alt-[` goes back. Buttons on the right edge of the chat show
the sessions one under the other: the active one in the accent color, the others in the second
accent, as the badges of the modes; a click chooses one. Each thread is a Claude Code
session, so the user can continue it in Claude Code. zendo keeps the session ID in its state in the
repository. When the chat opens the first time, zendo opens the session: the chat shows the
messages of the last session, and the agent sends its slash commands.

**When Claude is not available.** At the start, zendo does not open (ZENDO-2). While zendo runs, the
chat shows why the agent stopped, and zendo starts it again with the next question.

**Agents.** zendo works with Claude (the default) and with Codex (`zendo setup --agent codex`), each
through its pinned ACP adapter, and with any other ACP agent command. What an agent cannot do turns off
only its keys: with Codex there is no hand-off to the command line (`ctrl-o`); the rest works the same.

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

**`shell_is_blocked_and_resume_replays`** (mock agent; verifies CLAUDE-3, CLAUDE-6)

- Given a thread with the mock agent.
- When the agent requests a shell command, and later the user opens the thread again.
- Then the command is rejected without a question, and the chat shows the earlier messages.

## Detailed requirements

| ID        | Pattern    | Requirement                                                                                                                  | Source           |
| --------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| CLAUDE-4  | Unwanted   | If the agent stops while zendo runs, then the system shall show the reason in the chat and start the agent with the next question. | Design      |
| CLAUDE-5  | Ubiquitous | The system shall keep five agent sessions for the repository, each one thread for all files, and keep their session IDs and the active session in the repository. | User, 2026-11-12 |
| CLAUDE-6  | Event      | When the user asks in a thread that has a session ID, the system shall continue that session and show its earlier messages. | Design           |
| CLAUDE-9  | Event      | When the first question of a thread goes to Claude, the system shall send `.zendo/prompt.md`, if it exists. | Design |
| CLAUDE-26 | Event      | When a question goes to Claude and the open file did not go with one of the last 20 questions, the system shall attach the file and send the project rules for it. | User, 2026-11-12 |
| CLAUDE-31 | Event      | When the user presses `F6` to `F10`, or `alt` and one of them, the system shall open the chat on that agent session; when the user presses `alt-]` or `alt-[`, on the next or the previous session, after the last the terminals (TERM-41, TERM-49). | User, 2026-11-12; User, 2026-11-13 |
| CLAUDE-32 | Ubiquitous | The system shall show a button for each agent session on the right edge of the chat, one under the other, named by its F key: the active one in the accent color, the others in the second accent; a click chooses the session. | User, 2026-11-12 |
| CLAUDE-35 | Unwanted   | If the keymap in effect gives the agent sessions other keys than `F6` to `F10`, then the system shall name the agent sessions `1` to `5` on their buttons and in their names. | User, 2026-11-13 |
| CLAUDE-27 | Event      | When the chat opens the first time, the system shall open the session of the thread and show the messages of the last session in their order, each question without the context that zendo sent with it. | User, 2026-11-12 |
| CLAUDE-10 | Optional   | Where the agent has a command line with a resume, the system shall continue the thread there when the user presses `ctrl-o`. | Design           |
| CLAUDE-13 | Ubiquitous | The system shall work with Claude and with Codex through their pinned ACP adapters, and with any ACP agent command.         | User, 2026-11-11 |
| CLAUDE-14 | Unwanted   | If the agent cannot do a feature, then the system shall turn off only the keys and hints of that feature.                   | User, 2026-11-11 |
| CLAUDE-11 | Event      | When the user presses `esc` during an answer, the system shall stop the answer.                                              | Design           |
| CLAUDE-12 | Ubiquitous | The system shall not tell the agent that zendo can read files or run commands for it.                                         | Design           |

Retired in revision 1: CLAUDE-2 (wait for the decision of the user on an edit), CLAUDE-7 (show a proposed edit inline in the note) and CLAUDE-8 (`e` opens a proposed edit in the editor), because zendo does not wait for the user on an edit (User, 2026-11-12). Retired identifiers are never reused.

Tests: `one_thread_for_every_file_and_the_file_goes_once` (CLAUDE-5, CLAUDE-26) and
`the_chat_shows_the_last_session_when_it_opens` (CLAUDE-27, CLAUDE-6),
`five_agent_sessions_with_f6_to_f10_and_alt_n` (CLAUDE-5, CLAUDE-31, CLAUDE-32) and
`alt_and_an_f_key_choose_a_session_too` (CLAUDE-31) in `tests/chat_flows.rs`,
`other_session_keys_name_the_buttons_1_to_5_and_the_terminal_by_its_mark` (CLAUDE-35, TERM-48) in
`tests/keymap_hints.rs`.
