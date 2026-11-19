---
status: "implemented"
approved: "2026-11-19"
revision: "5"
date: "2026-11-19"
parent: "zendo.md"
level: "what zendo measures about the responsiveness of each action, and where the measurement lives in the code"
note: "Agent draft of the note of the user of 2026-11-19 (measuring responsiveness of zendo actions), approved by the user on 2026-11-19 with RESP-1 to RESP-7. Revision 2: implemented in zendo 0.121.0; the test names and kinds as in the code; the mechanism in ADR 0024. Revision 3, from the decisions of the user of 2026-11-19: the time spans the wait for the result, also for work in the background (RESP-1); `stats = false` turns the measurements off (RESP-8); approved by the user on 2026-11-19. Revision 4, after the code review of 2026-11-19: each piece of work in the background has its own identity, each input waits only for its own work, the time starts at the read of the event, and an action whose result has not come at the exit gets no time (User, 2026-11-19); implemented in zendo 0.123.2. Revision 5, from the decisions of the user of 2026-11-19, approved by the user on 2026-11-19 and implemented in zendo 0.124.0: the time of each action goes into its event of the telemetry log (telemetry.md), which takes the place of the file of the measurements; the requirements of the file move there."
---

# Responsiveness of actions

Sources: the note of the user of 2026-11-19, "Measuring responsiveness of Zendo actions", and the
decisions of the user in chat on 2026-11-19.

Parent: [zendo](zendo.md). The actions and their counts are in [Usage counts](usage.md); the log
that keeps the times is in [Telemetry log](telemetry.md).

## What zendo measures

zendo measures how long the user waits for the result of each action. An action is an entry of the
keymap, as in [Usage counts](usage.md): `copy`, `add_to_chat`, `send`, the change of a pane, and
each other action that has a name there.

The time of an action starts when zendo reads the key or the mouse event, and ends when the screen
that shows the result of the action is drawn. For an action that starts work in the background,
such as a search or a turn of the agent, the time ends at the first screen after the end of that
work, not at the first screen that shows that the work started (User, 2026-11-19).

Design: zendo knows which work runs in the background: the walk and the search of the find panel,
and the answer to each question of each agent session, also to a question in the queue. Each piece
of work has its own identity: a new search or the next question is other work. An action waits for
the work that started while zendo handled its input event. Work that ran before the key does not
hold the action. The time starts when the input thread reads the event, so the wait of the event in
the queue is in the time. A turn of the agent is timed under its own action (`chat.send`,
`read.ask`), so its time stays apart from the times of zendo's own work.

## Where the times are

The time of an action goes into the event of the action in the [telemetry log](telemetry.md), with
its pane, its name and its trigger. The log holds the time of each run; an analytics service makes
the distributions later (User, 2026-11-19). An action whose result has not come at the exit gets
an event with no time (User, 2026-11-19), so that its count stays.

## One mechanism for all actions

The user decided on one standard telemetry for all actions that stays unobtrusive in the code.
"Standard" means one uniform mechanism inside zendo. zendo measures an action at the one place where
actions run, so the code of an action holds no measurement code. A new action gets its measurement
with no extra work.

Design: this is the place where zendo also counts the action (USAGE-1), so one hook gives the event
and the start of the time of each action. The mechanism is in
[ADR 0024](../architecture/decisions/0024-time-actions-where-they-are-counted.md).

## Architecturally significant requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| RESP-1 | Event      | When an action of the keymap runs, the system shall measure the time that the user waits for its result: from the input event to the end of the draw of the screen that shows the result, also when the action starts work in the background, and shall record it in the event of the action (TELEM-1). | User, 2026-11-19 |
| RESP-2 | Ubiquitous | The system shall measure every action through one mechanism at the place where actions run, so that the code of an action holds no measurement code. | User, 2026-11-19 |

### Acceptance tests

**`every_action_gets_a_time`** (integration test with a test clock in `tests/responsiveness.rs`; verifies RESP-1)

- Given a note in read mode with a selection.
- When the user does `add_to_chat` two times and a global action one time, and the clock moves
  10 ms between each key and the draw of the next screen.
- Then the log holds two events of `read.add_to_chat` and one of the global action, each with a
  wait of 10 ms.

**`an_action_waits_for_the_work_that_it_started`** (integration test with a test clock in `tests/responsiveness.rs`; verifies RESP-1)

- Given a note in read mode.
- When an action starts a turn of the agent, the screen shows that the turn runs, and the turn ends
  1.5 s after the key.
- Then the event of the action has a wait of 1.5 s, not the time of the first screen.

**`actions_hold_no_measurement_code`** (integration test in `tests/responsiveness.rs`; verifies RESP-2)

- Given the source code of zendo.
- When the test looks for the calls that measure time.
- Then each such call is in the measurement mechanism, or in a file with a stated reason, and no
  action calls it.

## Detailed requirements

| ID     | Pattern  | Requirement | Source |
| ------ | -------- | ----------- | ------ |
| RESP-9 | Unwanted | If the result of an action has not come when zendo exits, or within 512 seconds, then the system shall record the event of the action with no time. | User, 2026-11-19; Design |

Tests that verify detailed requirements: `work_that_ran_before_the_key_does_not_hold_it`,
`a_queued_question_waits_for_its_own_answer`, `each_input_of_one_frame_waits_for_its_own_work`,
`a_new_search_ends_the_wait_for_the_old_one`, `each_question_without_an_answer_has_its_own_number`,
`each_question_gets_a_new_number` and `the_time_starts_at_the_read_of_the_event` (RESP-1);
`a_result_that_does_not_come_has_no_time` and `an_action_without_its_result_at_the_exit_has_no_time`
(RESP-9).

Retired in revision 5: RESP-3, RESP-4, RESP-5, RESP-6, RESP-7, RESP-8.

The retired requirements were those of the file of the measurements and of its buckets; the
telemetry log takes their place (TELEM-2 to TELEM-12). Retired identifiers are never reused.
