---
status: "approved"
approved: "2026-11-19"
date: "2026-11-19"
note: "Draft of the mechanism of the telemetry log that the user decided on 2026-11-19. Approved by the user on 2026-11-19; implemented in zendo 0.124.0."
sources:
  - "The user, 2026-11-19: telemetry accumulates as files with events, with a rolling rule of 50 files of 1 MB each, in a location that git ignores; later zendo sends it to an analytics service; the events stay in memory and a separate thread writes them."
  - "../../zendo/telemetry.md (TELEM-1 to TELEM-12)"
  - "ADR 0024: zendo times the actions where it counts them"
---

# 0025 A telemetry log of events in rolling files

## Context

zendo kept the usage counts and the times of the actions as summaries: one TOML file of each
process for each, replaced whole at each write. The user decided on 2026-11-19 that the telemetry is
a log of events instead, which a later step sends to an analytics service. The log must not grow
without end: at most 50 files of 1 MB (TELEM-3). Several zendo processes run at the same time on one
machine, and each writes events.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| One file for all processes, appended under a lock | One current file | A lock between processes; the rotation of a file that another process appends to |
| Files of each process, one limit for all files | No lock: two processes never write one file | A process that rolls also deletes the old files of other processes |
| A limit for each process | Simple | The log grows with the number of processes, not at most 50 MB |

## Decision

Files of each process, one limit for all files.

- Each process appends to its file `telemetry/<UTC time of the start of the file>-<process id>.jsonl`.
  The name sorts by age.
- One event is one line of JSON. The writer thread appends only whole lines, and starts a new file
  before a line takes the file over 1 MB.
- When a process starts a new file, it lists the folder and deletes the oldest files by name until
  at most 50 stay. A file that another process deleted first is not an error. The current file of
  another process can go too; that process continues in a new file at its next write.
- The events of the last 15 seconds stay in memory. The writer thread of `src/stats.rs` writes them;
  the loop only gives it the lines. A write that fails keeps the lines for the next write, at most
  1 MB of them; older lines go first.
- One event comes for each action of the keymap, from the hook of ADR 0024: when its result is on
  the screen, or with no time when its result has not come at the exit or within 512 seconds.

## Consequences

- The summaries `usage/` and `responsiveness/` go: zendo deletes their folders at the start
  (TELEM-14), and the counts and the distributions come from the events.
- At most about 50 MB of the disk, also with many processes.
- A process can lose its current file to another process that rolls: at most the events of that one
  file, the oldest of the log.
- An analytics service later reads whole lines; a crash in a write can leave one incomplete last
  line, which a reader skips.

## Verification

`the_log_keeps_at_most_50_files_of_1_mb` and `two_processes_write_two_files` check the rolling and
the files of each process; `a_slow_write_does_not_delay_the_screen` checks the writer thread.
