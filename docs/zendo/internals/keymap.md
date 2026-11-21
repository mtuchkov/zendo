---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "internals.md"
prev: "active-panel-and-overlays.md"
next: "subscriptions.md"
level: "the way of a key to a message: the key, the action, the active panel, the global keys, and sequences with time"
note: "Draft from the chapter The keymap of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# The keymap

Sources: the chapter [The keymap](../../book/09-keymap.md) of the zendo book.

Parent: [Internals of the TUI](internals.md). The keymap files, the hints and the help that the
user sees are in [Keys, hints and help](../panes/keymap.md).

## The key separately from the action

One key can have different meanings: in the tree, `y` means "yes, delete"; in the editor, it is a
letter. A key becomes a message in two steps:

1. Key → action, from the keymap: `ctrl-s` → `"save"`. The keymap is data from the config.
2. Action → message of the active panel: `"save"` in the editor → `Msg::Editor(Save)`.

"Which key" and "what to do" are different questions. When the user changes a key, only the keymap
changes. The components work with actions and do not know about keys. A test sends
`Msg::Editor(Save)` directly.

## Where a key goes

A key goes to the keymap of the [active panel](active-panel-and-overlays.md) or to the global
keymap. If neither has an action for the key, then nothing occurs. The keymap of a panel depends on
its mode.

**Global actions.** A global action works in all panels, in all modes and in all overlays, with no
exception. A key that does not work in one place is not global: it is a key of each panel, and the
panels use the same key for the same action.

The key of a global action can be a printable symbol (User, 2026-11-21). A panel that takes text
needs the printable symbols for the text. So, in such a panel, the symbol alone is text, and the
action needs the symbol together with `ctrl` or `alt`. This decision is newer than the chapter,
which lets only combinations with `ctrl` and `alt` be global.

`/` is a key of the panels, not a global key (User, 2026-11-21): it opens the find panel where
the panel takes no text (FIND-22), and in write mode it is a symbol (FIND-23).

`esc` is the single exception to "the keymap is data": it is fixed, and it is not in the keymap
(PANES-48; User, 2026-11-21).

**Sequences with time.** A run of `tab` is `{ count, last_at }` in the model of the panel. The
time comes in the message of the key ([MSG-3](messages.md)), so "is the pause long enough" is a
comparison in `update`.

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| KEYMAP-1 | Event | When a key comes, the core shall take its action from the keymap, and then the message of that action from the active panel; a component shall work with actions and shall not know the keys. | User, 2026-11-20 |
| KEYMAP-2 | Event | When a key comes, the core shall look for its action only in the keymap of the active panel, for the mode of that panel, and in the global keymap. | User, 2026-11-20 |
| KEYMAP-3 | Ubiquitous | A global action shall work in each panel, each mode and each overlay, with no exception. | User, 2026-11-20 |
| KEYMAP-4 | Optional | Where the key of a global action is a printable symbol, a panel or a mode that takes text shall take the symbol alone as text, and shall do the action for the symbol together with `ctrl` or `alt`. | User, 2026-11-21 |

### Acceptance tests

**`a_changed_key_changes_no_component`** (unit test, not written yet; verifies KEYMAP-1)

- Given the editor as the active panel, a keymap with `ctrl-s` for `save`, and a second keymap with
  `alt-w` for `save`.
- When `ctrl-s` comes with the first keymap, and `alt-w` with the second.
- Then the editor gets the same save message both times.

**`a_key_goes_to_the_active_panel_or_to_the_global_keymap`** (unit test, not written yet; verifies KEYMAP-2)

- Given a model with the delete question of the tree, and then with the editor in write mode.
- When `y` comes in each, and then a key that has no action.
- Then the tree confirms the delete, the editor adds the letter, and the key with no action
  changes no model and gives no command.

**`a_global_action_works_everywhere`** (unit test, not written yet; verifies KEYMAP-3)

- Given each panel, each of its modes, and each overlay, one after the other as the active panel.
- When the key of each global action comes, with `ctrl` or `alt` where the panel takes text.
- Then the core gives the message of the global action each time.

**`a_printable_global_key_needs_ctrl_or_alt_where_text_goes`** (unit test, not written yet; verifies KEYMAP-4)

- Given a keymap with a global action on `?`, and on `alt-?` for the panels that take text.
- When `?` comes in the file tree, then `?` and `alt-?` come in the editor in write mode.
- Then the tree gives the message of the action, the editor adds the symbol `?` to the text, and
  `alt-?` gives the message of the action.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| KEYMAP-5 | Ubiquitous | The keymap shall be data from the config, and the core shall hold no key in its code, with the single exception of `esc` (PANES-17, PANES-48). | User, 2026-11-20; User, 2026-11-21 |
| KEYMAP-6 | Unwanted | If the keymap of the active panel and the global keymap have no action for a key, then the core shall change nothing. | User, 2026-11-20 |
| KEYMAP-7 | Ubiquitous | The keymap of a panel shall depend on the mode of the panel. | User, 2026-11-20 |
| KEYMAP-8 | Ubiquitous | A sequence of keys with time shall be state of its panel in the model, and `update` shall compare the time of the key message with the time of the last key. | User, 2026-11-20 |
| KEYMAP-9 | Unwanted | If something must occur at the end of a pause, with no new key, then a timer message shall do it. | User, 2026-11-20 |
| KEYMAP-10 | Unwanted | If the keymap gives a printable symbol to a global action, and no key with `ctrl` or `alt` for the panels that take text, then the system shall keep the default keys of that action and tell the user (PANES-19). | Design |
