---
status: "draft"
revision: "2"
date: "2026-11-14"
parent: "zendo.md"
prev: "handoff.md"
next: "zendo-session.md"
level: "the messages of a running Claude Code session in the chat of its agent session: where they come from, what shows, when they go"
note: "Agent draft from the decision of the user on 2026-11-13 in chat. Not reviewed by the user as a document. The requirement IDs are proposals until the user reviews this document."
---

# Claude Code messages in the chat

While a Claude Code session runs (see [Sessions](sessions.md)), the user can go back to zendo with
`alt-q`. The chat of its agent session then shows what happened in Claude Code: each prompt, the
steps of each turn, and its final answer. The steps are the text that Claude writes between the
tool calls, and the tool calls, each as one line as for the tools of the agent. The thoughts, the
tool results and the work of a subagent do not show.

**Source.** zendo starts Claude Code with hooks of that run only (SESSIONS-18). The hooks of the
events "prompt sent", "stop" and "session start" append their input to a file of the session in
`.zendo/sessions/`. The input of "prompt sent" has the prompt; the input of "stop" has the last
answer of the turn. zendo reads the new lines on each tick.

**The transcript.** No hook has the text between the tool calls. Each hook input names the
transcript file of the conversation, and zendo reads the steps from its new lines. The format of
that file is not a public interface of Claude Code: zendo takes from it only the text and the tool
calls of the assistant, and a line that it does not understand gives nothing. The prompts and the
final answer still come from the hooks, so they show also when the format changes. The transcript
has the final answer too; it shows one time.

**Clear.** `/clear` in Claude Code starts a new conversation with a new transcript. The next hook
names it, and zendo reads the new one from its start. A resumed session is read from its end, so
its history does not show again.

**No echo.** A question that the user sends from the chat already shows in the chat. When the hook
reports the same prompt, zendo does not show it again. The hooks print nothing: Claude Code adds
the output of a "prompt sent" hook to the context of Claude.

**End.** When the Claude Code session ends, zendo opens the agent session again (SESSIONS-13). The
history that the agent loads then replaces these messages.

## Architecturally significant requirements

| ID       | Pattern    | Requirement                                                                                                                                      | Source           |
| -------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------- |
| CLIMSG-1 | State      | While a Claude Code session runs, the system shall show in the chat of its agent session each prompt that Claude Code gets, the steps of each turn (CLIMSG-8, CLIMSG-9), and the final answer of each turn one time. | User, 2026-11-13; User, 2026-11-14 |
| CLIMSG-2 | Ubiquitous | The system shall take the prompts and the final answers from the hooks that zendo gives to the run of Claude Code, and only the steps of a turn from the transcript file that the hooks name. | Design; SESSIONS-18; User, 2026-11-14 |

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
| CLIMSG-8 | State    | While a turn of Claude Code runs, the system shall show each text that Claude writes between the tool calls, in its order, and shall not show the thoughts, the tool results and the work of a subagent. | User, 2026-11-14 |
| CLIMSG-9 | State    | While a turn of Claude Code runs, the system shall show each tool call as the chat shows a tool of the agent: one line with the tool and what it is about. | User, 2026-11-14 |
| CLIMSG-10 | Event   | When the hooks name another transcript file, as after `/clear`, the system shall read the steps from that file: from its start for a new conversation, and from its end for a resumed one. | User, 2026-11-14 |
| CLIMSG-11 | Unwanted | If a line of the transcript file is not in the format that zendo knows, then the system shall show nothing for that line and shall still show the prompts and the final answers. | Design |

Tests of the detailed requirements: `a_question_from_the_chat_shows_once` (CLIMSG-4) and
`the_end_of_the_session_replaces_the_messages` (CLIMSG-7) in `tests/cli_messages.rs`; unit tests of
the message file in `src/session/messages_tests.rs` (CLIMSG-6, CLIMSG-8 to CLIMSG-11);
`the_steps_of_a_turn_show_in_the_chat` (CLIMSG-8, CLIMSG-9) and `the_steps_show_after_a_clear`
(CLIMSG-10) in `tests/cli_messages.rs`.

This revision changed CLIMSG-1 and CLIMSG-2 and added CLIMSG-8 to CLIMSG-11: the chat shows the steps of
a turn, which zendo reads from the transcript file.
