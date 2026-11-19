---
status: "draft"
revision: "1"
date: "2026-11-17"
parent: "zendo.md"
prev: "claude.md"
next: "threads.md"
level: "the questions of the agent before a tool call: which ones zendo answers, the quick action in the chat, its keys, a shell command in a terminal of zendo"
note: "Agent draft from the requests of the user on 2026-11-17. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document. Replaces CLAUDE-3 and CLAUDE-33. See ADR 0019. ASK-14 (a mark on a command that ran without a question) is withdrawn: the user does not want it (2026-11-17); the ID is not reused. The answers of the user on 2026-11-18: `up` goes to the options first (ASK-5), and not for Claude Code in a terminal: ASK-16 is withdrawn, and the ID is not reused."
---

# Questions of the agent

Claude Code and Codex allow reads and searches themselves, and ask before a shell command, a web
request, or work outside the workspace. Through ACP the question comes to zendo
([Claude](claude.md)). zendo answers only edits itself, as `acceptEdits` does, and gives each
other question to the user ([ADR 0019](../architecture/decisions/0019-questions-of-the-agent.md)).

**What zendo allows itself.** An edit of a file within the limits of the task (STRUCT-3,
NOTESREP-4), and a tool of the MCP server `zendo`, shown as one tool line. Each other question goes
to the user: a shell command, `Monitor`, a web fetch, a read outside the workspace, a tool of
another MCP server, a tool that zendo does not know.

**The quick action.** zendo shows the question at the end of the transcript of the thread: the
tool, and what it will do: the full command, or the input of the tool. The options stand under it
in one row, as for a question of the agent with choices (ACT-4 in [Activity](activity.md)), and the
status line shows `waits for your answer` (ACT-5).

- For a shell command: `Run in terminal`, `No`, `Say what to do instead`.
- For another tool: the options of the agent, for example `Allow once` and `No`, and
  `Say what to do instead`.

**The keys.** While a question waits, `up` on the first line of the chat input moves to the
options. `left` and `right` choose, `enter` answers, `esc` or `down` go back to the input, which
keeps its text. A click on an option answers too. Questions come one at a time, in their order.

**The mode of the agent.** zendo keeps the permission mode, also `auto`. A command that the
classifier of `auto` or an `allow` rule of the user allows runs in Claude Code, as a usual tool
line.

**A shell command runs in a terminal of zendo.** On `Run in terminal`, zendo runs the command in a
terminal of the session, as `▶▶` does ([Watched commands](terminal/terminal-watch.md), TERM-73).
With Claude, a hook of zendo replaces the command of the agent with a wait for the result, so the
agent gets the output as a normal shell result
([ADR 0020](../architecture/decisions/0020-answers-to-claude-through-a-hook.md)). Another agent
gets *no*, then the output by push and `command_result`.

**No, or something else.** Both answer *no*. `Say what to do instead` also puts the cursor into the
chat input.

**Without a user.** zendo answers *no* to the questions of a recap and other work of zendo
([Work](work.md)).

**A Claude Code session.** Claude Code asks in its chat through the hook of its permission request
(CLIACT-10 in [Activity of a Claude Code session](cli-activity.md)), with the choices of Claude
Code. A command there runs in Claude Code, never in a terminal of zendo.

## Architecturally significant requirements

| ID    | Pattern    | Requirement | Source |
| ----- | ---------- | ----------- | ------ |
| ASK-1 | Event      | When the agent asks for an edit of a file within the limits of the task, or for a tool of the MCP server `zendo`, the system shall allow it at once and show it in the chat as one line with the tool and the file. | User, 2026-11-12; User, 2026-11-17 |
| ASK-2 | Unwanted   | If the agent of a thread asks for any other tool call, also of a kind that zendo does not know, then the system shall show the question in the chat of that thread with the options of the agent, and answer it only with the choice of the user. | User, 2026-11-17 |
| ASK-3 | Ubiquitous | The system shall show in each question the full command or the full input of the tool, not only its title. | Design |
| ASK-4 | Event      | When the user answers `Run in terminal` to a shell command, the system shall run the command in a terminal of zendo and watch it for that session, as TERM-55 does, and give its output to the agent as the result of its shell command where the agent allows it. | User, 2026-11-17 |
| ASK-5 | State      | While a question waits, the system shall move to its options when the user presses `up` on the first line of the chat input, before `up` brings a waiting question back (HISTORY-5, INTER-14). | User, 2026-11-17; User, 2026-11-18 |
| ASK-6 | Event      | When the user answers `Say what to do instead`, the system shall answer *no* to the agent and move the cursor into the chat input. | User, 2026-11-17 |
| ASK-7 | Unwanted   | If a question comes from work of zendo without a user, then the system shall answer *no* to each call that ASK-1 does not allow. | Design |

### Acceptance tests

The tests are in `tests/questions.rs`. They use the mock agent, and `bash` as the shell.

**`an_edit_needs_no_question_and_a_read_does`** (mock agent; verifies ASK-1, ASK-2)

- Given a thread with the mock agent.
- When the agent asks for an edit of a note, then for a read of a file outside the repository.
- Then the edit runs at once with one tool line, and the read waits for the user as a question.

**`monitor_and_an_unknown_tool_go_to_the_user`** (mock agent; verifies ASK-2, ASK-3)

- Given a thread with the mock agent.
- When the agent asks for `Monitor` with the command `touch ran`, with the kind `other`.
- Then the chat shows the question with `touch ran` and its options, `ran` does not exist, and the
  agent waits. The same for a tool `mcp__other__x` without a kind.

**`run_in_terminal_runs_the_command_and_the_agent_gets_the_output`** (mock agent; verifies ASK-4)

- Given a question of the agent for the shell command `echo zendo-$((40+2))`.
- When the user answers `Run in terminal`.
- Then the `F5` terminal shows `zendo-42`, and the agent gets `zendo-42` as the result of its
  shell command; with a mock agent that is not Claude, it gets *no* and the result by push
  (TERM-56).

**`up_moves_to_the_options`** (mock agent; verifies ASK-5)

- Given a question that waits, and the text `why` in the chat input.
- When the user presses `up`, `right` and `enter`.
- Then the second option answers the question, and the input still holds `why`.

**`say_what_to_do_instead_refuses_and_moves_to_the_input`** (mock agent; verifies ASK-6)

- Given a question that waits.
- When the user answers `Say what to do instead`.
- Then the agent got *no*, and the cursor is in the chat input.

**`work_of_zendo_gets_no_question`** (mock agent; verifies ASK-7)

- Given a recap that runs with the mock agent.
- When the agent of the recap asks for a shell command.
- Then the agent got *no*, and the chat shows no question.

## Detailed requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| ASK-8  | State      | While the options have the focus, the system shall move between them with `left` and `right`, answer with `enter`, and go back to the input with `esc` or `down`. | User, 2026-11-17 |
| ASK-9  | Event      | When the user clicks an option, the system shall answer with it. | Design |
| ASK-10 | Ubiquitous | The system shall show one question at a time for each thread, in the order of the requests. | Design |
| ASK-11 | Event      | When the user answers `No` to a shell command, the system shall answer *no* to the agent, and `command_result` shall report the command as refused. | Design |
| ASK-12 | Event      | When the user stops the answer while a question waits, the system shall remove the question and cancel the request. | Design; CLAUDE-11 |
| ASK-13 | Ubiquitous | The system shall take a shell command that the agent wants to run in the background as any other shell command. | Design |
| ASK-15 | Ubiquitous | The system shall not change the permission mode of the agent. | User, 2026-11-17 |

