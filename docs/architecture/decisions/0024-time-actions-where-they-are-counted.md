---
status: "approved"
approved: "2026-11-19"
date: "2026-11-19"
note: "Agent draft of the mechanism that the open point of docs/zendo/responsiveness.md asked an ADR for, revised after the code review of 2026-11-19 and for the telemetry log. Approved by the user on 2026-11-19."
sources:
  - "../../zendo/responsiveness.md (RESP-1, RESP-2, RESP-9)"
  - "../../zendo/telemetry.md (TELEM-1, TELEM-5)"
  - "../../zendo/usage.md (USAGE-1): zendo counts an action where it runs, in App::did and App::did_by_mouse"
---

# 0024 Time the actions where zendo counts them

## Context

RESP-1 asks for the time that the user waits for the result of every action of the keymap, from the
read of its input event to the end of the draw of the screen that shows the result, also when the
action starts work in the background (User, 2026-11-19). RESP-2 asks for one mechanism, with no
measurement code in the code of an action. TELEM-5 asks that the write of the log does not delay the
screen. Every action of the keymap already passes one hook: `App::did` and `App::did_by_mouse` count it (USAGE-1).

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Time in the hook of the usage counts | No new place in the code of the actions; the counts and the times cover the same actions | The time knows only the input event and the draw, not the inner steps of an action |
| Spans of the `tracing` crate around each action | Inner steps can get spans later | A new dependency; each action needs a span, which is measurement code in the action |
| A wrapper around each handler of a key | Explicit | Many dispatch paths; each new action needs the wrapper |

## Decision

Time the actions in the hook of the usage counts.

- The input thread puts the time of its read on each input event; the terminal loop gives that
  time to the app, and marks the end of each draw. The wait of an event in the queue is in the
  time.
- `App::did` and `App::did_by_mouse` call one private function that counts the action and gives it
  to the measurements. The measurements keep the action until its result is on the screen, and then
  add the time from its input event to that draw.
- At each input event and at each draw, the app says which work runs in the background: the walk
  and the search of the find panel, and the answer to each question of each thread, in its turn or
  in the queue. Each piece of work has its identity: the number of the query, the number of the
  question. The app reads this from its own state; the code that starts or ends the work holds no
  measurement code.
- An action waits for the work that started while the app handled its input event: from the read
  of that event to the read of the next one or to the draw. It gets its time at the first draw after
  that work ended, or no time when it waited longer than 512 seconds. At the exit, an action that
  still waits gets no time (User, 2026-11-19).
- Each action becomes one event of the telemetry log with its time (TELEM-1): the app gives the
  events of the runs that ended to the log of ADR 0025, which keeps them in memory and writes them
  in a thread of its own.

The hook in `src/app/count.rs` shall be the only code outside `src/responsiveness/` that reports an
action to the measurements.

## Consequences

- A new action of the keymap gets its time when it gets its count.
- An action that zendo does not count is not timed: typed text, keys that go to a session or a
  terminal (USAGE-8).
- The exit waits at most one second for the last write.
- A question that waits in the queue of a busy thread waits for its own answer, and the question
  before it waits only for its answer.
- Work that is not in the list, such as a recap or the scan of the tree, does not hold an action. A
  new kind of work needs one line in the list.
- Each question gets its number when the user sends it, from one counter of the app: a clear of a
  thread does not start the numbers again, and a rename of a thread does not change the work. A
  question that stays in the queue after the agent exited waits until it goes to the agent, or gets
  an event with no time.

## Verification

`actions_hold_no_measurement_code` in `tests/responsiveness.rs` lists the files that use the
measurements or mark the input and the draw, and checks the one call in the hook and the marks in
the loop. `every_action_gets_a_time` checks the times of
real keys through the app; `an_action_waits_for_the_work_that_it_started` checks the wait for a turn
of the agent; `the_time_starts_at_the_read_of_the_event` checks the time in the queue. The unit
tests check the queue of questions, two inputs before one draw, a new search, and the exit.
`a_slow_write_does_not_delay_the_screen` checks the writer thread.

## Decision record

Proposed by the agent on 2026-11-19. Approved by the user on 2026-11-19.
