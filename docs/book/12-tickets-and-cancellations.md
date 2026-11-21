---
review:
  reviewer: "agent, new context"
  date: "2026-11-20"
  passes: 1
  result: "pass"
  hash: "sha256:a3defa4c368c2f0fc2a4e6605937dea6f713378bfea94d5e7d9ee5bba7ad72fc"
---
# Chapter 12: Tickets and cancellations

When all is asynchronous, a new problem occurs: answers come **at the wrong time**. Sometimes they
come late, and sometimes they come in the wrong order. This chapter tells how the user always sees
a consistent view: the screen shows only results of the input that is on the screen now.

**An example with find**

1. The user types `ab`. The core sends search number 1.
2. Immediately, the user adds `c`. The core sends search number 2 for `abc`.
3. Search number 2 is short, and it finishes first. The screen shows the results for `abc`. This is
   correct.
4. Then search number 1, which is slow, finishes. If the core simply applies it, then the screen
   shows the results for `ab`, but the field shows `abc`.

This is a classic race. In synchronous code, it does not occur. In asynchronous code, it always
occurs.

**The solution: one type for a field that a command fills**

The core has one generic type, `Latest<T>`. A component uses it for each field of its model that a
command fills. It keeps only the result of the latest request.

```rust
pub struct Latest<T> {
    value: T,             // private
    ticket: Option<u64>,  // the request that the field waits for
    next: u64,
}
```

- `start()` gives a new ticket. The command takes the ticket, and its
  answer returns it.
- `resolve(ticket, value)` writes the value only if the ticket is the latest one. If not, it
  discards the value.
- The value is private, thus there is no other way to write it.

A component does not compare tickets itself. It only declares the field as `Latest<T>`. In the
example with find, the results are `Latest<Vec<Item>>`. The answer to search number 1 has an old
ticket, thus `resolve` discards it.

The rule is simple: the core applies a result only if the model still waits for it.

A result for a cache needs no ticket. The key of the cache is the hash of the input. An old result
is still correct for its key, but nothing asks for it.

**Cancellation**

To discard an old answer is correct, but the work is already done. If the user types fast, then ten
unnecessary searches load the CPU. Thus, when `start()` gives a new ticket, it also gives `Cmd::Cancel`
for the old ticket. When a component closes, the runtime cancels all its tickets. The runtime
**cancels** the task through `AbortHandle`.

It is important to keep the two functions separate:

- **the ticket** is the guarantee of correctness. It always works;
- **the cancellation** only saves work. The task can finish before the cancellation, and then its
  answer comes all the same. `resolve` discards it.

Thus do not use the cancellation for correctness. Use it only for speed. For streams, it is even
more simple: when a subscription is gone from the list, the runtime stops the source itself.
