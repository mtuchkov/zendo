---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "internals.md"
prev: "tickets-and-cancellations.md"
next: "file-writes/file-writes.md"
level: "the two last places where the UI can stop: the write of the frame to the terminal, and a slow update"
note: "Agent draft from the chapter The frame and the time budget of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# The frame and the time budget

Sources: the chapter [The frame and the time budget](../../book/13-frame-and-budget.md) of the
zendo book.

Parent: [Internals of the TUI](internals.md). The time that the user waits for an action is in
[Responsiveness of actions](../responsiveness.md); the log is in
[Telemetry log](../telemetry.md).

## The write to the terminal is IO

The drawing has two steps. The [view](view.md) fills a buffer in memory: this is fast. Then the
frame goes to the terminal through stdout: this is IO. Through a slow SSH connection or with a
busy tmux, stdout can block. Then the loop stops, the keys collect, and the screen freezes.

So a separate writer thread writes the frame. The loop builds the buffer and gives it to the writer
through a channel for one frame. If the writer did not finish the last frame, the new frame
replaces the frame that waits. The frames between are lost: the user needs only the last frame.

## A slow `update`

[Async IO](async-io.md) protects from a wait for IO. It does not protect from slow code in the
core: an O(n²) operation on a large document makes the frame late.

So the core has a budget: 8 ms for `update` and `view` together. This is half of a frame at 60 Hz,
with a margin. The runtime measures each message in one place, from the arrival of the message to
the end of the frame. A message over the budget gets a record in the telemetry with its type, so a
person sees "`Msg::Editor(Paste)` is slow on a 2 MB file", not only "it is slow".

The budget and the telemetry find a slow `update`; they do not prevent it. This is the only place
where the architecture uses measurement, not construction.

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| FRAME-1 | Ubiquitous | A writer thread of its own shall write each frame to the terminal, and the UI loop shall not wait for the terminal. | User, 2026-11-20 |
| FRAME-2 | Unwanted | If the writer has not taken the last frame when a new frame is ready, then the new frame shall replace the frame that waits. | User, 2026-11-20 |
| BUDGET-1 | Ubiquitous | The core shall handle a message within 8 ms, for `update` and `view` together. | User, 2026-11-20 |
| BUDGET-2 | Ubiquitous | The runtime shall measure each message in one place, from the arrival of the message to the end of the frame, and the code of the components shall hold no measurement. | User, 2026-11-20 |
| BUDGET-3 | Unwanted | If a message goes over the budget, then the runtime shall write a record with the type of the message in the telemetry. | User, 2026-11-20 |

### Acceptance tests

**`a_blocked_terminal_does_not_stop_the_loop`** (scenario test with fake services, not written yet; verifies FRAME-1)

- Given the runtime with a fake terminal that blocks its write.
- When the user pushes three keys.
- Then `update` gets the three keys, and the model has their changes, while the write still blocks.

**`the_writer_writes_only_the_newest_frame`** (scenario test with fake services, not written yet; verifies FRAME-2)

- Given a fake terminal that blocks during the write of frame 1.
- When the loop builds the frames 2, 3 and 4, and then the test lets the terminal go.
- Then the next frame on the terminal is frame 4, and the terminal never gets the frames 2 and 3.

**`keys_change_the_screen_while_services_are_slow`** (scenario test with fake services, not written yet; verifies BUDGET-1)

- Given the runtime with fake services that answer each request after 5 seconds.
- When the user pushes keys that open a file, start a search and move the cursor.
- Then the frame after each key is built within the budget.

**`components_hold_no_measurement_code`** (unit test, not written yet; verifies BUDGET-2)

- Given the source code of zendo.
- When the test looks for the calls that measure time.
- Then each call is in the runtime, and no file of the core has one.

**`a_slow_message_gets_a_record`** (scenario test with fake services, not written yet; verifies BUDGET-3)

- Given a test clock that moves 20 ms during the update of a paste, and 1 ms during a cursor key.
- When both messages are handled.
- Then the telemetry log holds one record, with the type of the paste message.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| FRAME-3 | Ubiquitous | The UI loop shall build each frame in a buffer in memory and give it to the writer through a channel that holds one frame. | User, 2026-11-20 |
| FRAME-4 | Ubiquitous | The writer shall keep the last frame that it wrote and shall send only the difference to the terminal. | User, 2026-11-20 |
| BUDGET-4 | Ubiquitous | The record of a slow message shall hold nothing that identifies the user (TELEM-7). | TELEM-7 |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| Is "the end of the frame" the end of the build of the buffer, or the end of the write by the writer thread? A slow terminal goes into the time only in the second case. | BUDGET-2, BUDGET-3 | User |
| A batch of messages gives one frame (ASYNC-2). How does the runtime divide the time of the frame between the messages of the batch? | BUDGET-2 | User |
| RESP-1 ends the time of an action at "the end of the draw of the screen that shows the result". With a writer thread that drops frames, which frame is that? | RESP-1, FRAME-2 | User |
