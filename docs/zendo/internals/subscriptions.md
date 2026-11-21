---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "internals.md"
prev: "keymap.md"
next: "async-io.md"
level: "the streams of events: the model tells which sources run, and the runtime starts and stops them"
note: "Agent draft from the chapter Subscriptions of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Subscriptions

Sources: the chapter [Subscriptions](../../book/10-subscriptions.md) of the zendo book.

Parent: [Internals of the TUI](internals.md).

## The model tells what to listen to

A [command](commands.md) is a single action. Many events come as a stream: the agent sends text in
parts, a session prints, files on the disk change. The core does not poll for them. It tells which
sources it must listen to now:

```rust
fn subscriptions(model: &Model) -> Vec<Sub>
```

This is a pure function of the model. The agent is connected → `Sub::Agent`. Two sessions are open
→ `Sub::Pty(1)`, `Sub::Pty(2)`. Search number 17 runs → `Sub::Find(17)`.

Each [component](components.md) declares its own subscriptions. The parent collects them and wraps
their address, as it does for commands. The runtime starts and stops the sources and keeps their
channels; the core has no channels ([MODEL-2](model.md)).

## The runtime compares the lists

After each `update`, the runtime calls `subscriptions` and compares the new list with the old one:
a new subscription → start the source; a subscription is gone → stop the source; a subscription
stays → do not touch the source. For the comparison, each subscription has an identity: `Pty(2)` is
always session 2.

The sources push their events. When a session prints, a message comes. Nothing asks "is there
news?".

The model alone decides which sources run. No code must remember to start a source in one place
and to stop it in a different place: the model says "the session is open", and the source exists.

An animation is not a subscription, because it does not change the model: the view asks for its
frames ([VIEW-9](view.md)).

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| SUB-1 | Ubiquitous | `subscriptions` shall be a pure function of the model that gives the sources of events that the core listens to now. | User, 2026-11-20 |
| SUB-2 | Event | When `update` ends, the runtime shall compare the new subscriptions with the old ones: it shall start the source of each new subscription, stop the source of each subscription that is gone, and not touch the source of a subscription that stays. | User, 2026-11-20 |
| SUB-3 | Ubiquitous | The core shall not poll: each source shall push its events as messages, and only when an event exists. | User, 2026-11-20 |

### Acceptance tests

**`the_model_decides_which_sources_run`** (scenario test with fake services, not written yet; verifies SUB-1, SUB-2)

- Given the runtime with a fake PTY service that records each start and each stop of a source.
- When the user opens session 1 and session 2, closes session 1, and then closes session 2.
- Then the record is: start 1, start 2, stop 1, stop 2, and the source of session 2 was not
  started again when session 1 closed.

**`a_source_pushes_its_events`** (scenario test with fake services, not written yet; verifies SUB-3)

- Given an open session whose fake source is quiet and records each call that it gets.
- When timers and keys of other panels come, and then the source sends one output.
- Then the source got no call while it was quiet, `update` got no message of the session in that
  time, and one message of the session came after the output.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| SUB-4 | Ubiquitous | Each subscription shall have an identity that stays the same while the model asks for the same source. | User, 2026-11-20 |
| SUB-5 | Ubiquitous | Each component shall declare its own subscriptions, and the parent shall collect them and wrap their address. | User, 2026-11-20 |
| SUB-6 | Unwanted | If two components ask for the same source, then the runtime shall start it one time and shall put each event into the message queue for both. | User, 2026-11-20 |
| SUB-7 | Ubiquitous | The runtime shall put an event of a source into the message queue and shall not wait until a component handles it. | User, 2026-11-20 |
| SUB-8 | Ubiquitous | The runtime shall keep the channels of the sources. | User, 2026-11-20 |
| SUB-9 | Ubiquitous | An animation shall not be a subscription. | User, 2026-11-20 |
