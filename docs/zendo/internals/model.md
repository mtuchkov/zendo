---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "internals.md"
prev: "commands.md"
next: "view.md"
level: "the model as the one place of the state: what it holds, who changes it, and its shape"
note: "Agent draft from the chapter The model (Model) of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# The model

Sources: the chapter [The model (Model)](../../book/04-model.md) of the zendo book.

Parent: [Internals of the TUI](internals.md).

## All the state in one place

The model holds all the state of the application: the open documents, the cursor, the selection,
the chats, the mode, the size of the screen, and the write queue. If a thing has an effect on what
the user sees, then it is in the model. The screen keeps no state of its own
([VIEW-3](view.md)), so the screen and the data cannot differ.

**The model is data only.** The handle of the agent, a PTY, the find thread: these live in the
runtime and in the services. The model keeps only what is known about them: "the agent is
connected", "session 3 is busy", "search number 17 runs".

**The model changes in place.** `update` gets `&mut Model` and changes it directly, because a new
copy of a large document for each message is slow. A pure core means "no IO and no clock"
([CORE-1](core-and-runtime.md)), not "no change of data". The order stays strict: only `update`
changes the model, and only as the response to a message.

## The shape prevents errors

A state that cannot exist does not compile. The float exists only in write mode, so it is inside
the variant of that mode:

```rust
enum Mode {
    Read { cursor: Cursor, sel: Option<Selection> },
    Write { editor: Editor, float: Option<Float> },
}
```

No code checks "the mode is Read, and the float is open": the model cannot hold that state.

The model is large, so it is a tree of parts: see [Components](components.md).

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| MODEL-1 | Ubiquitous | The model shall hold all the state that has an effect on what the user sees, and shall be the only source of that state. | User, 2026-11-20 |
| MODEL-2 | Ubiquitous | The model shall be data only: it shall hold no channel, no handle of a task, a thread or a process, and no file. | User, 2026-11-20 |
| MODEL-3 | Ubiquitous | Only `update` shall change the model, and only as the response to a message. | User, 2026-11-20 |

### Acceptance tests

**`a_copy_of_the_model_gives_the_same_screen`** (unit test, not written yet; verifies MODEL-1)

- Given the model at the end of a scenario with an open document, a selection and a chat.
- When the test copies the model and draws the copy with the same cache and the same time.
- Then the screen of the copy is equal to the screen of the scenario.

**`the_model_holds_no_handle`** (unit test, not written yet; verifies MODEL-2)

- Given the source code of the core.
- When the test looks in the types of the model for a channel, a handle of a task, a thread or a
  process, and a file.
- Then it finds none.

**`only_update_changes_the_model`** (unit test, not written yet; verifies MODEL-3)

- Given the source code of the core.
- When the test looks for the functions that get the model for a change.
- Then each one is `update` or a function that only `update` calls; `view`, `layout` and
  `subscriptions` get the model for a read only.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| MODEL-4 | Ubiquitous | `update` shall change the model in place and shall not build a new model for each message. | User, 2026-11-20 |
| MODEL-5 | Ubiquitous | For a thing of the runtime or of a service, the model shall keep only what is known about it, such as "connected", "busy", or the number of the search that runs. | User, 2026-11-20 |
| MODEL-6 | Ubiquitous | The types of the model shall not be able to hold a state that cannot exist: data that exists only in one mode shall be inside the variant of that mode. | User, 2026-11-20 |
