---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "internals.md"
prev: "core-and-runtime.md"
next: "commands.md"
level: "the message as the one input of the core: what a message is, the time in it, and the replay of messages"
note: "Agent draft from the chapter Messages (Msg) of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Messages

Sources: the chapter [Messages (Msg)](../../book/02-messages.md) of the zendo book.

Parent: [Internals of the TUI](internals.md).

## A message is a fact

A message is a value that describes what occurred. It is not a command and it is not a function.
Examples: the user pushed `ctrl-s`; the agent sent a part of the text; the runtime read a file, and
here is its content; a write to the disk failed; 100 ms went by, and here is the time now.

All messages are one enum, `Msg`. To find what can change the state, a person reads this enum: it
is the full list of the events.

## One door into the core

Each event becomes a `Msg` and goes into one function, `update`, from every source: the keyboard,
the mouse, the agent, the disk, a timer. The core has no other input.

**The time comes in a message.** The core does not read a clock ([CORE-1](core-and-runtime.md)).
The runtime brings the time. A test says "the time is 10:00:05" and checks the result, with no wait
and no randomness.

**Messages can be recorded and played again.** The core depends only on its messages, so the same
sequence of messages always gives the same result. The log of the messages of a user with a defect
gives the same defect on another computer.

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| MSG-1 | Ubiquitous | Each event from outside the core shall become a message, a value that describes what occurred: terminal input, the result of a command, an event of a subscription, or a timer. | User, 2026-11-20 |
| MSG-2 | Ubiquitous | The core shall have one input: each message, from every source, shall go into `update`. | User, 2026-11-20 |
| MSG-3 | Ubiquitous | The runtime shall bring the time to the core in messages: the message of a key and the message of a timer shall hold their time. | User, 2026-11-20 |
| MSG-4 | Ubiquitous | The same first model and the same sequence of messages shall always give the same model and the same commands. | User, 2026-11-20 |

### Acceptance tests

**`each_event_becomes_one_message`** (scenario test with fake services, not written yet; verifies MSG-1, MSG-2)

- Given the runtime with fake services and a record of the messages that `update` gets.
- When a key, a paste, a mouse click, a resize, the answer of a command, an event of a subscription
  and a timer occur, one of each.
- Then the record holds one message for each event, and the model changed only in `update`.

**`the_time_comes_in_the_message`** (unit test, not written yet; verifies MSG-3)

- Given a model with a panel that counts a run of `tab`.
- When the test gives two messages of `tab` with times 200 ms apart, and a third one 5 s later.
- Then the first two are one run and the third starts a new run, and the test waited for nothing.

**`a_replayed_log_gives_the_same_state`** (unit test, not written yet; verifies MSG-4)

- Given the recorded messages of a scenario with keys, answers of the agent and timers.
- When the test plays the record two times, each time from the same first model.
- Then the two models are equal, and the two lists of commands are equal.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| MSG-5 | Ubiquitous | The messages shall be one enum, so that the enum lists each event that can change the state. | User, 2026-11-20 |
| MSG-6 | Ubiquitous | A message shall hold data only: no function, no handle and no channel. | User, 2026-11-20 |
| MSG-7 | Unwanted | If an operation of the runtime fails, then the failure shall come to the core as a message, as each other event does. | User, 2026-11-20 |
