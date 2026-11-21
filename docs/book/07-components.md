# Chapter 7: Components

The model of zendo is large: the editor, the reader, the chat, the file tree, find, the terminals,
the board, the status bar. Thus the model is divided into **components**.

**A component is a small TEA inside the large one.** Each component has its own:

- model (`tree::Model`);
- messages (`tree::Msg`);
- `update` (`tree::update`);
- `view` (`tree::view`).

You can understand and test a component separately, without the remaining application.

**The parent sends the messages to the components.** The top-level message wraps the message of a
component:

```rust
enum Msg {
    Tree(tree::Msg),
    Chat(chat::Msg),
    Editor(editor::Msg),
    // ...
}
```

The top `update` looks at the wrapper and gives the content to the correct component. It does
nothing more.

**A component does not see its neighbors.** The tree does not know that the chat exists. This is
the main rule of isolation. But sometimes an action in one component must change a different
component. An example: the user deleted a file in the tree, and the chat of this file must also go.

For this, there are **out-messages**. The component does not go into its neighbor. It tells the
parent what occurred:

```rust
// tree::update returns
Out::FileDeleted("notes/a.md")
```

The parent gets this and decides itself: "then the chat gets
`chat::Msg::DropThread("notes/a.md")`". The knowledge about the links between components is only
in the parent, in one place.

**Commands also come back to the correct address.** If the tree asked to delete a file, then the
answer `FileDeleted` must come back into the tree. The parent wraps the result in `Msg::Tree(..)`,
and the result goes to the component that asked.

**A balanced tree.** Each level of depth has a readability penalty: one more wrapper and one more
step for each message. Thus keep the tree balanced. When a parent has too many children, put
related children under a new parent. Group the components by meaning: components that often send
messages to each other go under the same parent. Then an out-message goes only to the nearest
common parent, not to the root.
