---
status: "draft"
revision: "1"
date: "2026-11-21"
parent: "internals.md"
prev: "active-panel-and-overlays.md"
next: "subscriptions.md"
level: "the way of a key to a message: the key, the action, the active panel, the global keys, and sequences with time"
note: "Agent draft from the chapter The keymap of the zendo book, the target architecture (User, 2026-11-20). Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
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

**Global keys.** A global key works in all panels, in all modes and in all overlays, with no
exception. A key that does not work in one place is not global: it is a key of each panel, and the
panels use the same key for the same action. A text input needs the printable symbols, so only
combinations such as `ctrl-…` and `alt-…` can be global.

`/` is such a key of the panels, not a global key (User, 2026-11-21): it opens the find panel where
the panel takes no text (FIND-22), and in write mode it is a symbol (FIND-23).

**Sequences with time.** A run of `tab` is `{ count, last_at }` in the model of the panel. The
time comes in the message of the key ([MSG-3](messages.md)), so "is the pause long enough" is a
comparison in `update`.

## Architecturally significant requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| KEYMAP-1 | Event | When a key comes, the core shall take its action from the keymap, and then the message of that action from the active panel; a component shall work with actions and shall not know the keys. | User, 2026-11-20 |
| KEYMAP-2 | Event | When a key comes, the core shall look for its action only in the keymap of the active panel, for the mode of that panel, and in the global keymap. | User, 2026-11-20 |
| KEYMAP-3 | Ubiquitous | A global key shall work in each panel, each mode and each overlay, with no exception. | User, 2026-11-20 |
| KEYMAP-4 | Ubiquitous | A global key shall not be a letter or a different printable symbol. | User, 2026-11-20 |

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

**`a_global_key_works_everywhere`** (unit test, not written yet; verifies KEYMAP-3)

- Given each panel, each of its modes, and each overlay, one after the other as the active panel.
- When each key of the global keymap comes.
- Then the core gives the message of its global action each time.

**`no_global_key_is_printable`** (unit test, not written yet; verifies KEYMAP-4)

- Given the default keymap file.
- When the test reads the keys of the global actions.
- Then no key is a printable symbol without `ctrl` or `alt`.

## Detailed requirements

| ID | Pattern | Requirement | Source |
| -- | ------- | ----------- | ------ |
| KEYMAP-5 | Ubiquitous | The keymap shall be data from the config, and the core shall hold no key in its code (PANES-17, PANES-48). | User, 2026-11-20 |
| KEYMAP-6 | Unwanted | If the keymap of the active panel and the global keymap have no action for a key, then the core shall change nothing. | User, 2026-11-20 |
| KEYMAP-7 | Ubiquitous | The keymap of a panel shall depend on the mode of the panel. | User, 2026-11-20 |
| KEYMAP-8 | Ubiquitous | A sequence of keys with time shall be state of its panel in the model, and `update` shall compare the time of the key message with the time of the last key. | User, 2026-11-20 |
| KEYMAP-9 | Unwanted | If something must occur at the end of a pause, with no new key, then a timer message shall do it. | User, 2026-11-20 |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| What does zendo do when the keymap of the user gives a printable symbol to a global action: keep the default key and tell the user, as PANES-19 does? | KEYMAP-4 | User |
| `esc` stays fixed and is not in the keymap (PANES-48). Is it a key of each panel, or a global key outside the keymap? | KEYMAP-3, KEYMAP-5 | User |
