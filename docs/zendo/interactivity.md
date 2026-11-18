---
status: "draft"
revision: "3"
date: "2026-11-18"
parent: "zendo.md"
prev: "live-status.md"
next: "terminal/terminal.md"
level: "what the user can do during a turn without Claude Code: steer the turn, open a step, take the next question, stop one task, change the mode"
note: "Agent draft from the request of the user on 2026-11-18 in chat: a more interactive chat. The status line is in live-status.md. Revision 2: no queue during a turn; a key makes the agent read the steering now (User, 2026-11-18). Revision 3: `ctrl-s` stays, a click opens a step, the keys do not clash (User, 2026-11-18). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Interactivity of the chat

[Activity of the agent](activity.md) shows what the agent does, and
[A status line that is alive](live-status.md) shows why it is quiet. This document lets the user
act on the turn without Claude Code. Some of it uses extensions of the Claude adapter, not of ACP.
Where an agent does not have a feature, only its keys turn off (CLAUDE-14).

**Steer the turn.** While a turn runs, a question that the user sends with `enter` steers the turn:
there is no queue (User, 2026-11-18). zendo gives it to the agent when the step that runs now ends,
so the agent does not lose its work. The transcript shows it at its place with `↪ next step`. When
the turn ends first, it goes as the next question. The new action `steer_now` (`ctrl-s`; `ctrl-x`
is cut, and the keymap has no key pairs) gives the waiting questions and the text of the input to
the agent at once: the agent stops what it writes and reads them, and a command that runs goes on
in the background. `↑` in the empty input takes a waiting question back. Where the agent cannot
steer, a question during a turn waits for the end of the turn, as now.

**Open a step.** In the transcript, a click on a step or `o` on it opens it under its line: the
input of the tool, its output, its diff, or its error; for a command, the last three lines of its
output (ACT-6). A click or `o` again closes it. A click on the file of a step opens the
file at its line in the note pane. `a` adds the result of the step to the question as a chip
(CLAUDE-29).

**The next question.** When a turn ends and the agent suggests the next question, the empty chat
input shows it in the dim color. `→` puts it into the input and does not send it. The first key
that types text removes the suggestion.

**Stop one task.** `x` on the step of a command or a monitor that runs in the background stops only
that task; the turn goes on. A click on `×` at its name in the `background` line (ACT-12) does the
same.

**The mode.** When the mode of the agent is not the default one, such as `plan` or `accept edits`,
the status line shows it. `shift-tab` in the chat input changes to the next mode, as in Claude Code.

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| INTER-1  | Event   | When the user sends a question while a turn runs, where the agent can steer, the system shall give the question to the agent in the running turn when the step that runs now ends. | User, 2026-11-18 |
| INTER-2  | Event   | When the user opens a step, the system shall show the input, the output, the diff, or the error of the step under its line. | User, 2026-11-18 |
| INTER-3  | Event   | When a turn ends with a suggested next question, the system shall show it in the empty chat input in the dim color, and put it into the input on `→` without sending it. | User, 2026-11-18 |
| INTER-4  | Event   | When the user stops a task that runs in the background, the system shall stop only that task and let the turn go on. | User, 2026-11-18 |
| INTER-5  | Event   | When the user presses `shift-tab` in the chat input, the system shall change the agent to its next mode and show the mode in the status line. | User, 2026-11-18 |

### Acceptance tests

Tests: `tests/interactivity.rs`, with the mock agent.

**`a_question_during_a_turn_steers_it`** (mock agent; verifies INTER-1)

- Given a running turn of a mock agent that can steer, with a step that runs.
- When the user types `use the tests` and presses `enter`, and later types `stop` and presses
  `ctrl-s`.
- Then the agent gets `use the tests` in the running turn when the step ends, and `stop` at once;
  the transcript shows each with `↪`.

**`a_step_opens_and_closes`** (mock agent; verifies INTER-2)

- Given a done step of the mock agent with an output of three lines.
- When the user moves the cursor to the step and presses `o`, and `o` again.
- Then the three lines show under the step, and then they do not.

**`the_next_question_fills_the_input`** (mock agent; verifies INTER-3)

- Given a turn that ends with the suggestion `run the tests`.
- When the user presses `→`.
- Then the input holds `run the tests`, and nothing went to the agent.

**`one_task_stops_and_the_turn_goes_on`** (mock agent; verifies INTER-4)

- Given a running turn with one command in the background.
- When the user presses `x` on its step.
- Then the agent gets a stop for that task only, and the turn still runs.

**`shift_tab_changes_the_mode`** (mock agent; verifies INTER-5)

- Given a mock agent with the modes `default` and `plan`.
- When the user presses `shift-tab` in the chat input.
- Then the agent gets the mode `plan`, and the status line shows `plan`.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| INTER-6  | Ubiquitous | The system shall show a question that steers a turn at the place where it went, with `↪ next step` while it waits and `↪` after the agent got it. | Design |
| INTER-7  | Event      | When the user presses `steer_now` while a turn runs, the system shall give the waiting questions and the text of the input to the agent at once, so that the agent reads them before it goes on. | User, 2026-11-18 |
| INTER-8  | Unwanted   | If the agent cannot steer, then the system shall keep a question that the user sends during a turn until the turn ends, and then send it. | Design |
| INTER-9  | Event      | When the user clicks the file of a step, the system shall open the file at its line in the note pane. | Design |
| INTER-10 | Event      | When the user presses `a` on a step, the system shall add the result of the step to the question as a chip. | Design |
| INTER-11 | Event      | When the user types text into the input that shows a suggestion, the system shall remove the suggestion. | Design |
| INTER-12 | Event      | When the user clicks `×` at the name of a task in the `background` line, the system shall stop that task (INTER-4). | Design |
| INTER-13 | Event      | When the turn ends while a question that steers it waits, the system shall send the question as the next question. | Design |
| INTER-14 | Event      | When the user presses `↑` in the empty chat input while a question that steers the turn waits, the system shall take the last waiting question back into the input. | Design |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| Steering and the suggestion are extensions of the Claude adapter; [ADR 0012](../architecture/decisions/0012-agent-activity-from-acp-and-hooks.md) (draft) uses them. Does the user approve the record? | INTER-1, INTER-3 | User |
| The adapter gives the life of a background task, and its stop, only to a client that declares the `asyncTasks` capability of JetBrains AIR. Declare it, or ask the adapter to give it to every client ([ADR 0012](../architecture/decisions/0012-agent-activity-from-acp-and-hooks.md))? Until then INTER-4 and INTER-12 are off. | INTER-4, INTER-12 | User |
