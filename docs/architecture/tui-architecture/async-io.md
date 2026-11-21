---
status: "draft"
date: "2026-11-20"
sources:
  - "The user, 2026-11-20: all IO is asynchronous and never blocks the UI."
  - "../../zendo/failures.md (FAIL-1, FAIL-2), ../../zendo/responsiveness.md (RESP-1, RESP-2)"
---

# Asynchronous IO

Parent: [The architecture of the zendo TUI](tui-architecture.md).

This document describes how the runtime does IO so that the UI loop never waits for it.

## Subscriptions

- `subscriptions(&Model)` shall return the sources that the model needs now: agent events, PTY
  output, the file watcher, find results, timers.
- After each update, the runtime shall compare the subscriptions with the last ones and start or
  stop sources to match.
- While no element animates, no animation timer shall run.
- The core shall not poll for news.

## The executor and the UI loop

- The UI loop shall wait only for messages and for the frame timer. It shall not wait for a
  command.
- When `update` returns commands, the runtime shall give them to the executor and return to the
  loop at once.
- The executor shall run each command as a tokio task.
- If an API blocks (the file system, `ignore`, `syntect`, `similar`, a PTY read, terminal input),
  then the executor shall run it in `spawn_blocking` or on a thread of its own.
- CPU-heavy work shall be a command. Until its result comes, the view shall show the last cached
  result or a placeholder.
- Each result shall carry the version of its input. If the version is old, then `update` shall
  drop the result.
- When the model no longer needs a result, the runtime shall cancel its task.
- The runtime shall update with all messages that are ready, and then draw once.
- A thread of its own shall write frames to the terminal. A new frame shall replace a frame that
  is not written yet.
- Each PTY shall read on a thread of its own and shall only wake the loop.

## Responsiveness

- The runtime shall measure the time of each `Msg` at one place (RESP-1, RESP-2).
- If an update and its view take more than 8 ms, then the runtime shall record the type of the
  `Msg` in the telemetry.

## Writes of user data

- Writes to one path shall go through one queue, and the last write shall win.
- While a write runs, the status bar shall show it.
- If a write fails, then the runtime shall send `Msg::WriteFailed`, and the model shall keep the
  data as not saved and show the failure (FAIL-1, FAIL-2).
- The saved state shall be a projection of the model. When its version changes, the runtime shall
  write it after a debounce.
- When the user quits, the runtime shall wait for the queued writes with a time limit, and shall
  keep drawing.
