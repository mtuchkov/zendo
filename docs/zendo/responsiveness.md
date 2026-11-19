---
status: "implemented"
approved: "2026-11-19"
revision: "4"
date: "2026-11-19"
parent: "zendo.md"
level: "what zendo measures about the responsiveness of each action, where it keeps the measurements, and where the measurement lives in the code"
note: "Agent draft of the note of the user of 2026-11-19 (measuring responsiveness of zendo actions), approved by the user on 2026-11-19 with RESP-1 to RESP-7. Revision 2: implemented in zendo 0.121.0; the test names and kinds as in the code; the mechanism in ADR 0024. Revision 3, from the decisions of the user of 2026-11-19: the time spans the wait for the result, also for work in the background (RESP-1); `stats = false` turns the measurements off (RESP-8); approved by the user on 2026-11-19. Revision 4, after the code review of 2026-11-19: each piece of work in the background has its own identity, each input waits only for its own work, the time starts at the read of the event, and an action whose result has not come at the exit gets no time (User, 2026-11-19). The design text and the test names of revision 4 are an agent draft that waits for the review of the user."
---

# Responsiveness of actions

Sources: the note of the user of 2026-11-19, "Measuring responsiveness of Zendo actions", and the
decisions of the user in chat on 2026-11-19.

Parent: [zendo](zendo.md). The actions and their counts are in [Usage counts](usage.md).

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
the queue is in the time. A result that does not come goes into the last bucket of the distribution.
An action whose result has not come at the exit gets no time (User, 2026-11-19). A turn of the
agent is timed under its own action (`chat.send`, `read.ask`), so its time stays apart from the
times of zendo's own work.

## One mechanism for all actions

The user decided on one standard telemetry for all actions that stays unobtrusive in the code.
"Standard" means one uniform mechanism inside zendo, not a standard format with an export. zendo
measures an action at the one place where actions run, so the code of an action holds no
measurement code. A new action gets its measurement with no extra work.

Design: this is the place where zendo also counts the action (USAGE-1), so one hook gives the count
and the start of the time of each action. The mechanism is in
[ADR 0024](../architecture/decisions/0024-time-actions-where-they-are-counted.md).

## Where the measurements are

Like the usage counts, the measurements stay on the local disk of the user, in the data folder of
zendo, in a file of one zendo process. For each keymap pane and action name, zendo keeps the number
of runs and the distribution of the times, not the time of a single run.

`stats = false` in `~/.zendo/config.toml` turns off the usage counts and the measurements
(User, 2026-11-19). Design: the old setting `usage = false` works as `stats = false`.

Design, until the user answers the open point: the file is
`responsiveness/<date of the start>-<process id>.toml`, apart from the usage file.

## Architecturally significant requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| RESP-1 | Event      | When an action of the keymap runs, the system shall measure the time that the user waits for its result: from the input event to the end of the draw of the screen that shows the result, also when the action starts work in the background. | User, 2026-11-19 |
| RESP-2 | Ubiquitous | The system shall measure every action through one mechanism at the place where actions run, so that the code of an action holds no measurement code. | User, 2026-11-19 |
| RESP-3 | Ubiquitous | The system shall keep the measurements only in files in the data folder of zendo on the local disk, and shall send them nowhere. | User, 2026-11-19; USAGE-2 |
| RESP-4 | Ubiquitous | The system shall write the measurements off the UI thread, so that keys and the drawing of the screen do not wait for the write. | Design; User, 2026-11-19 (USAGE-15) |

### Acceptance tests

**`every_action_gets_a_time`** (integration test with a test clock in `tests/responsiveness.rs`; verifies RESP-1)

- Given a note in read mode with a selection.
- When the user does `add_to_chat` two times and a global action one time, and the clock moves
  10 ms between each key and the draw of the next screen.
- Then the measurements hold two runs of `read.add_to_chat` and one run of the global action, each
  of 10 ms.

**`an_action_waits_for_the_work_that_it_started`** (integration test with a test clock in `tests/responsiveness.rs`; verifies RESP-1)

- Given a note in read mode.
- When an action starts a turn of the agent, the screen shows that the turn runs, and the turn ends
  1.5 s after the key.
- Then the time of the action is in the bucket up to 2 s, not in the bucket of the first screen.

**`actions_hold_no_measurement_code`** (integration test in `tests/responsiveness.rs`; verifies RESP-2)

- Given the source code of zendo.
- When the test looks for the calls that measure time.
- Then each such call is in the measurement mechanism, or in a file with a stated reason, and no
  action calls it.

**`the_measurements_stay_in_the_data_folder`** (unit test; verifies RESP-3)

- Given an empty data folder and the measurements of one run.
- When zendo writes them.
- Then only files in the data folder of zendo change.

**`a_slow_write_does_not_delay_the_screen`** (unit test; verifies RESP-4)

- Given a write of the measurements that does not return.
- When the user does an action.
- Then the screen shows the result of the action, and the next action works.

## Detailed requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| RESP-5 | Ubiquitous | The system shall keep, for each keymap pane and action name, the number of runs and the distribution of their times. | Design |
| RESP-6 | Ubiquitous | The system shall write with the measurements no text of the user, no file name, no path and no time of a single run. | Design; USAGE-9 |
| RESP-7 | Unwanted   | If the write of the measurements fails, then the system shall continue with no message and no delay of the screen. | Design; USAGE-5 |
| RESP-8 | Optional   | Where `stats = false` is in the config, the system shall measure nothing and write no file of measurements. | User, 2026-11-19 |

Tests that verify detailed requirements: `a_run_goes_into_the_bucket_of_its_time` (RESP-1, RESP-5);
`work_that_ran_before_the_key_does_not_hold_it`, `a_result_that_does_not_come_goes_into_the_last_bucket`,
`a_queued_question_waits_for_its_own_answer`, `each_input_of_one_frame_waits_for_its_own_work`,
`a_new_search_ends_the_wait_for_the_old_one`, `an_action_without_its_result_at_the_exit_gets_no_time`,
`each_question_without_an_answer_has_its_own_number`, `each_question_gets_a_new_number` and
`the_time_starts_at_the_read_of_the_event`
(RESP-1); `the_file_has_a_head_and_only_buckets` (RESP-6); `a_failed_write_keeps_the_measurements`
and `a_failed_write_is_written_again_at_the_exit` (RESP-7); `off_measures_nothing_and_writes_nothing` and `stats_false_turns_the_stats_off` (RESP-8).

RESP-7 differs from the [failure default](failures.md): the measurements are not user data, and a
message about them would disturb the user more than the loss of the measurements.

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| What does the user do with the measurements: read the files, see them in `zendo doctor`, or get a mark when an action is slow? | RESP-3, RESP-5 | User |
| Do the measurements go into the usage file or into a file of their own? | RESP-3, USAGE-9 | User |
