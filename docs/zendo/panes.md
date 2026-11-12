---
status: "draft"
revision: "1"
date: "2026-11-11"
parent: "zendo.md"
next: "notes.md"
level: "the panes as a whole: layouts, widths, hiding, focus, mouse selection, themes"
note: "Agent rewrite of the reference of zendo 0.9.1. Not reviewed by the user."
---

# Panes and layouts

zendo has four panes: the file tree, the note, the chat with Claude and the delta. The note is the
main pane. The other panes open and close on the request of the user. Each pane has its own
document for what it shows: [Notes](notes.md), [File tree](files.md), [Claude](claude.md),
[Work, delta and recaps](work.md).

The user has two layout presets and no other layout settings. In wide screen, the open panes stand
side by side. In standard, the note and the chat share the screen, and the delta takes the place of
the note. The top bar shows the file and the branch; there are no buttons, every action has a key.

Keys in every pane: `tab` moves the focus to the next pane, `ctrl-shift-b` or `alt-b` opens or closes the file
tree,
`ctrl-←` and `ctrl-→` move the edge of the pane in focus, `alt-=` and `alt--` make the
text column wider or narrower (kept for the repository). The font size is the terminal's own: most
terminals change it with `ctrl-+` and `ctrl--`. In read mode: `w` switches the layout,
`t` switches the theme, `?` shows all keys.

## Architecturally significant requirements

| ID       | Pattern  | Requirement                                                                                                                                                                                              | Source           |
| -------- | -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| PANES-1  | Unwanted | If the open panes do not fit into the terminal, then the system shall hide panes from the edges by turns: first the rightmost pane right of the note, then the leftmost pane left of the note, and again. | User, 2026-11-11 |
| PANES-2  | Unwanted | If the system hides panes, then it shall keep the pane in focus, and shall hide the note last.                                                                                                          | User, 2026-11-11 |
| PANES-3  | Event    | When the user drags the mouse, the system shall select text only inside the pane where the drag started.                                                                                                | User, 2026-11-11 |

### Acceptance tests

**`panes_hide_from_the_right_then_the_left_by_turns`** (unit test; verifies PANES-1)

- Given the wide screen layout with the file tree, the note, the chat and the delta open.
- When the terminal gets narrower, step by step.
- Then the delta hides first, then the file tree, then the chat.

**`the_pane_in_focus_stays`** (unit test; verifies PANES-2)

- Given the same panes, with the focus on the delta.
- When the terminal is too narrow for all panes.
- Then the delta stays open, and another pane hides.

**`selection_stays_inside_its_panel`** (unit test; verifies PANES-3)

- Given the note and the chat side by side.
- When the user drags from the note into the chat.
- Then the selection holds only lines of the note.

## Detailed requirements

| ID       | Pattern    | Requirement                                                                                                                         | Source           |
| -------- | ---------- | ----------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| PANES-4  | Ubiquitous | The system shall have two layouts: wide screen and standard.                                                                         | User, 2026-11-11 |
| PANES-5  | Ubiquitous | The system shall use wide screen when the user did not choose a layout.                                                              | User, 2026-11-11 |
| PANES-6  | Event      | When the user switches the layout, the system shall keep the choice for the repository.                                              | Design           |
| PANES-7  | State      | While the layout is wide screen, the system shall show the open panes side by side in this order: file tree, note, chat, delta.      | User, 2026-11-11 |
| PANES-8  | State      | While the layout is standard and the user reads, the system shall show the delta in the place of the note.                           | Design           |
| PANES-9  | Ubiquitous | The system shall show the file tree on the left in both layouts.                                                                     | Design           |
| PANES-10 | Event      | When the user drags the separator between two panes, or presses `ctrl-←` or `ctrl-→`, the system shall change the width of the pane. | User, 2026-11-11 |
| PANES-11 | Ubiquitous | The system shall keep a minimum width for the note, and give the note the width that the other panes do not use.                    | Design           |
| PANES-12 | Event      | When the user changes the width of a pane, the system shall keep the width for the repository.                                       | Design           |
| PANES-13 | Event      | When the user releases the mouse button after a drag, the system shall copy the selected text to the clipboard, also over SSH.       | User, 2026-11-11 |
| PANES-14 | Event      | When the user turns the mouse wheel, the system shall scroll the pane under the pointer.                                             | Design           |
| PANES-20 | Event      | When scroll events come faster than one each 40 ms, as from a trackpad, the system shall scroll one line for each event and draw once for all the events that are waiting. | User, 2026-11-12 |
| PANES-15 | Event      | When the user presses `t`, the system shall show the next of the four themes, and keep the choice.                                   | User, 2026-11-11 |
| PANES-16 | Event      | When the user presses `?` or `f1`, the system shall show the keys of the pane in focus and the global keys.                         | Design           |
| PANES-17 | Ubiquitous | The system shall take the keys of every action of every pane from the default keymap file that it ships, then from the keymap file that the config names, then from `[keys.<pane>]` in the config. | User, 2026-11-11 |
| PANES-18 | Ubiquitous | The system shall have a key for every action, and no buttons.                                                                         | User, 2026-11-11 |
| PANES-19 | Unwanted   | If `[keys]` in the config names an unknown pane, action or key, then the system shall keep the default keys and tell the user.       | Design           |
| PANES-21 | Event      | When the user opens the delta and it does not fit beside the file tree, the system shall close the file tree. | User, 2026-11-12 |
| PANES-22 | Event      | When the user presses `alt-=` or `alt--`, the system shall make the text column 4 chars wider or narrower, and keep the width for the repository. | User, 2026-11-12 |
| PANES-23 | Event      | When the user runs `zendo keymap`, the system shall print the default keymap file, so that the user can start a keymap of their own from it. | User, 2026-11-12 |
| PANES-24 | Unwanted   | If one key has two actions in one pane of the keymap, then the system shall tell the user, and the key shall do the first action. | User, 2026-11-12 |
| PANES-25 | Unwanted   | If the terminal has no kitty keyboard protocol and sends `ctrl-shift` and a letter as `ctrl` and the letter, then the system shall do the action of `ctrl-shift` and the letter when `ctrl` and the letter has no action. | User, 2026-11-12 |

zendo takes the mouse. To use the selection of the terminal, the user holds a modifier key, usually
Shift, or Option in iTerm2.

Tests that verify detailed requirements: `the_delta_closes_the_tree_when_it_does_not_fit` (PANES-21),
`the_width_keys_change_the_text_column` (PANES-22), `the_default_keymap_file_names_every_action` (PANES-17),
`a_user_keymap_file_then_the_config_change_keys`, `zendo_keymap_prints_the_default_keymap` (PANES-17,
PANES-23), `a_key_of_two_actions_is_a_problem` (PANES-24), `without_the_kitty_protocol_ctrl_hjkl_select`
(PANES-25).
