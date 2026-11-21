---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "internals.md"
prev: "messages.md"
next: "model.md"
level: "the command as the one output of the core to the outer world, and the way of its result back into the core"
note: "Draft from the chapter Commands (Cmd) of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Commands

Sources: the chapter [Commands (Cmd)](../../book/03-commands.md) of the zendo book.

Parent: [Internals of the TUI](internals.md). The commands of the command line (`zendo setup`,
`zendo doctor`) are a different topic: see [Commands](../cli/cli.md), with the IDs `CMD`.

## A command is an order

A [message](messages.md) describes what occurred. A command describes what to do. A command is a
value: a description of an action. It does nothing itself. Examples: delete a file, write a file,
send to the agent, copy to the clipboard, ring the bell.

`update` gets a message, changes the model, and returns a list of commands:

```rust
fn update(model: &mut Model, msg: Msg) -> Vec<Cmd>
```

The runtime takes the list and does the commands. All commands are one enum, `Cmd`: it is the full
list of what zendo does with the outer world.

## The loop closes

The result of a command comes back into the core as a new message. For the delete of a file:

1. The user confirms the delete. A message of the key comes.
2. `update` returns the command to delete `notes/a.md`.
3. The runtime deletes the file. During this time, the core is free ([ASYNC-1](async-io.md)).
4. The runtime sends the message that the file is deleted, or the message that the delete failed.
5. `update` removes the file from the tree, or shows the failure.

A failure comes by the same path as each other event. `update` decides what the user sees; the
[failure default](../failures.md) says what that is. The answer can come late or in the wrong
order: see [Tickets and cancellations](tickets-and-cancellations.md).

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| COMMAND-1 | Ubiquitous | Each effect of zendo on the outer world shall be a command: a value that `update` returns and that the runtime does. | User, 2026-11-20 |
| COMMAND-2 | Event | When a command ends, the runtime shall send its result to the core as a message, for a success and for a failure. | User, 2026-11-20 |

### Acceptance tests

**`a_decision_is_a_command`** (unit test, not written yet; verifies COMMAND-1)

- Given a model with the file tree and the file `notes/a.md` selected, and no folder on the disk.
- When the test gives the messages of the keys of the delete and of its confirmation.
- Then `update` returns one command, the delete of `notes/a.md`, and the tree still shows the file.

**`the_result_of_a_command_comes_back_as_a_message`** (scenario test with fake services, not written yet; verifies COMMAND-2)

- Given the runtime with a fake disk that deletes the first file and fails for the second.
- When the user deletes two files.
- Then the tree does not show the first file, shows the second, and the screen shows the failure of
  the second delete once.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| COMMAND-3 | Ubiquitous | The commands shall be one enum, so that the enum lists each effect of zendo on the outer world. | User, 2026-11-20 |
| COMMAND-4 | Ubiquitous | A command shall hold data only: it shall describe the action and shall not do it. | User, 2026-11-20 |
| COMMAND-5 | Unwanted | If a command fails, then only `update` shall decide what the user sees: the code that asked for the command shall have no other path for the error. | User, 2026-11-20 |
| COMMAND-6 | Ubiquitous | `update` shall return its commands and shall not wait for the result of one. | User, 2026-11-20 |
