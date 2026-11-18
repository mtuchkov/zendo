---
status: "implemented"
approved: "2026-11-18"
revision: "6"
date: "2026-11-17"
parent: "panes.md"
prev: "text-input/text-input.md"
next: "key-hints.md"
level: "how the user changes one key from a hint or the help, and how a light model suggests free keys"
note: "Agent draft from the request of the user on 2026-11-17. Revision 2 takes the answers of the user on 2026-11-17: the API, only a double click, no key path in the help, `esc` is fixed. Revision 3 added the tip menu; revision 4 moves it to ../tips.md; revision 5 lets the float add a key; revision 6 moves the suggestions to key-hints.md, takes the answers of 2026-11-18: two clicks, remove a key, every key but `esc`, `haiku` with low effort, and retires the IDs that had two meanings in the drafts. Approved by the user on 2026-11-18. Implemented in zendo 0.93.0 to 0.95.0; REKEY-32 waits for a way to know that the terminal sends no mouse events. The frequent actions of REKEY-31 are the actions of the hints of the status bar until zendo ships the usage counts from real use (PANES-18)."
---

# Change a key

The user changes the key of one action in place, from the status bar or the help. See
[Keys, hints and help](keymap.md), [Keymap check and wizard](../cli/keymap-wizard.md),
[Key hints](key-hints.md), [ADR 0013](../../architecture/decisions/0013-key-hints-from-a-light-model.md) and [Tips](../tips.md).

**The float.** A click on a key in the status bar or in the help makes that key a button. A click
on the button opens a small float with the action, its pane and its keys; a double click does both.
When the focus moves, the button turns back into text. Every key of the keymap can change, except
`esc`: the escape actions keep it, and no action gets it.

The float has three choices: `[replace]` (the default), `[add]` and `[remove]`. It says that the
hints and the help show the first key of the action; an added key goes first. The float takes the
next key: `esc` closes it. A free key replaces the old key or joins the keys of the action; zendo
writes and applies the keymap at once and says what it changed. A key that clashes changes nothing,
and the float shows the clash and waits. A key clashes under the rules of WIZARD-10: another action
of the pane or of the global pane has it, or it has no modifier where the user types text.

**Suggestions.** On a clash, a light model suggests free keys: see [Key hints](key-hints.md).

The tip menu tells the user about the float: see [Tips](../tips.md).

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| REKEY-1 | Event      | When the user clicks a key in the status bar or in the help, the system shall show the key as a button; when the user clicks the button, the system shall open the key float for that action and that key. | User, 2026-11-18 |
| REKEY-2 | Event      | When the user presses a free key in the key float, the system shall replace the old key with it, write the change, apply the keymap at once, and say what it changed. | User, 2026-11-17 |
| REKEY-3 | Unwanted   | If the pressed key clashes, then the system shall show the clash in the float, keep the old key, and ask the light model for free keys. | User, 2026-11-17 |
| REKEY-28 | Ubiquitous | The system shall let the user change every key of the keymap except `esc`: it shall open no float for `esc`, and take `esc` as no new key. | User, 2026-11-18 |

### Acceptance tests

**`a_click_makes_a_button_and_a_second_click_opens_the_float`** (unit test; verifies REKEY-1, REKEY-30)

- Given the hint `alt-b tree` in the status bar.
- When the user clicks `alt-b`, moves the focus, clicks `alt-b` again, then clicks the button.
- Then the first button turns back into text, and the float shows `toggle_tree`, `global`,
  `alt-b` (marked) and `ctrl-shift-b`.

**`a_free_key_goes_to_the_keymap_file_and_the_hints`** (unit test; verifies REKEY-2)

- Given a config that names a keymap file with comments, and the float for `alt-b`.
- When the user presses `alt-t`, which is free.
- Then the file has `toggle_tree = ["alt-t", "ctrl-shift-b"]`, the comments stay, and the status
  bar shows `alt-t tree`.

**`a_clash_keeps_the_old_key`** (unit test; verifies REKEY-3) and
**`a_clash_asks_the_model_with_low_effort_and_the_keymap`** (mock API; verifies REKEY-3, REKEY-6,
in [Key hints](key-hints.md))

- Given the float for `toggle_tree` and a mock model API.
- When the user presses `tab`, the key of `next_pane`.
- Then the float shows the clash, the keymap file does not change, and the mock API gets one
  request for `haiku` with low effort and the keymap in effect.

**`esc_does_not_change`** (unit test; verifies REKEY-28)

- Given the hint `esc` of an escape action, and the float for the key of the search box.
- When the user clicks `esc` twice, then presses `esc` in the float.
- Then no float opens for `esc`, the float closes, and the keymap does not change.

## Detailed requirements

| ID       | Pattern    | Requirement | Source |
| -------- | ---------- | ----------- | ------ |
| REKEY-9  | Ubiquitous | The system shall change only the key that the user changed and keep the other keys of the action. | Design |
| REKEY-25 | Event      | When the user clicks `[add]` in the float and presses a free key, the system shall put the key first among the keys of the action and keep the other keys. | User, 2026-11-17 |
| REKEY-26 | Ubiquitous | The system shall say in the float that the hints and the help show the first key of the action. | User, 2026-11-17 |
| REKEY-29 | Event      | When the user clicks `[remove]` in the float, the system shall remove the key from the action, write the change and apply the keymap at once. | User, 2026-11-18 |
| REKEY-31 | Unwanted   | If the key is the last key of a frequently used action (PANES-18), then the system shall not remove it and shall say why. | PANES-18, Design |
| REKEY-30 | Event      | When the focus moves, the system shall show the key button as text again. | User, 2026-11-18 |
| REKEY-14 | Event      | When `esc` reaches the float, the system shall close it and change nothing. | User, 2026-11-17 |
| REKEY-15 | State      | While the float waits for a key and no key comes for a short time, the system shall say that the OS or the terminal can take the key. | Design |
| REKEY-16 | Ubiquitous | The system shall write the key to `[keys.<pane>]` of the config when that sets the action, else to the keymap file that the config names, else to a new `~/.zendo/keymap.toml` that it names in the config, and keep the comments and the other lines. | WIZARD-5, Design |
| REKEY-27 | Unwanted   | If the system cannot write the change, then the system shall keep the old key, keep the float open, and show the error. | Design |
| REKEY-32 | Unwanted   | If the terminal sends no mouse events, then the system shall name the keymap file and `zendo doctor --keys` in the help as the way to change keys. | Design |

Retired in revision 2: REKEY-8 (a key path in the help), because the user wants only a click.
Retired in revision 4: REKEY-19 to REKEY-24, moved to [Tips](../tips.md) as TIPS-1 to TIPS-6.
Retired in revision 6: REKEY-17 and REKEY-18, because each had two meanings in the drafts (a new
keymap file, the failed write, `esc`); they are REKEY-16, REKEY-27 and REKEY-28 now. REKEY-19 also
had two meanings (the message of a change, now in REKEY-2, and the tip menu).
Retired identifiers are never reused.
