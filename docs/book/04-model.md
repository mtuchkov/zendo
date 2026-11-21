---
review:
  date: "2026-11-21"
  passes: 1
  result: "pass"
  hash: "sha256:0eb7968af73339dbf3e9198498f605d8d05c326250ccf80c9b7427ff52726d26"
---
# Chapter 4: The model (Model)

The model is all the state of the application in one place. It has the open documents, the cursor,
the selection, the chats, the mode, the size of the screen and the write queue. If a thing has an effect
on what the user sees, it is in the model.

**The model is only data.** It has no channels, no handles of processes, and no file descriptors.
The handle of the agent, the PTY, the find thread: all of these live in the runtime and in the
services. The model keeps only what is known about them: "the agent is connected", "session 3 is
busy", "search number 17 runs".

**The model is the only source of truth.** The screen keeps no state of its own. The screen is
always built again from the model. Thus the screen and the data cannot be different.

**The model changes in place.** In Elm, the model cannot change: `update` returns a new model. In
Rust, this is not convenient, and it is slow for large documents. Thus `update` gets `&mut Model`
and changes it directly. Here, a "pure" core means "no IO and no clock", not "no changes of data".
The order is still strict: only `update` can change the model, and only as a response to a message.

**The shape of the model prevents errors.** Look at the mode of the editor. You can keep it like this:

```rust
mode: Mode,              // Read or Write
float: Option<Float>,    // is the float open?
```

Then the state "the mode is Read, and the float is open" is possible, but the float occurs only in
Write. The code must check this in all places. This is better:

```rust
enum Mode {
    Read { cursor: Cursor, sel: Option<Selection> },
    Write { editor: Editor, float: Option<Float> },
}
```

Now the float can be only inside Write. An impossible state does not compile. It is not necessary
to check it.

The model is large, thus it is divided into parts: the editor, the chat, the tree, and others. The
chapter about components tells how it is divided. For now, the main point is this: the state is one,
it is in one place, and only `update` changes it.
