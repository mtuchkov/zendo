# Chapter 2: Messages (Msg)

The core does nothing itself. Then how does it know that an event occurred? Through messages.

A **message** is a usual value that describes a fact: "what occurred". It is not a command and it is
not a function. It is a record of an event. For example:

- the user pushed `ctrl-s`;
- the agent sent a part of the text;
- the file is read, and here is its content;
- the write to the disk failed;
- 100 ms went by, and here is the time now.

In the code, this is one enum:

```rust
enum Msg {
    Key(KeyEvent),
    AgentText { key: String, text: String },
    FileRead { path: PathBuf, text: String },
    WriteFailed { path: PathBuf, error: String },
    Tick(Instant),
}
```

The main rule: **one door goes into the core**. Each event becomes a `Msg` and goes into one
function, `update`, from all sources (the keyboard, the agent, the disk, a timer). There are no
other inputs.

Why this is important:

1. **One path is easy to understand.** To find what can change the state, look at the enum `Msg`.
   It is the full list of all events that can occur.

2. **The time also comes in a message.** The core does not call `Instant::now()`. The runtime
   brings the time to the core. Thus a test can say "the time is 10:00:05" and check, for example,
   that a button blinks correctly. There is no wait and no randomness.

3. **You can record the messages and play them again.** If the core depends only on messages, then
   the same sequence always gives the same result. Record the log of `Msg` from a user with a bug,
   play it again on your computer, and you get the same bug.

An example from zendo. In this architecture, a key push in the file tree is only a `Msg::Key`.
The core decides "it is necessary to delete the file", but the core does not delete the file
itself. How the core asks for this is the next chapter.
