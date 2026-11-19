---
status: "draft"
revision: "1"
date: "2026-11-19"
parent: "zendo.md"
level: "what the telemetry log of zendo holds, where it is, how large it grows, and when zendo writes it"
note: "Agent draft of the decisions of the user of 2026-11-19: the telemetry is a log of events, in files with a rolling rule of 50 files of 1 MB, in the data folder; a later step sends it to an analytics service. The log takes the place of the files of the usage counts and of the times of the actions; their requirements of the files move here. Waits for the review of the user."
---

# Telemetry log

Sources: the decisions of the user in chat on 2026-11-19.

Parent: [zendo](zendo.md). What goes into an event: [Usage counts](usage.md) and
[Responsiveness of actions](responsiveness.md). The mechanism of the files:
[ADR 0025](../architecture/decisions/0025-telemetry-log-with-rolling-files.md).

## What the log holds

zendo keeps its telemetry as a log of events. Each action of the keymap becomes one event: its
keymap pane, its action name and its trigger ([Usage counts](usage.md)), and the time that the user
waited for its result ([Responsiveness of actions](responsiveness.md)). The log holds the events,
not counts or distributions: a later step sends the log to an analytics service, which makes the
counts and the distributions (User, 2026-11-19).

Design: each event is one line of JSON, so that a tool can read the log line by line:

```json
{"at":"2026-11-19T13:04:05.120Z","zendo":"0.124.0","os":"linux","pid":4711,"pane":"read","action":"add_to_chat","trigger":"key","wait_ms":12}
```

An event holds no text of the user, no file name and no path.

## Where the log is

The log is in the folder `telemetry` of the data folder of zendo (`~/.local/share/zendo`, or
`$XDG_DATA_HOME/zendo`). This folder is outside every repository, so git never sees the log
(User, 2026-11-19).

The log rolls (User, 2026-11-19):

- A file of the log grows to at most 1 MB. Then zendo continues in a new file.
- At most 50 files stay. When a new file comes and there are more, zendo deletes the oldest.
- The limit is for the files of all zendo processes together: the log takes at most about 50 MB.

Design: each zendo process writes only files of its own, so that two processes never write the
same file.

## When zendo writes the log

The new events stay in memory. A thread of their own appends them to the file, so that keys and
the draw of the screen never wait for the disk (User, 2026-11-19). zendo writes the waiting events
every 15 seconds and at the exit; the exit waits at most one second for the write. A crash loses
at most the last 15 seconds.

`stats = false` in `~/.zendo/config.toml` turns the log off. The old setting `usage = false` works
the same.

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| TELEM-1 | Ubiquitous | The system shall keep its telemetry as a log of events, with one event for each action of the keymap. | User, 2026-11-19 |
| TELEM-2 | Ubiquitous | The system shall keep the telemetry log only in the folder `telemetry` of the data folder of zendo on the local disk. | User, 2026-11-19 |
| TELEM-3 | Ubiquitous | The system shall keep at most 50 files of the telemetry log, each of at most 1 MB, for all zendo processes together. | User, 2026-11-19 |
| TELEM-4 | Event      | When the file of the log that the system writes reaches 1 MB, the system shall continue in a new file and delete the oldest files above 50. | User, 2026-11-19 |
| TELEM-5 | Ubiquitous | The system shall keep the new events in memory and write them in a thread of their own, so that keys and the drawing of the screen do not wait for the write. | User, 2026-11-19 |

### Acceptance tests

**`an_action_becomes_one_event`** (integration test with a test clock in `tests/telemetry.rs`; verifies TELEM-1)

- Given a note in read mode with a selection, and an empty data folder.
- When the user does `add_to_chat` with its key, the clock moves 10 ms before the draw, and zendo
  exits.
- Then the log holds one line: the event of `read.add_to_chat` with the trigger `key` and a wait of
  10 ms.

**`the_log_stays_in_the_data_folder`** (unit test; verifies TELEM-2)

- Given an empty data folder and the events of two actions.
- When zendo writes them.
- Then only files in `telemetry/` of the data folder change.

**`the_log_keeps_at_most_50_files_of_1_mb`** (unit test; verifies TELEM-3, TELEM-4)

- Given a log folder with 50 full files of other processes.
- When zendo writes more than 1 MB of events.
- Then no file is larger than 1 MB, 50 files stay, and the oldest files are gone.

**`a_slow_write_does_not_delay_the_screen`** (unit test; verifies TELEM-5)

- Given a write of the log that does not return.
- When the user does two actions 15 seconds apart, and zendo exits.
- Then both actions have their events in memory, and the exit waits at most one second.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| TELEM-6  | Ubiquitous | The system shall write each event as one line of JSON with the time of its input event, the version of zendo, the system and the process id. | Design |
| TELEM-7  | Ubiquitous | The system shall write to the log no text of the user, no file name and no path. | Design; USAGE-9, RESP-6 |
| TELEM-8  | State      | While new events wait in memory, the system shall write them every 15 seconds. | Design; USAGE-3 |
| TELEM-9  | Event      | When zendo exits, the system shall write the waiting events, and wait at most one second for the write. | Design; USAGE-12 |
| TELEM-10 | Ubiquitous | The system shall write in each zendo process only files of that process. | Design; USAGE-4 |
| TELEM-11 | Unwanted   | If a write of the log fails, then the system shall continue with no message and no delay of the screen, and shall keep the events for the next write, at most 1 MB of them. | Design; USAGE-5, RESP-7 |
| TELEM-12 | Optional   | Where `stats = false` is in the config, the system shall record no event and write no file of the log. | User, 2026-11-19; USAGE-14, RESP-8 |

Tests that verify detailed requirements: `an_event_is_one_line_of_json` (TELEM-6, TELEM-7),
`the_events_reach_the_disk_within_15_seconds` (TELEM-8), `the_exit_writes_the_events` (TELEM-9),
`two_processes_write_two_files` (TELEM-10), `a_failed_write_keeps_the_events` (TELEM-11),
`off_records_nothing_and_writes_nothing` and `stats_false_turns_the_stats_off` (TELEM-12).

TELEM-11 differs from the [failure default](failures.md): the log is not user data, and a message
about it would disturb the user more than the loss of some events.

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| When and how does zendo send the log to an analytics service, and what does the user agree to before the first send? | TELEM-2 | User |
| Does zendo delete the old folders `usage/` and `responsiveness/` of the data folder? | TELEM-2 | User |
