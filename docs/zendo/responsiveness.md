---
status: "implemented"
approved: "2026-11-19"
revision: "2"
date: "2026-11-19"
parent: "zendo.md"
level: "what zendo measures about the responsiveness of each action, where it keeps the measurements, and where the measurement lives in the code"
note: "Agent draft of the note of the user of 2026-11-19 (measuring responsiveness of zendo actions), approved by the user on 2026-11-19 with RESP-1 to RESP-7. Revision 2: implemented in zendo 0.121.0; the test names and kinds as in the code; the mechanism in ADR 0024; the file and the setting as the code has them, until the user answers the open points."
---

# Responsiveness of actions

Sources: the note of the user of 2026-11-19, "Measuring responsiveness of Zendo actions".

Parent: [zendo](zendo.md). The actions and their counts are in [Usage counts](usage.md).

## What zendo measures

zendo measures how fast it answers each action of the user. An action is an entry of the keymap,
as in [Usage counts](usage.md): `copy`, `add_to_chat`, the change of a pane, and each other action
that has a name there.

Design: the time of an action starts when zendo reads the key or the mouse event, and ends when the
screen that shows the result of the action is drawn. For an action that starts work in the
background, such as a search or a turn of the agent, the time ends at the first screen that shows
that the work started, not at the end of the work.

## One mechanism for all actions

The user decided on one standard telemetry for all actions that stays unobtrusive in the code.
"Standard" means one uniform mechanism inside zendo, not a standard format with an export. zendo
measures an action at the one place where actions run, so the code of an action holds no
measurement code. A new action gets its measurement with no extra work.

Design: this is the place where zendo also counts the action (USAGE-1), so one hook gives the count
and the time of each action.

## Where the measurements are

Design: like the usage counts, the measurements stay on the local disk of the user, in the data
folder of zendo, in a file of one zendo process. For each keymap pane and action name, zendo keeps
the number of runs and the distribution of the times, not the time of a single run.

Design, until the user answers the open points: the file is
`responsiveness/<date of the start>-<process id>.toml`, apart from the usage file, and
`usage = false` turns the measurements off too.

The mechanism is in [ADR 0024](../architecture/decisions/0024-time-actions-where-they-are-counted.md).

## Architecturally significant requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| RESP-1 | Event      | When an action of the keymap runs, the system shall measure the time from the input event to the end of the draw of the screen that shows the result of the action. | User, 2026-11-19; Design |
| RESP-2 | Ubiquitous | The system shall measure every action through one mechanism at the place where actions run, so that the code of an action holds no measurement code. | User, 2026-11-19 |
| RESP-3 | Ubiquitous | The system shall keep the measurements only in files in the data folder of zendo on the local disk, and shall send them nowhere. | User, 2026-11-19; USAGE-2 |
| RESP-4 | Ubiquitous | The system shall write the measurements off the UI thread, so that keys and the drawing of the screen do not wait for the write. | Design |

### Acceptance tests

**`every_action_gets_a_time`** (integration test with a test clock in `tests/responsiveness.rs`; verifies RESP-1)

- Given a note in read mode with a selection.
- When the user does `add_to_chat` two times and a global action one time, and the clock moves
  10 ms between each key and the draw of the next screen.
- Then the measurements hold two runs of `read.add_to_chat` and one run of the global action, each
  of 10 ms.

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

Tests that verify detailed requirements: `a_run_goes_into_the_bucket_of_its_time` (RESP-1, RESP-5),
`the_file_has_a_head_and_only_buckets` (RESP-6), `a_failed_write_keeps_the_measurements` (RESP-7).

RESP-7 differs from the [failure default](failures.md): the measurements are not user data, and a
message about them would disturb the user more than the loss of the measurements.

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| What does the user do with the measurements: read the files, see them in `zendo doctor`, or get a mark when an action is slow? | RESP-3, RESP-5 | User |
| For an action that starts work in the background, does the time end at the first screen or at the end of the work? | RESP-1 | User |
| Does `usage = false` also turn the measurements off, or do they get a setting of their own? | RESP-3 | User |
| Do the measurements go into the usage file or into a file of their own? | RESP-3, USAGE-9 | User |
