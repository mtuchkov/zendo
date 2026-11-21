---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "internals.md"
prev: "layout-and-mouse.md"
next: "active-panel-and-overlays.md"
level: "the division of the model into components, their isolation, and the messages between them through the parent"
note: "Agent draft from the chapter Components of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Components

Sources: the chapter [Components](../../book/07-components.md) of the zendo book.

Parent: [Internals of the TUI](internals.md).

## A small core inside the large one

The [model](model.md) of zendo is large: the editor, the reader, the chat, the file tree, find,
the terminals, the board, the status bar. It is divided into components. Each component has its
own model (`tree::Model`), messages (`tree::Msg`), `update` and `view`. A person can understand
and test a component without the remaining application.

**The parent sends the messages.** The message of the parent wraps the message of a component:
`Msg::Tree(tree::Msg)`. `update` of the parent looks at the wrapper and gives the content to the
component. It does nothing more.

**A component does not see its neighbors.** The tree does not know that the chat exists. When an
action in one component must change a different one, the component tells its parent what occurred,
with an out-message:

```rust
// tree::update returns
Out::FileDeleted("notes/a.md")
```

The parent decides: the chat gets `chat::Msg::DropThread("notes/a.md")`. The knowledge about the
links between components is only in the parent.

**A result comes back to its address.** When the tree asks for the delete of a file, the parent
wraps the result in `Msg::Tree(..)`, and the result goes to the tree.

**A balanced tree.** Each level of depth costs one more wrapper for each message. Components that
often send messages to each other go under the same parent, so that an out-message goes only to
the nearest common parent, not to the root.

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| COMP-1 | Ubiquitous | The model shall be a tree of components, and each component shall have its own model, messages, `update` and `view`. | User, 2026-11-20 |
| COMP-2 | Ubiquitous | A component shall not read and shall not change the model of a different component. | User, 2026-11-20 |
| COMP-3 | Event | When an action in a component needs a change in a different component, the component shall return an out-message to its parent, and the parent shall decide which messages its other children get. | User, 2026-11-20 |
| COMP-4 | Ubiquitous | A component shall be testable alone: a test shall give its model and a message, and shall check its model, its commands and its out-messages, with no other component. | User, 2026-11-20 |

### Acceptance tests

**`a_component_runs_alone`** (unit test, not written yet; verifies COMP-1, COMP-4)

- Given the model of the file tree with two files, and no other part of the application.
- When the test gives the tree the messages of the delete of one file and of its confirmation.
- Then the `update` of the tree returns the delete command, and its model shows that the delete
  runs.

**`a_component_names_no_neighbor`** (unit test, not written yet; verifies COMP-2)

- Given the source code of the core.
- When the test looks in the files of each component for a type of a sibling component.
- Then it finds none.

**`the_parent_routes_an_out_message`** (unit test, not written yet; verifies COMP-3)

- Given a model with the file `notes/a.md` in the tree and a chat thread of that file.
- When the message that the file is deleted comes to the tree.
- Then the tree returns the out-message of the deleted file, the parent gives the chat the message
  to drop the thread, and the chat has no thread of that file.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| COMP-5 | Ubiquitous | The message of a parent shall wrap the message of each child, and `update` of the parent shall give the content to that child and do nothing more with it. | User, 2026-11-20 |
| COMP-6 | Event | When a command of a component ends, the parent shall wrap the result as a message of that component, so that the result comes to the component that asked. | User, 2026-11-20 |
| COMP-7 | Ubiquitous | The links between components shall be only in the code of their nearest common parent. | User, 2026-11-20 |
| COMP-8 | Ubiquitous | Components that send out-messages to each other shall have the same parent, so that no out-message goes through the root for two components of one group. | User, 2026-11-20 |
