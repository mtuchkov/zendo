---
status: "draft"
approved: "2026-11-18"
revision: "6"
date: "2026-11-18"
parent: "zendo.md"
prev: "work.md"
next: "questions.md"
level: "the connection to Claude: what zendo sends, what Claude may do, the agents, failures"
note: "Agent rewrite of the reference of zendo 0.9.1. Not reviewed by the user. Revision 4 moved the threads to threads.md. Revision 5: CLAUDE-26 names the choice of the open document (chat-chips.md), asked by the user on 2026-11-17; not reviewed yet. The flags: `--claude` of doctor is `--agent`, and `--agent CMD` is `--agent-cmd CMD`, on the request of the user on 2026-11-18 (approved by the user on 2026-11-18). Revision 6 (agent draft, not reviewed): the questions of the agent moved to questions.md; CLAUDE-3, CLAUDE-12 and CLAUDE-33 retired (the user, 2026-11-17)."
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

**What Claude may do.** Claude reads and edits files itself, and zendo allows it at once. Every
other tool call, a shell command too, waits for the user in the chat
([Questions of the agent](questions.md)). The agent gets the output of a command that the user runs
with `▶▶` or allows ([Watched commands](terminal/terminal-watch.md)).

**When Claude is not available.** At the start, zendo does not open (ZENDO-2). While zendo runs, the
chat shows why the agent stopped, and zendo starts it again with the next question.

**Agents.** zendo works with Claude (the default) and with Codex (`zendo setup --agent-cmd codex`), each
through its pinned ACP adapter, and with any other ACP agent command. What an agent cannot do turns off
only its keys. With Codex, the hand-off to the command line (`ctrl-o`) starts `codex resume`
([Codex in its terminal](codex-cli.md)); the rest works the same.

The five threads of the chat, and how zendo keeps and resumes them, are in [Threads](threads.md).
Claude Code sessions that run in the background while the user works in zendo are in
[Sessions](sessions.md). The chat input, its chips and attachments are in [Chat](chat.md).

## Architecturally significant requirements

| ID       | Pattern  | Requirement                                                                                                                                 | Source                 |
| -------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------- |
| CLAUDE-1 | Event    | When zendo starts without `--offline`, the system shall start the agent and finish the ACP handshake and the login check before it opens the screen. | ZENDO-2           |

### Acceptance tests

**`start_check_fails_without_claude`** (mock agent, not written yet; verifies CLAUDE-1). See
[zendo](zendo.md).

The tests of the questions of the agent (ASK-1 to ASK-7) are in
[Questions of the agent](questions.md). `shell_is_blocked_and_resume_replays` in
`tests/acp_mock.rs` becomes `resume_replays_the_thread` and verifies CLAUDE-6 only.

## Detailed requirements

| ID        | Pattern    | Requirement                                                                                                                  | Source           |
| --------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| CLAUDE-4  | Unwanted   | If the agent stops while zendo runs, then the system shall show the reason in the chat and start the agent with the next question. | Design      |
| CLAUDE-9  | Event      | When the first question of a thread goes to Claude, the system shall send `.zendo/prompt.md`, if it exists. | Design |
| CLAUDE-26 | Event      | When a question goes to Claude and the open file did not go with one of the last 20 questions, the system shall attach the file and send the project rules for it, unless the user unselected `open document` (CLAUDE-55). | User, 2026-11-12; User, 2026-11-17 |
| CLAUDE-13 | Ubiquitous | The system shall work with Claude and with Codex through their pinned ACP adapters, and with any ACP agent command.         | User, 2026-11-11 |
| CLAUDE-14 | Unwanted   | If the agent cannot do a feature, then the system shall turn off only the keys and hints of that feature.                   | User, 2026-11-11 |
| CLAUDE-11 | Event      | When the user presses `esc` during an answer, the system shall stop the answer.                                              | Design           |

The requirements of the threads (CLAUDE-5, CLAUDE-6, CLAUDE-10, CLAUDE-27, CLAUDE-31, CLAUDE-32,
CLAUDE-35, CLAUDE-40) are in [Threads](threads.md).

Retired in revision 1: CLAUDE-2 (wait for the decision of the user on an edit), CLAUDE-7 (show a proposed edit inline in the note) and CLAUDE-8 (`e` opens a proposed edit in the editor), because zendo does not wait for the user on an edit (User, 2026-11-12). Retired in revision 6: CLAUDE-3 (reject each shell command without a question), CLAUDE-33 (allow each call that is not a shell command; the queue of [Chat](chat.md), which had the same ID, is now CLAUDE-66) and CLAUDE-12 (do not tell the agent that zendo runs commands): the user, 2026-11-17, wants each other call as a question in the chat, and a shell command in a terminal of zendo; ASK-1 and ASK-2 replace them. Retired identifiers are never reused.

Tests: `one_thread_for_every_file_and_the_file_goes_once` (CLAUDE-26) in `tests/chat_flows.rs`.
