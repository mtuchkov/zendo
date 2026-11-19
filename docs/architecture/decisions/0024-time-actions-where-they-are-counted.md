---
status: "draft"
date: "2026-11-19"
note: "Agent draft of the mechanism that the open point of docs/zendo/responsiveness.md asked an ADR for. The user approved the specification on 2026-11-19 and asked to implement it; this record is not reviewed by the user."
sources:
  - "../../zendo/responsiveness.md (RESP-1, RESP-2, RESP-4)"
  - "../../zendo/usage.md (USAGE-1): zendo counts an action where it runs, in App::did and App::did_by_mouse"
---

# 0024 Time the actions where zendo counts them

## Context

RESP-1 asks for the time that the user waits for the result of every action of the keymap, from the
read of its input event to the end of the draw of the screen that shows the result, also when the
action starts work in the background (User, 2026-11-19). RESP-2 asks for one mechanism, with no
measurement code in the code of an action. RESP-4 asks that the write of the measurements does not
delay the screen. Every action of the keymap already passes one hook: `App::did` and `App::did_by_mouse` count it (USAGE-1).

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Time in the hook of the usage counts | No new place in the code of the actions; the counts and the times cover the same actions | The time knows only the input event and the draw, not the inner steps of an action |
| Spans of the `tracing` crate around each action | Inner steps can get spans later | A new dependency; each action needs a span, which is measurement code in the action |
| A wrapper around each handler of a key | Explicit | Many dispatch paths; each new action needs the wrapper |

## Decision

Proposal of the agent: time the actions in the hook of the usage counts.

- The terminal loop marks the read of each input event and the end of each draw.
- `App::did` and `App::did_by_mouse` call one private function that counts the action and gives it
  to the measurements. The measurements keep the action until its result is on the screen, and then
  add the time from its input event to that draw.
- At each draw, the app says which work runs in the background: the search of the find panel, and
  the turn of the agent in each thread. The app reads this from its own state; the code that starts
  or ends the work holds no measurement code.
- An action waits for the work that started between the draw before its input and the first draw
  after it. It gets its time at the first draw after that work ended, or when it waited longer than
  the last bucket.
- A thread of the measurements writes the file. The loop only sends the text to it.

The hook in `src/app/count.rs` shall be the only code outside `src/responsiveness/` that reports an
action to the measurements.

## Consequences

- A new action of the keymap gets its time when it gets its count.
- An action that zendo does not count is not timed: typed text, keys that go to a session or a
  terminal (USAGE-8).
- The exit waits at most one second for the last write.
- Work that a key starts while work of the same kind runs, such as a question that waits in the
  queue of a busy thread, does not hold the action: its kind ran before the key.
- Work that is not in the list, such as a recap or the scan of the tree, does not hold an action. A
  new kind of work needs one line in the list.

## Verification

`actions_hold_no_measurement_code` in `tests/responsiveness.rs` lists the files that use the
measurements and checks the one call in the hook. `every_action_gets_a_time` checks the times of
real keys through the app; `an_action_waits_for_the_work_that_it_started` checks the wait for a turn
of the agent. `a_slow_write_does_not_delay_the_screen` checks the writer thread.

## Decision record

Proposed by the agent on 2026-11-19. Waits for the review of the user.
