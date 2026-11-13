---
status: "draft"
revision: "1"
date: "2026-11-13"
parent: "zendo.md"
prev: "sessions.md"
next: "cli-messages.md"
level: "ctrl-o while the chat answers: the waiting session view, the stop, and the start of Claude Code"
note: "Agent draft from the decisions of the user on 2026-11-13 in chat. It replaces SESSIONS-15 of sessions.md. Not reviewed by the user as a document. The requirement IDs are proposals until the user reviews this document."
---

# Hand-off during a turn

The user can press `ctrl-o` while the chat of the active agent session answers. zendo does not
refuse. It shows the session view at once, and starts Claude Code when the turn ends. See
[Sessions](sessions.md) for the Claude Code sessions.

**One writer for a thread.** The agent and Claude Code write the same session. While the turn runs,
the agent writes it. Claude Code starts only after the turn ends, because `claude --resume` reads the
session once at start, and two writers can fork the history and edit the same files.

**The waiting view.** The session view takes the whole screen and shows the answer of the chat as
it streams. The bottom line says that Claude Code starts when the turn ends, and that `ctrl-o` again
stops the turn. zendo sends no key to a program in this view.

**Stop and continue.** `ctrl-o` twice within 5 s stops the turn. The bottom line shows
`stopping the turn…` until the turn ends. Then zendo starts Claude Code with a first prompt that
tells it to continue the stopped turn, so the user does not type it. Claude Code gets the prompt as
an argument of the command line, so no key is lost while Claude Code starts. If the turn does not
end within 5 s after the stop, zendo starts Claude Code anyway and says so.

**Leave.** `alt-q` leaves the waiting view. Then zendo does not start Claude Code when the turn ends.

## Architecturally significant requirements

| ID        | Pattern  | Requirement                                                                                                                                                        | Source           |
| --------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------- |
| HANDOFF-1 | Event    | When the user presses `ctrl-o` while the chat of the active agent session answers, the system shall show the session view at once, with the answer of the chat in it. | User, 2026-11-13 |
| HANDOFF-2 | State    | While the chat answers in the session view, the system shall not start the Claude Code session.                                                                     | User, 2026-11-13 |
| HANDOFF-3 | Event    | When the turn ends while the session view waits, the system shall start the Claude Code session in the session view.                                                 | User, 2026-11-13 |
| HANDOFF-4 | Event    | When the user presses `ctrl-o` again within 5 s while the session view waits, the system shall stop the turn.                                                       | User, 2026-11-13 |
| HANDOFF-5 | Event    | When a turn that the user stopped from the session view ends, the system shall start the Claude Code session with a first prompt to continue the stopped turn.       | User, 2026-11-13 |
| HANDOFF-6 | Unwanted | If the turn does not end within 5 s after the stop, then the system shall start the Claude Code session, and shall say that the turn did not stop.                   | User, 2026-11-13 |

### Acceptance tests

The tests are in `tests/handoff.rs`, with the mock agent and the mock command line of
[Sessions](sessions.md).

**`ctrl_o_during_a_turn_waits_for_its_end`** (mock agent and mock command line; verifies HANDOFF-1, HANDOFF-2, HANDOFF-3)

- Given a question that the chat answers.
- When the user presses `ctrl-o`.
- Then the session view shows at once, Claude Code does not run, and after the turn ends Claude Code
  runs in the session view without a first prompt.

**`ctrl_o_again_stops_the_turn_and_continues_in_claude_code`** (mock agent and mock command line; verifies HANDOFF-4, HANDOFF-5)

- Given a question that the chat answers.
- When the user presses `ctrl-o` twice.
- Then the bottom line shows that the turn stops, and after the turn ends Claude Code runs with the
  prompt to continue.

**`claude_code_starts_if_the_turn_does_not_stop`** (mock agent and mock command line; verifies HANDOFF-6)

- Given a stopped turn that does not end.
- When 5 s go by.
- Then Claude Code runs with the prompt to continue, and zendo says that the turn did not stop.

## Detailed requirements

| ID        | Pattern | Requirement                                                                                                                                         | Source           |
| --------- | ------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| HANDOFF-7 | State   | While the session view waits, the system shall show on its bottom line that Claude Code starts when the turn ends, and that `ctrl-o` again stops the turn. | User, 2026-11-13 |
| HANDOFF-8 | State   | While the session view waits after a stop, the system shall show `stopping the turn…` on its bottom line.                                            | User, 2026-11-13 |
| HANDOFF-9 | State   | While the session view waits, the system shall keep `alt-q`, `F5` to `F10` and `alt-n`, and shall ignore all other keys and paste.                     | User, 2026-11-13 |
| HANDOFF-10 | Event  | When the user leaves the session view while it waits, the system shall not start the Claude Code session when the turn ends.                         | Design           |
| HANDOFF-11 | Event  | When the turn ends while the session view waits, the system shall put the queued questions of the chat into the chat input, and shall not send them to the agent. | Design |

Tests of the detailed requirements: `keys_in_the_waiting_view` (HANDOFF-9, HANDOFF-10) and
`queued_questions_go_to_the_input` (HANDOFF-11) in `tests/handoff.rs`.
