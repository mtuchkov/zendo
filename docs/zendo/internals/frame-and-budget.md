---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "internals.md"
prev: "tickets-and-cancellations.md"
next: "frame-and-budget-measurements.md"
level: "the two last places where the UI can stop: the write of the frame to the terminal, and a slow update"
note: "Draft from the chapter The frame and the time budget of the zendo book, the target architecture (User, 2026-11-20), and from the decisions of the user of 2026-11-21 on the end of the frame, the cycle and the measurement of the write. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# The frame and the time budget

Sources: the chapter [The frame and the time budget](../../book/13-frame-and-budget.md) of the
zendo book; the decisions of the user on 2026-11-21.

Parent: [Internals of the TUI](internals.md). The records of the measurements are in
[Measurements](frame-and-budget-measurements.md). The time that the user waits for an action is in
[Responsiveness of actions](../responsiveness.md).

## The write to the terminal is IO

The drawing has two steps. The [view](view.md) fills a buffer in memory: this is fast. Then the
frame goes to the terminal through stdout: this is IO. A slow SSH connection or a busy tmux can
block stdout, and the screen freezes.

So a separate writer thread writes the frame. The loop gives the buffer to the writer through a
channel for one frame. If the writer did not finish the last frame, the new frame replaces the
frame that waits: the user needs only the last frame.

Each frame has a number, and the writer tells the number of the last frame that it wrote. With
it, the runtime knows when the screen shows a result (RESP-1).

## A slow `update`

[Async IO](async-io.md) does not protect from slow code in the core.

So the core has a budget: 8 ms for a frame. A cycle of the UI loop gives one frame, so the budget
is for all `update` calls of the cycle and the `view` together: half of a frame at 60 Hz. The frame ends when the buffer is filled. The write to the terminal is a
measurement of its own, so a slow terminal does not look like a slow core.

The runtime measures in one place: each `update` call, and each cycle. A cycle over the budget
gets a record in the telemetry, and a slow `update` gets one too: see
[Measurements](frame-and-budget-measurements.md).

The budget finds a slow `update`; it does not prevent it.

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| FRAME-1 | Ubiquitous | A writer thread of its own shall write each frame to the terminal, and the UI loop shall not wait for the terminal. | User, 2026-11-20 |
| FRAME-2 | Unwanted | If the writer has not taken the last frame when a new frame is ready, then the new frame shall replace the frame that waits. | User, 2026-11-20 |
| BUDGET-1 | Ubiquitous | The core shall handle the messages of one cycle of the UI loop within 8 ms, for the `update` calls and the `view` together. | User, 2026-11-20; User, 2026-11-21 |
| BUDGET-2 | Ubiquitous | The runtime shall measure, in one place, each `update` call and each cycle, from the first message of the cycle to the filled buffer of the frame; the code of the components shall hold no measurement. | User, 2026-11-20; User, 2026-11-21 |
| BUDGET-3 | Unwanted | If a cycle goes over the budget, then the runtime shall write one record of the cycle in the telemetry, with the types of its messages. | User, 2026-11-20; User, 2026-11-21 |

### Acceptance tests

**`a_blocked_terminal_does_not_stop_the_loop`** (scenario test with fake services, not written yet; verifies FRAME-1)

- Given the runtime with a fake terminal that blocks its write.
- When the user pushes three keys.
- Then `update` gets the three keys, and the model has their changes, while the write still blocks.

**`the_writer_writes_only_the_newest_frame`** (scenario test with fake services, not written yet; verifies FRAME-2)

- Given a fake terminal that blocks during the write of frame 1.
- When the loop builds the frames 2, 3 and 4, and then the test lets the terminal go.
- Then the terminal gets frame 4 next, and never the frames 2 and 3.

**`keys_change_the_screen_while_services_are_slow`** (scenario test with fake services, not written yet; verifies BUDGET-1)

- Given the runtime with fake services that answer each request after 5 seconds.
- When the user pushes keys that open a file, start a search and move the cursor.
- Then the frame after each key is built within the budget.

**`a_slow_terminal_is_not_in_the_budget`** (scenario test with fake services, not written yet; verifies BUDGET-2)

- Given a test clock, and a fake terminal that takes 50 ms for each write.
- When a cycle with one cursor key takes 1 ms to the filled buffer.
- Then the time of the cycle is 1 ms, and the 50 ms of the write are in their own measurement.

**`a_slow_cycle_gets_a_record`** (scenario test with fake services, not written yet; verifies BUDGET-3)

- Given a cycle with a paste of 20 ms and two cursor keys of 1 ms each, and a second cycle of 2 ms.
- When both cycles end.
- Then the telemetry log holds one record, of the first cycle: the paste type with a count of 1
  and 20 ms, and the cursor type with a count of 2.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| FRAME-3 | Ubiquitous | The UI loop shall build each frame in a buffer in memory and give it to the writer through a channel that holds one frame. | User, 2026-11-20 |
| FRAME-4 | Ubiquitous | The writer shall keep the last frame that it wrote and shall send only the difference to the terminal. | User, 2026-11-20 |
| FRAME-6 | Ubiquitous | Each frame shall have a number, and the writer shall tell the number of the last frame that it wrote. | User, 2026-11-21 |
| FRAME-7 | Ubiquitous | For RESP-1, the draw of the screen that shows a result shall end when the writer wrote the first frame that is not older than the frame with the result. | User, 2026-11-21 |
