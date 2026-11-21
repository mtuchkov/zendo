---
review:
  reviewer: "agent, new context"
  date: "2026-11-20"
  passes: 1
  result: "pass"
  hash: "sha256:1f8f95b6ef17fc8a165790a698472e90529773bed1a2b25f237f3b65768c5ca6"
---
# Chapter 11: Async IO: the UI loop never waits

This topic is large, thus it has two parts. This part tells how the loop works and which part does
the commands. The chapter about tickets and cancellations tells what to do with answers that come late.

**The UI loop**

The loop of the runtime does the same steps again and again:

1. It waits for a message or for the frame timer. A message is a key, an answer of a command, or an
   event of a subscription.
2. It takes **all** the messages that are ready, and gives each message to `update`.
3. It gives the commands from `update` to the executor.
4. It draws the frame one time.

It is important what the loop does **not** do: it never waits for a command to finish. It gives
the command to the executor and continues immediately. Thus a slow disk, git, or the agent cannot
stop the screen.

Step 2 is also important. The agent streams text word by word, and a trackpad sends tens of scroll
events. If the loop draws after each event, then there are too many frames. Thus the runtime first
applies all the events that are waiting, and then it draws one time.

**The executor**

The executor gets a command and starts it as a tokio task. The task works separately. When the task
is complete, the result comes into the loop as a message.

A trap: in Rust, "async" does not mean "does not block". Many APIs are synchronous: `std::fs`, the
walk of files with `ignore`, the highlight with `syntect`, the diff with `similar`, the PTY read,
the keyboard read. If a tokio task calls such an API directly, the API holds a tokio thread and can
make other tasks slow. Thus the rule is:

- a blocking call goes to `spawn_blocking` or to a thread of its own;
- the PTY and the keyboard each read in a permanent thread of their own and only wake the loop.

**Heavy calculations are also IO**

For the UI, there is no difference: it waits for the disk, or it calculates the markdown of a large
file for 200 ms. In both cases, the frame is late. Thus a heavy calculation is also a command:
`Cmd::RenderMarkdown`, `Cmd::Diff`, `Cmd::Highlight`. While there is no result, the view shows the
old result from the cache or a placeholder. The user sees the screen immediately, and the exact
result comes a moment later.
