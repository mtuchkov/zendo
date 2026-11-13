---
status: "draft"
revision: "1"
date: "2026-11-13"
parent: "zendo.md"
prev: "handoff.md"
next: "chat.md"
level: "the messages of a running Claude Code session in the chat of its agent session: where they come from, what shows, when they go"
note: "Agent draft from the decision of the user on 2026-11-13 in chat. Not reviewed by the user as a document. The requirement IDs are proposals until the user reviews this document."
---

# Claude Code messages in the chat

While a Claude Code session runs (see [Sessions](sessions.md)), the user can go back to zendo with
`alt-q`. The chat of its agent session then shows what happened in Claude Code: each prompt, and
the final answer of each turn. Tool calls and the text between them do not show.

**Source.** zendo starts Claude Code with hooks of that run only (SESSIONS-18). The hooks of the
events "prompt sent" and "stop" append their input to a file of the session in `.zendo/sessions/`.
The input of "prompt sent" has the prompt; the input of "stop" has the last answer of the turn.
zendo reads the new lines on each tick. zendo does not read the transcript file of Claude Code: its
format is not a public interface, and Claude Code writes it with a delay.

**No echo.** A question that the user sends from the chat already shows in the chat. When the hook
reports the same prompt, zendo does not show it again. The hooks print nothing: Claude Code adds
the output of a "prompt sent" hook to the context of Claude.

**End.** When the Claude Code session ends, zendo opens the agent session again (SESSIONS-13). The
history that the agent loads then replaces these messages.

## Architecturally significant requirements

| ID       | Pattern    | Requirement                                                                                                                                      | Source           |
| -------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------- |
| CLIMSG-1 | State      | While a Claude Code session runs, the system shall show in the chat of its agent session each prompt that Claude Code gets, and the final answer of each turn. | User, 2026-11-13 |
| CLIMSG-2 | Ubiquitous | The system shall take the prompts and the answers from the hooks that zendo gives to the run of Claude Code, not from the transcript file of Claude Code. | Design; SESSIONS-18 |

### Acceptance tests

The tests are in `tests/cli_messages.rs`, with the mock command line of [Sessions](sessions.md). On
`enter`, the mock writes the inputs of the "prompt sent" and "stop" hooks to the file of the session.

**`claude_code_messages_show_in_the_chat`** (mock command line; verifies CLIMSG-1, CLIMSG-2, CLIMSG-3)

- Given a running Claude Code session on the full screen.
- When the user types a prompt in Claude Code, presses `enter`, and goes back with `alt-q`.
- Then the chat of the agent session shows the prompt and the answer.

**`hooks_print_nothing_and_append_one_line`** (unit test; verifies CLIMSG-2, CLIMSG-5)

- Given the settings with the hooks.
- When the hook command of "prompt sent" gets input on several lines.
- Then it prints nothing, and the file gets one line with that input.

## Detailed requirements

| ID       | Pattern  | Requirement                                                                                                                                | Source           |
| -------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------ | ---------------- |
| CLIMSG-3 | Event    | When the user goes back to the chat from the session view, the system shall show the messages that came while the user was in the session view. | User, 2026-11-13 |
| CLIMSG-4 | Unwanted | If a prompt is a question that the user sent from the chat, then the system shall not show it again.                                        | Design           |
| CLIMSG-5 | Ubiquitous | The hooks that report messages shall print nothing.                                                                                      | Design           |
| CLIMSG-6 | Unwanted | If the input of a hook has no text of the answer, then the system shall show no answer for that turn.                                       | Design           |
| CLIMSG-7 | Event    | When a Claude Code session ends, the system shall replace its messages in the chat with the history that the agent loads.                    | Design; SESSIONS-13 |

Tests of the detailed requirements: `a_question_from_the_chat_shows_once` (CLIMSG-4) and
`the_end_of_the_session_replaces_the_messages` (CLIMSG-7) in `tests/cli_messages.rs`; unit tests of
the message file in `src/session/messages_tests.rs` (CLIMSG-6).
