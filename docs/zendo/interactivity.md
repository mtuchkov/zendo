---
status: "approved"
approved: "2026-11-18"
revision: "3"
date: "2026-11-18"
parent: "zendo.md"
prev: "live-status.md"
next: "background.md"
level: "what the user can do during a turn without Claude Code: steer the turn, open a step, take the next question, change the mode"
note: "Draft from the request of the user on 2026-11-18 in chat: a more interactive chat. The status line is in live-status.md. Revision 2: no queue during a turn; a key makes the agent read the steering now (User, 2026-11-18). Revision 3: `ctrl-s` stays, a click opens a step, the keys do not clash (User, 2026-11-18). Background processes and their stop are in background.md (User, 2026-11-18). Approved by the user on 2026-11-18."
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
in the background. `up` brings a waiting question back into the input, as for the history (HISTORY-5), and takes it out of the waiting questions. Where the agent cannot
steer, a question during a turn waits for the end of the turn, as now.

**Open a step.** In the transcript, a click on a step or `o` on it opens it under its line: the
input of the tool, its output, its diff, or its error; for a command, the last three lines of its
output (ACT-6). A click or `o` again closes it. A click on the file of a step opens the
file at its line in the note pane. `a` adds the result of the step to the question as a chip
(CLAUDE-29).

**The next question.** When a turn ends and the agent suggests the next question, the empty chat
input shows it in the dim color. `→` puts it into the input and does not send it. The first key
that types text removes the suggestion.

**Stop one task.** The stop of one background process is in [Background processes](background.md).

**The mode.** When the mode of the agent is not the default one, such as `plan` or `accept edits`,
the status line shows it. `shift-tab` in the chat input changes to the next mode, as in Claude Code.

## Architecturally significant requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| INTER-1  | Event   | When the user sends a question while a turn runs, where the agent can steer, the system shall give the question to the agent in the running turn when the step that runs now ends. | User, 2026-11-18 |
| INTER-2  | Event   | When the user opens a step, the system shall show the input, the output, the diff, or the error of the step under its line. | User, 2026-11-18 |
| INTER-3  | Event   | When a turn ends with a suggested next question, the system shall show it in the empty chat input in the dim color, and put it into the input on `→` without sending it. | User, 2026-11-18 |
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
| INTER-13 | Event      | When the turn ends while a question that steers it waits, the system shall send the question as the next question. | Design |
| INTER-14 | Event      | When the user brings a waiting question that steers the turn back into the input with `up` (HISTORY-5), the system shall take it out of the waiting questions. | Design |

Retired: INTER-4 and INTER-12 (the stop of one task on ACP v2 only), because BG-3 of [Background processes](background.md) covers them on every version (User, 2026-11-18).

## Open points

None. The user answered the open points on 2026-11-18.
