---
status: "draft"
revision: "1"
date: "2026-11-14"
parent: "panes.md"
prev: "selection.md"
next: "key-change.md"
level: "where the keys of zendo come from, and how the status bar and the help show them"
note: "Agent draft, split from panes.md on 2026-11-14; no requirement changed. On 2026-11-17 the user changed PANES-18: a key for each frequently used action, not for every action; on 2026-11-18, frequent by the usage counts from real use, and every key in the keymap except `esc` (PANES-48); the user approved PANES-18 and PANES-48 on 2026-11-18. The other requirements are not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Keys, hints and help

Each frequently used action of zendo has a key, by the usage counts from real use; other actions can have only a button or a mouse
action, such as `[OK]` of the tip menu or a double click on a key. See [Panes and layouts](panes.md) for the
panes.

**The keymap.** zendo ships a default keymap file. The keys of an action come from that file, then
from the keymap file that the config names, then from `[keys.<pane>]` in the config. `zendo keymap`
prints the default file, so that the user can start a keymap of their own from it. zendo tells the
user what it does not understand in a keymap, and keeps the default keys.

**Hints.** The status bar shows the key of the help first, then the key hints of the pane or mode,
and its right side whole. Each hint, message and help text shows the key of the keymap in effect.
An action without a key gets no hint.

**Help.** `alt-?` (or `f1`, and `?` in read mode) shows the keys of the pane in focus, or of the
mode, and the global keys, in groups. It also shows the keymap file and the version of zendo. A double click on a key in the
status bar or in the help changes that key: see [Change a key](key-change.md).

The keys of the search box, the theme list, the question of a file and the file name filter of the
tree are in the keymap too. Only `esc` is fixed: it leaves a mode, a float or a box.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| PANES-18 | Ubiquitous | The system shall have a key for each action that the usage counts from real use show as frequently used; other actions can have only a button or a mouse action. | User, 2026-11-17; User, 2026-11-18 |
| PANES-48 | Ubiquitous | The system shall take every key from the keymap, also the keys of the search box, the theme list, the question of a file and the file name filter of the tree, except `esc`, which stays fixed. | User, 2026-11-18 |
| PANES-17 | Ubiquitous | The system shall take the keys of every action of every pane from the default keymap file that it ships, then from the keymap file that the config names, then from `[keys.<pane>]` in the config. | User, 2026-11-11 |
| PANES-19 | Unwanted   | If `[keys]` in the config names an unknown pane, action or key, then the system shall keep the default keys and tell the user.       | Design           |
| PANES-23 | Event      | When the user runs `zendo keymap`, the system shall print the default keymap file, so that the user can start a keymap of their own from it. | User, 2026-11-12 |
| PANES-24 | Unwanted   | If one key has two actions in one pane of the keymap, then the system shall tell the user, and the key shall do the first action. | User, 2026-11-12 |
| PANES-25 | Unwanted   | If the terminal has no kitty keyboard protocol and sends `ctrl-shift` and a letter as `ctrl` and the letter, then the system shall do the action of `ctrl-shift` and the letter when `ctrl` and the letter has no action. | User, 2026-11-12 |
| PANES-28 | Ubiquitous | The system shall take the key in each key hint, message and help text from the keymap in effect, for each action that the keymap has. | User, 2026-11-13 |
| PANES-29 | Unwanted   | If an action has no key in the keymap in effect, then the system shall show no hint for that action. | User, 2026-11-13 |
| PANES-26 | Ubiquitous | The system shall show the key of the help first in the status bar, in every pane and mode.                                          | User, 2026-11-12 |
| PANES-27 | Ubiquitous | The system shall show the right side of the status bar whole: the last key hints give way to it, and the key of the help stays.     | User, 2026-11-12 |
| PANES-33 | State      | While the chat pane has the focus, the system shall name it `CHAT` in the badge of the status bar, and `SHELL` while a shell terminal shows in the chat pane. | User, 2026-11-13 |
| PANES-16 | Event      | When the user presses `alt-?` or `f1` in any pane or mode, the system shall show the keys of the pane in focus, or of the mode, and the global keys. | User, 2026-11-12 |
| PANES-30 | Ubiquitous | The system shall show the actions of each pane in groups, in the help and in the default keymap file: navigation, text selection, editing, features, sessions and terminals, panes and view, zendo. | User, 2026-11-13 |
| PANES-31 | Event      | When the system shows the help, the system shall show the keymap file that the config names, and the version of zendo; the commit and the local copy only for a build with uncommitted changes. | User, 2026-11-13 |
| PANES-32 | Unwanted   | If the user presses `ctrl-c`, then the system shall not quit: `ctrl-c` copies. | User, 2026-11-13 |

Tests that verify detailed requirements: `the_default_keymap_file_names_every_action` (PANES-17),
`a_user_keymap_file_then_the_config_change_keys`, `zendo_keymap_prints_the_default_keymap` (PANES-17,
PANES-23), `a_key_of_two_actions_is_a_problem` (PANES-24), `without_the_kitty_protocol_ctrl_hjkl_select`
(PANES-25), `the_status_bar_asks_for_zendo_init_without_a_structure_file` (PANES-27), the tests of
`tests/keymap_hints.rs` (PANES-28, PANES-29),
`the_keymap_file_groups_the_actions_as_the_help_does` and `each_group_of_a_pane_has_one_place`
(PANES-30), `the_help_shows_groups_the_keymap_file_and_the_version` (PANES-30, PANES-31),
`the_help_shows_the_details_of_the_version_only_for_a_build_with_changes` (PANES-31),
`ctrl_c_does_not_quit` (PANES-32), `alt_enter_is_a_new_line_and_the_status_bar_says_so` and
`a_terminal_shows_in_the_chat_pane` (PANES-33).

The identifiers keep the topic `PANES`: they came from [Panes and layouts](panes.md).
