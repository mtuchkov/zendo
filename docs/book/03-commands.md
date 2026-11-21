# Chapter 3: Commands (Cmd)

A message describes what *occurred*. A command describes what *to do*. They are a mirror pair.

A **command** is also a usual value: a description of an action. The command itself does nothing.
It is the sheet of paper with the order from the example with the cook.

```rust
enum Cmd {
    DeleteFile(PathBuf),
    WriteFile { path: PathBuf, text: String },
    SendToAgent(agent::Cmd),
    Copy(String),
    Bell,
}
```

The function `update` gets a message and returns a list of commands:

```rust
fn update(model: &mut Model, msg: Msg) -> Vec<Cmd>
```

It changes the state and says: "also do this". The runtime takes the list and does the commands.

**The loop closes.** The result of a command comes back into the core as a new message. An example
with the delete of a file:

1. The user pushes `d`, then `y`. A `Msg::Key` comes.
2. `update` decides to delete the file and returns `Cmd::DeleteFile("notes/a.md")`.
3. The runtime deletes the file on the disk. During this time, the core is free.
4. The runtime sends `Msg::FileDeleted("notes/a.md")` or `Msg::DeleteFailed { .. }`.
5. `update` removes the file from the tree or shows the error.

Why this is good:

1. **A test checks the decision, not the disk.** Give `Msg::Key('y')` and check that `update`
   returned `Cmd::DeleteFile`. You need no temporary folder.

2. **Errors come by the same path.** A failure is only the message `DeleteFailed`. The core handles
   it as it handles all other messages. There are no separate `if let Err` in all of the code,
   where each handler decides itself what to show.

3. **All side effects are visible.** The enum `Cmd` is the full list of all that the application
   can do with the outer world. To find where zendo writes to the disk, look in one place.

One subtle point: some time goes by between step 2 and step 4. While the runtime deletes the file,
the user can push a different key. The core handles this correctly, because it does not wait. The
chapter about tickets tells how the core knows that an answer is old.
