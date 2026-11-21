---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "internals.md"
prev: "async-io.md"
next: "frame-and-budget.md"
level: "answers that come late or in the wrong order: the ticket for correctness, the cancellation for speed"
note: "Agent draft from the chapter Tickets and cancellations of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Tickets and cancellations

Sources: the chapter [Tickets and cancellations](../../book/12-tickets-and-cancellations.md) of
the zendo book.

Parent: [Internals of the TUI](internals.md).

## Answers come at the wrong time

When all IO is [asynchronous](async-io.md), answers come late, and sometimes in the wrong order.
The user must always see a consistent view: the screen shows only results of the input that is on
the screen now.

An example with find: the user types `ab`, and the core sends search 1. The user adds `c`, and the
core sends search 2 for `abc`. Search 2 finishes first, and the screen shows the results for `abc`.
Then the slow search 1 finishes. If the core applies it, the screen shows the results for `ab`
under the field `abc`.

## One type for a field that a command fills

The core has one generic type, `Latest<T>`. A component uses it for each field of its model that a
command fills. It keeps only the result of the latest request.

```rust
pub struct Latest<T> {
    value: T,             // private
    ticket: Option<u64>,  // the request that the field waits for
    next: u64,
}
```

`start()` gives a new ticket; the command takes the ticket, and its answer returns it.
`resolve(ticket, value)` writes the value only if the ticket is the latest one. The value is
private, so there is no other way to write it. A component does not compare tickets itself.

## Cancellation

To discard an old answer is correct, but the work is already done: ten unnecessary searches load
the CPU. So a new ticket also cancels the task of the old one. The two functions stay separate:
the ticket is the guarantee of correctness, and it always works; the cancellation only saves work.
A task can finish before its cancellation, and then its answer comes all the same.

An abort handle stops a task only where the task waits. A blocking call
([ASYNC-3](async-io.md)), such as a search, does not wait: it checks a stop flag between its steps
(User, 2026-11-21).

For a stream, the runtime stops the source when its subscription is gone
([SUB-2](subscriptions.md)).

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| TICKET-1 | Ubiquitous | The core shall apply the result of a command only if the model still waits for that result. | User, 2026-11-20 |
| TICKET-2 | Ubiquitous | Each field of the model that a command fills shall have the one generic type of the core that keeps only the result of the latest request, and no other code shall be able to write the value of the field. | User, 2026-11-20 |
| TICKET-3 | Ubiquitous | The correctness of the screen shall not depend on a cancellation: the ticket shall discard the answer of a cancelled task that comes all the same. | User, 2026-11-20 |

### Acceptance tests

**`a_late_answer_of_an_old_request_does_not_show`** (unit test, not written yet; verifies TICKET-1)

- Given the find panel with the query `ab` and its search 1, then the query `abc` and its search 2.
- When the answer of search 2 comes, and then the answer of search 1.
- Then the panel shows the results for `abc` after both messages.

**`a_field_takes_only_the_latest_answer`** (unit test, not written yet; verifies TICKET-2)

- Given a field of the type `Latest` with two requests started, and the source code of the core.
- When the test resolves the first ticket, and then the second.
- Then the first value is discarded and the second is written; and each field that the answer of a
  command fills has the type `Latest`.

**`an_answer_after_a_cancellation_is_discarded`** (unit test, not written yet; verifies TICKET-3)

- Given a field with request 1, and then request 2, which gives the command to cancel request 1.
- When the answer of request 1 comes all the same.
- Then the field does not take it, and still waits for request 2.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| TICKET-4 | Event | When a component starts a request, the field shall give a new ticket; the command shall hold the ticket, and its answer shall return it. | User, 2026-11-20 |
| TICKET-5 | Event | When an answer comes, the field shall write the value only if the ticket of the answer is the latest one, and shall discard the value if not. | User, 2026-11-20 |
| TICKET-6 | Event | When a field gives a new ticket while it waits for an older one, the field shall also give the command to cancel the task of the older ticket. | User, 2026-11-20 |
| TICKET-7 | Event | When a component closes, the runtime shall cancel the tasks of all its tickets. | User, 2026-11-20 |
| TICKET-8 | Ubiquitous | The runtime shall cancel a task through the abort handle of the task. | User, 2026-11-20 |
| TICKET-9 | Ubiquitous | A result for the cache of the view shall need no ticket: its key shall be the hash of its input. | User, 2026-11-20 |
| TICKET-10 | Ubiquitous | A blocking call that can run long shall check a stop flag between its steps, and the cancellation of its ticket shall set that flag. | User, 2026-11-21 |
