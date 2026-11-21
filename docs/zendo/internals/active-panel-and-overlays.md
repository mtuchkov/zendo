---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "internals.md"
prev: "components.md"
next: "keymap.md"
level: "the one active panel, the mode as a part of its panel, and the stack of overlays"
note: "Draft from the chapter The active panel and overlays of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# The active panel and overlays

Sources: the chapter [The active panel and overlays](../../book/08-active-panel-and-overlays.md) of
the zendo book.

Parent: [Internals of the TUI](internals.md). What the user sees of the focus is in
[Panes and layouts](../panes/panes.md).

## The active panel

The model has one active panel. A key goes to it: see [The keymap](keymap.md).

The mode is a part of the panel. There is no global mode. The editor has its own modes, `Read` and
`Write` ([MODEL-6](model.md)).

## Overlays: a stack on top of the panels

Overlays are on top of all panels: the music question, the tips, the window of the key change, the
chip window, the float, the wait for the end of a turn. An overlay is a panel. When it opens, it
becomes the active panel, and it decides the keys.

The overlays are a stack in the model:

```rust
overlays: Vec<Overlay>
```

The overlay that opened last is on top. The priority is the order of opening: it is visible in the
data, not in the code. The same stack controls the drawing: the view draws the overlays from the
bottom to the top ([LAYOUT-5](layout-and-mouse.md)). To close an overlay is to remove it from the
stack; then the previous panel is active again.

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| ACTIVE-1 | Ubiquitous | The model shall have one active panel at each time. | User, 2026-11-20 |
| ACTIVE-2 | Ubiquitous | The mode shall be a part of the state of its panel, and the model shall have no global mode. | User, 2026-11-20 |
| OVERLAY-1 | Ubiquitous | The overlays shall be one stack in the model, in the order of opening. | User, 2026-11-20 |
| OVERLAY-2 | State | While the overlay stack is not empty, the top overlay shall be the active panel, and the view shall draw the overlays from the bottom of the stack to the top. | User, 2026-11-20 |
| OVERLAY-3 | Event | When an overlay closes, the core shall remove it from the stack, and the panel that was active before it shall be active again. | User, 2026-11-20 |

### Acceptance tests

**`the_mode_belongs_to_its_panel`** (unit test, not written yet; verifies ACTIVE-1, ACTIVE-2)

- Given a model with the editor in write mode as the active panel.
- When the file tree becomes the active panel, gets a key, and the editor becomes active again.
- Then only the tree handled the key, and the editor is still in write mode with its text.

**`the_top_overlay_gets_the_keys_and_is_drawn_on_top`** (unit test, not written yet; verifies OVERLAY-1, OVERLAY-2)

- Given a model where the tips opened first and the window of the key change opened second, in the
  same cells.
- When a key comes that both overlays and the panel under them have an action for.
- Then only the window of the key change handles the key, and the screen shows it on top of the
  tips.

**`a_closed_overlay_gives_the_keys_back`** (unit test, not written yet; verifies OVERLAY-3)

- Given the same model.
- When the window of the key change closes, and then the tips close.
- Then the tips are the active panel after the first close, and the panel that was active before
  the overlays is active after the second.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| OVERLAY-4 | Ubiquitous | An overlay shall be a panel: the rules of a panel for the keys, the modes and the view shall hold for it. | User, 2026-11-20 |
| OVERLAY-5 | Ubiquitous | No code shall hold a fixed order of priority between overlays: the order shall come only from the stack. | User, 2026-11-20 |
