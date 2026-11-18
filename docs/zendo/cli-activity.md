---
status: "draft"
revision: "2"
date: "2026-11-18"
parent: "zendo.md"
prev: "cli-messages.md"
next: "back-to-the-agent.md"
level: "what the chat of a running Claude Code session shows about its turn, from the hooks of that run, and what the user can do there"
note: "Agent draft from the answer of the user on 2026-11-18 in chat: the chat of a Claude Code session shall show the same detail as the chat of the agent. Revision 2: the user answers questions and permissions of Claude Code in the chat; zendo needs the latest Claude Code (User, 2026-11-18). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Activity of a Claude Code session

While a Claude Code session runs ([Sessions](sessions.md)), its chat shows the same activity as
the chat of the agent: the status line, the steps, the checklist, the questions and the commands
([Activity of the agent](activity.md), [A status line that is alive](live-status.md)). The source
is not ACP. It is the hooks of that run (SESSIONS-18) and, for the text and the thought, the
transcript of Claude Code ([Claude Code messages in the chat](cli-messages.md)). What no hook and
no line of the transcript tells, the chat does not show. zendo does not read the screen of Claude
Code for it.

**What shows.**

- The step that runs now and its time come from the hooks before and after each tool. A failed
  tool comes from the hook of a failure (ACT-8).
- A subagent is one step, from the hooks of its start and its stop (ACT-13).
- The checklist comes from the to-do and task tools of Claude Code (ACT-3).
- `compacting context…` shows between the hooks before and after a compaction (LIVE-2).
- The last line of the thought shows in the status line, when the transcript has its text (LIVE-1).
- `no news for 1 min` counts from the last hook or the last line of the transcript (ACT-2).
- `waits for your answer` shows while Claude Code waits for a permission or a question, with the
  question in the chat (ACT-5).
- A turn that ends with an error, such as a rate limit, shows its reason.
- The mode of Claude Code shows in the status line, from the input of the hooks.

**Commands.** The hooks tell about a command only when it starts and when it ends. A command step
shows `running` until it ends. When the user opens the step, it shows the last three lines of the
output and the exit code from the hook after the tool (ACT-6).

**Answers in the chat.** When Claude Code asks for a permission or asks a question, the chat shows
it with its choices, as for the agent (ACT-4). The user answers in the chat, and the answer goes
back to Claude Code through the hook of the permission request (User, 2026-11-18). The hook waits
for the answer of the user. When the user answers in the session view (`ctrl-o`) first, the
question in the chat goes away.

**Version.** zendo needs the latest version of Claude Code at the date of its release (User,
2026-11-18). With an older version, what its hooks do not give does not show (CLIACT-2).

**What the user can do.** A question that the user sends during a turn goes into Claude Code
(SESSIONS-4), and Claude Code takes it into the running turn itself: this is the steering of a
Claude Code session. `shift-tab` in the chat input goes to Claude Code and changes its mode. A
click or `o` opens a step (INTER-2). `steer_now` and the next question are off for a Claude Code
session, because its hooks do not give them (CLAUDE-14). Its background processes show and stop as
for the agent ([Background processes](background.md)).

## Architecturally significant requirements

| ID        | Pattern  | Requirement | Source |
| --------- | -------- | ----------- | ------ |
| CLIACT-1  | State    | While a turn of a Claude Code session runs, the system shall show in its chat the status line, the steps, the subagents and the checklist (ACT-1, ACT-3, ACT-8, ACT-13), from the hooks of that run. | User, 2026-11-18 |
| CLIACT-2  | Unwanted | If no hook and no line of the transcript tells a part of the activity, then the system shall not show that part, and shall not read the screen of Claude Code for it. | Design |
| CLIACT-3  | State    | While Claude Code waits for a permission or a question, the system shall show `waits for your answer` in the status line of its chat, and the question in the chat. | User, 2026-11-18 |
| CLIACT-10 | Event   | When the user answers a permission or a question of Claude Code in its chat, the system shall send the answer to Claude Code through the hook of the permission request. | User, 2026-11-18 |

### Acceptance tests

Tests: `tests/cli_activity.rs`, with the mock command line of [Sessions](sessions.md).

**`the_chat_of_a_session_shows_its_steps`** (mock command line; verifies CLIACT-1)

- Given a mock command line that runs the hooks before and after a read of `docs/a.md`, and a to-do
  list of two items.
- When the hooks run.
- Then the status line of the chat shows `reading docs/a.md`, and then the checklist `plan 0/2`.

**`nothing_shows_without_a_hook`** (mock command line; verifies CLIACT-2)

- Given a mock command line that prints to its screen and runs no hook for 61 s.
- Then the chat shows `no news for 1 min`, and no text of the screen.

**`a_waiting_session_asks_in_the_chat`** (mock command line; verifies CLIACT-3, CLIACT-10)

- Given a mock command line that runs the hook of a permission request with the choices `allow`
  and `deny`.
- When the user selects `deny` in the chat and presses `enter`.
- Then the status line of the chat showed `waits for your answer`, and the hook returned `deny`.

## Detailed requirements

| ID        | Pattern    | Requirement | Source |
| --------- | ---------- | ----------- | ------ |
| CLIACT-4  | Event      | When the user opens the step of a command that ended, the system shall show the last three lines of its output and its exit code, from the hook after the tool. | User, 2026-11-18 |
| CLIACT-5  | State      | While Claude Code compacts the context, the system shall show `compacting context…` in the status line of its chat. | Design |
| CLIACT-6  | State      | While Claude Code thinks and the transcript has the text of the thought, the system shall show the last line of the thought in the status line of its chat. | Design |
| CLIACT-7  | Event      | When a turn of Claude Code ends with an error, the system shall show the reason in its chat. | Design |
| CLIACT-8  | Event      | When the user presses `shift-tab` in the chat input of a Claude Code session, the system shall send it to Claude Code. | Design |
| CLIACT-9  | Ubiquitous | The system shall turn off `steer_now` and the next question in the chat of a Claude Code session. | Design |
| CLIACT-11 | Event     | When the user answers a question of Claude Code in the session view first, the system shall remove the question from its chat. | Design |
| CLIACT-12 | Ubiquitous | The system shall work with the latest version of Claude Code at the date of the release of zendo. | User, 2026-11-18 |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| Can zendo make Claude Code read a question now, as `steer_now` does for the agent? | CLIACT-9 | Architect |
