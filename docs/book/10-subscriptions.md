---
review:
  date: "2026-11-21"
  passes: 1
  result: "pass"
  hash: "sha256:b222dcb2fdb11b812b4ee11a8fe2cf2c24918a0b6d07fc237d6d6e2fe490e2e1"
---
# Chapter 10: Subscriptions

A command is a single action: "delete the file", "send the answer". But many events come as a
**stream**: the agent sends text in parts, a session prints, files on the disk change. Streams need a different thing: a subscription.

**How it is in this architecture.** The core does not poll. It tells **what it must listen to now**:

```rust
fn subscriptions(model: &Model) -> Vec<Sub>
```

This is a pure function of the model. For example:

- the agent is connected → `Sub::Agent`;
- two sessions are open → `Sub::Pty(1)`, `Sub::Pty(2)`;
- search number 17 runs → `Sub::Find(17)`.

Each component declares its own subscriptions, and the parent collects them and wraps their
address, as it does for `Cmd`. The runtime subscribes: it starts and stops the sources and keeps
their channels. The core has no channels. If two components ask for the same source, then the
runtime starts it one time and puts each event into the message queue for both. The runtime does
not wait until a component handles the event: it only puts the event into the queue.

**The runtime compares the lists.** After each `update`, the runtime calls `subscriptions` and
compares the new list with the old list:

- a new subscription → start the source;
- a subscription is gone → stop the source;
- a subscription stays → do not touch the source.

Thus each subscription has an identity (`Pty(2)` is always session 2). Without an identity, it is
not possible to compare.

**The sources push the events themselves.** When a session prints, a message comes. When a file
changes, a message comes. Nothing asks "is there news?". The events come immediately, and only when
they exist.

The main benefit: the model alone decides which sources run. You do not need to remember "start
the source here" and "do not forget to stop it there". The model says "the session is open", and
the source exists. When the session closes, the source stops. It is not possible to forget to stop
it.

An animation is not a subscription, because it does not change the model. The view asks for its
frames. The chapter about the view tells how.
