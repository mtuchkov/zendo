---
status: "approved"
approved: "2026-11-18"
revision: "4"
date: "2026-11-18"
parent: "zendo.md"
prev: "chat-history.md"
next: "checklist.md"
level: "what the chat shows about the work of the agent during a turn: the status line, the steps, the questions, the commands, the signs of life"
note: "Agent draft from the request of the user on 2026-11-18 in chat: the chat shall show the current state of the agent, so that the user is calm and does not switch to Claude Code to see it. Revision 2: the answers of the user on 2026-11-18 (questions on, allowed commands run and show). Revision 3: `no news` after 1 min, the output of a command only in an open step, the same detail for a Claude Code session (User, 2026-11-18). Revision 4: the checklist moved to checklist.md, to keep the document within two pages (User, 2026-11-18). Background processes are in background.md (User, 2026-11-18). Approved by the user on 2026-11-18."
---

# Activity of the agent

During a turn, the chat shows what the agent does now, what it did, and what it plans, so that the
user does not open Claude Code (`ctrl-o`) only to see that the agent works. The activity comes from
the agent through ACP ([ADR 0001](../architecture/decisions/0001-claude-through-acp.md)). What an
agent does not tell, the chat does not show, and nothing else changes (CLAUDE-14). The time of each
message is in [Transcript](transcript.md); what the user can do during a turn is in
[Interactivity of the chat](interactivity.md).

**The status line.** One line under the last message of the turn shows the step that runs now,
such as `reading src/app/cli.rs · 3 s`, or `thinking · 12 s`, or `waits for your answer`. A long
step and a stuck turn look the same when nothing moves, so after 1 min without an update the line
adds `no news for 1 min` and counts up (User, 2026-11-18).

**Steps.** Each tool call shows as one line, as now (CLAUDE-33, CLIMSG-9), with its state (running,
done, failed) and its time. A failed step shows the first line of its error. A subagent is one step
with its description and the step that it runs now. The plan of the agent is a checklist at the
top of the transcript ([Checklist](checklist.md)).

**Questions.** A question of the agent with choices shows in the transcript, and the turn waits.
The user selects a choice with the arrow keys and `enter`, or types an own answer; `esc` stops the
turn (CLAUDE-11). zendo still allows each permission request at once (CLAUDE-33): only questions
wait for the user.

**Commands and monitors.** zendo rejects each shell command that asks for permission (CLAUDE-3).
A command that the settings of the user allow runs without a question, and so does a monitor: the
user allowed it, so zendo does not block it (User, 2026-11-18).
Its step shows the command and its exit code. The last three lines of its output show only when the
user opens the step ([Interactivity of the chat](interactivity.md)), and change while it runs.
A command that goes on in the background shows in the background line ([Background processes](background.md)).

The chat of a running Claude Code session shows the same activity from its hooks
([Activity of a Claude Code session](cli-activity.md)).

## Architecturally significant requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| ACT-1  | State      | While a turn runs, the system shall show under the last message of the turn one status line with the step that runs now, or `thinking`, and the time of the turn. | User, 2026-11-18 |
| ACT-2  | Unwanted   | If no update came from the agent for 1 min during a turn, then the system shall show in the status line the time since the last update, until the next update comes. | User, 2026-11-18 |
| ACT-4  | Event      | When the agent asks the user a question with choices, the system shall show the question and its choices in the chat, and send the choice or the own answer of the user to the agent. | User, 2026-11-18 |
| ACT-5  | State      | While a question of the agent waits for the user, the system shall show `waits for your answer` in the status line. | User, 2026-11-18 |
| ACT-6  | Event      | When a command or a monitor of the agent runs, also one that the settings of the user allow without a question, the system shall show its exit code in its step when it ends, and its last three lines of output while the user has the step open. | User, 2026-11-18 |
| ACT-7  | Ubiquitous | The system shall take the activity of the agent of the chat only from the agent connection, and shall not read the files of Claude Code for it. | Design |

### Acceptance tests

Tests: `tests/activity.rs`, with the mock agent.

**`the_status_line_names_the_step_that_runs`** (mock agent; verifies ACT-1, ACT-7)

- Given a turn of the mock agent with one tool call that reads `docs/a.md`.
- When the tool call starts, and later ends.
- Then the status line shows `reading docs/a.md` and its time, and then `thinking`.

**`a_quiet_turn_shows_the_time_since_the_last_update`** (mock agent; verifies ACT-2)

- Given a turn of the mock agent without an update for 61 s.
- Then the status line shows `no news for 1 min`, and the next update removes it.

**`a_question_waits_and_the_answer_goes_to_the_agent`** (mock agent; verifies ACT-4, ACT-5)

- Given a question of the mock agent with the choices `yes` and `no`.
- When the user selects `no` and presses `enter`.
- Then the status line shows `waits for your answer` until `enter`, and the agent gets `no`.

**`a_command_shows_its_output_and_exit_code`** (mock agent; verifies ACT-6)

- Given a command of the mock agent that prints five lines and exits with code 1.
- When the command runs, the user opens its step, and later the command ends.
- Then the step shows no output until it is open, then the last three lines, and `exit 1` after
  it ends.

## Detailed requirements

| ID     | Pattern  | Requirement | Source |
| ------ | -------- | ----------- | ------ |
| ACT-8  | Event    | When a step fails, the system shall mark it as failed and show the first line of its error. | Design |
| ACT-9  | Optional | Where the agent tells how full the context of the thread is, the system shall show it in the status line as a percentage. | Design |
| ACT-11 | Event    | When a question of the agent waits and the user does not see its session, the system shall ring and blink as at the end of a turn (ATTN-1, ATTN-2). | Design |
| ACT-13 | Ubiquitous | The system shall show a subagent as one step with its description, the step that it runs now, and its time. | Design |

Retired: ACT-12 (the background line on ACP v2 only), because BG-1 of [Background processes](background.md) covers it on every version (User, 2026-11-18).

## Open points

None. The user answered the open points on 2026-11-18.
