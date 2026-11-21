---
status: "draft"
date: "2026-11-20"
sources:
  - "The user, 2026-11-20: the TUI follows The Elm Architecture."
---

# Components, keys and view

Parent: [The architecture of the zendo TUI](tui-architecture.md).

This document describes how the core divides the model, routes keys, and builds the screen.

## Components

- The model shall be a tree of components: `Editor`, `Reader`, `Chat`, `Tree`, `Find`,
  `Terminals`, `Board`, and the status bar.
- Each component shall have its own `Msg`, `update` and `view`.
- A component shall not see the model of another component.
- If a component needs a change in another component, then it shall send an out-message to its
  parent, and the parent shall route it.
- Components shall nest at most two levels.

## Modes, overlays, keys

- `Mode` shall be an enum that holds the data of the mode, so that a state that cannot exist does
  not compile.
- Overlays (questions, tips, the key float, the chip window, the float, the hold view) shall be a
  stack in the model.
- If the overlay stack is not empty, then the top overlay shall get each key.
- The keymap shall turn a key into an action, and the action into a `Msg` of the component in
  focus.

## View and mouse

- `view` shall return a tree of elements. A clickable element shall carry the `Msg` of its click.
- The runtime shall keep the map from rectangles to messages of the last frame.
- When a mouse event comes, the runtime shall turn it into the `Msg` of the element under it. The
  model shall hold no frame cache.
- Expensive results (markdown, syntax highlight, diff) shall go into a cache outside the model,
  keyed by the version of their input.
