---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "internals.md"
prev: "subscriptions.md"
next: "tickets-and-cancellations.md"
level: "the UI loop that never waits, the executor of commands, blocking calls, and heavy calculations as commands"
note: "Agent draft from the chapter Async IO of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Async IO

Sources: the chapter [Async IO: the UI loop never waits](../../book/11-async-io.md) of the zendo
book.

Parent: [Internals of the TUI](internals.md). What the user gets from this is in
[Responsiveness of actions](../responsiveness.md) and in
[Find: search in the background](../find-background.md) (FIND-36). Answers that come late are in
[Tickets and cancellations](tickets-and-cancellations.md).

## The UI loop

The loop of the runtime does the same steps again and again:

1. It waits for a message or for the frame timer. A message is a key, an answer of a command, or an
   event of a subscription.
2. It takes all the messages that are ready, and gives each one to `update`.
3. It gives the commands from `update` to the executor.
4. It draws the frame one time.

The loop never waits for a command to finish, so a slow disk, git or the agent cannot stop the
screen. Step 2 keeps the number of frames low: the agent streams text word by word, and a trackpad
sends tens of scroll events. The runtime applies all the events that wait, and then draws one time.

## The executor

The executor starts each command as a tokio task. When the task is complete, the result comes into
the loop as a message.

In Rust, "async" does not mean "does not block". These APIs are synchronous: `std::fs`, the walk
of files with `ignore`, the highlight with `syntect`, the diff with `similar`, the PTY read, the
keyboard read. A task that calls one of them directly holds a tokio thread and makes other tasks
slow.

## Heavy calculations are also IO

For the UI, a wait for the disk and 200 ms of markdown for a large file are the same: the frame is
late. So a heavy calculation is a command: the render of markdown, a diff, the highlight. Until
the result comes, the view shows the old result from the cache or a placeholder
([VIEW-4](view.md)).

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| ASYNC-1 | Ubiquitous | The UI loop shall wait only for a message or for the frame timer, and shall never wait for a command to end. | User, 2026-11-20 |
| ASYNC-2 | Event | When messages are ready, the UI loop shall give all of them to `update`, give the commands to the executor, and then draw the frame one time. | User, 2026-11-20 |
| ASYNC-3 | Ubiquitous | A blocking call shall run in `spawn_blocking` or in a thread of its own, and shall not run directly in a task of the asynchronous runtime. | User, 2026-11-20 |
| ASYNC-4 | Ubiquitous | A heavy calculation (the render of markdown, a diff, the highlight of code) shall be a command, and shall not run in `update` or in `view`. | User, 2026-11-20 |

### Acceptance tests

**`a_key_is_handled_while_a_command_runs`** (scenario test with fake services, not written yet; verifies ASYNC-1)

- Given the runtime with a fake disk that holds the answer of a delete.
- When the user confirms the delete of a file, and then moves the cursor of the tree two times.
- Then the screen shows the cursor on its new row while the delete still runs.

**`a_burst_of_messages_gives_one_frame`** (scenario test with fake services, not written yet; verifies ASYNC-2)

- Given the runtime with a fake agent and a fake terminal that counts the frames.
- When the agent sends 50 parts of text before the loop takes the next message.
- Then `update` gets 50 messages, and the loop draws one frame with all the text.

**`a_blocking_call_does_not_hold_the_executor`** (scenario test with fake services, not written yet; verifies ASYNC-3)

- Given the executor with one thread of the asynchronous runtime, and a fake disk whose read blocks
  its thread until the test lets it go.
- When a command of that read runs, and then a command to the fake agent.
- Then the answer of the agent comes to the core while the read still blocks.

**`a_heavy_calculation_is_a_command`** (unit test, not written yet; verifies ASYNC-4)

- Given a model that opens a large file whose highlight is not in the cache.
- When the message with the text of the file comes.
- Then `update` returns the command of the highlight, the view shows the plain text, and the view
  shows the colors after the message with the result.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| ASYNC-5 | Event | When the executor gets a command, the executor shall start it as a task of its own, and the result of the task shall come into the UI loop as a message. | User, 2026-11-20 |
| ASYNC-6 | Ubiquitous | The executor shall handle these calls as blocking calls: the file system, the walk of files, the highlight, the diff, the PTY read and the keyboard read. | User, 2026-11-20 |
| ASYNC-7 | Ubiquitous | Each PTY and the keyboard shall read in a permanent thread of their own that only wakes the UI loop. | User, 2026-11-20 |
| ASYNC-8 | Ubiquitous | The UI loop shall not draw a frame for each message of a batch. | User, 2026-11-20 |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| ADR 0009 (git on a thread, with a poll in the tick) and ADR 0017 (one worker thread of find) give mechanisms of the code before this architecture. Do commands and subscriptions take their place, with the same behavior for the user? | ASYNC-3, ASYNC-5, FIND-36 | User |
