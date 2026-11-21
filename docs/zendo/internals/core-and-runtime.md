---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "internals.md"
next: "messages.md"
level: "the division of the TUI into a core that decides and a runtime that does"
note: "Agent draft from the chapter Core and Runtime of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Core and runtime

Sources: the chapter [Core and Runtime](../../book/01-decide-and-do.md) of the zendo book.

Parent: [Internals of the TUI](internals.md).

## Two parts

**The core decides.** It holds all the state of the application: the open document, the cursor,
the chats, the mode. When an event occurs, the core decides how the state changes and what zendo
does outside. The core does nothing outside itself: it reads no file, uses no network, and reads no
clock.

**The runtime does.** It reads the keyboard, writes to the disk, talks to the agent, and draws on
the screen. It makes no decision of its own: it does what the core tells it. The services (the
agent, the PTY sessions, git, find, the disk, the music, the board) do the IO for the runtime.

The division gives two qualities:

- The core never waits for a slow operation, because no slow operation runs in the core. The disk
  and the network cannot stop the screen.
- A test of the core gives it an event and checks the result. It needs no temporary folder, no
  mock agent and no delay.

The form of an event is in [Messages](messages.md), the form of an order to the runtime in
[Commands](commands.md), and the loop of the runtime in [Async IO](async-io.md).

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| CORE-1 | Ubiquitous | The core shall do no IO: it shall not use the file system, the network, a clock, a process or the asynchronous runtime. | User, 2026-11-20 |
| CORE-2 | Ubiquitous | The core shall hold all the state of the application and shall make each decision: how the state changes when an event occurs, and what the runtime does outside. | User, 2026-11-20 |
| CORE-3 | Ubiquitous | The runtime shall make no decision of its own: it shall call a service only for a command or a subscription of the core, and shall give each result to the core. | User, 2026-11-20 |
| CORE-4 | Ubiquitous | The core shall be testable with values only: a test shall give a model and an event, and shall check the new model and the orders to the runtime, with no file, no agent and no wait. | User, 2026-11-20 |

### Acceptance tests

**`the_core_uses_no_io`** (unit test, not written yet; verifies CORE-1)

- Given the source code of zendo.
- When the test looks in the files of the core for the file system, the network, a clock, a
  process and tokio.
- Then it finds none.

**`a_key_gives_a_new_model_and_an_order`** (unit test, not written yet; verifies CORE-2, CORE-4)

- Given a model with the file tree, the file `notes/a.md` selected, and no folder on the disk.
- When the test gives the keys of the delete and of its confirmation to the core.
- Then the core returns the order to delete `notes/a.md`, the model shows that the delete runs, and
  the test touched no file and waited for nothing.

**`the_runtime_does_only_what_the_core_asks`** (scenario test with fake services, not written yet; verifies CORE-3)

- Given the runtime with fake services that record each call.
- When a scenario of keys, answers and events runs.
- Then each recorded call belongs to a command or a subscription that the core returned in the
  scenario, and each answer of a service came to the core.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| CORE-5 | Ubiquitous | A service shall talk to the core only through the runtime: it gets commands, and it gives messages. | Design |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| The architecture document before this one had `init(Config)`, which gives the first model and the first commands, and the rule "the first frame draws before zendo reads a file". The book does not state the start. Does the rule stay, and how does it fit the start check of ZENDO-2? | CORE-2 | User |
