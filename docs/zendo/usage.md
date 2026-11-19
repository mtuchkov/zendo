---
status: "draft"
approved: "2026-11-19"
revision: "6"
date: "2026-11-15"
parent: "zendo.md"
prev: "lofi.md"
next: "tips.md"
level: "what zendo counts about the actions of the user, where it keeps the counts, and what it never does with them"
note: "Agent draft of the decisions of the user on 2026-11-14, approved by the user. Revision 2: the test names and kinds as in the code; what the mouse counts. Revision 3: USAGE-6 lets the keymap wizard rank its suggestions by the counts (User, 2026-11-15). Revision 4: USAGE-14 names `stats = false`, which also turns off the times of the actions (User, 2026-11-19); the old `usage = false` still works. The user approved revision 4 on 2026-11-19. Revision 5: USAGE-15, the counts stay in memory and a thread of their own writes them (User, 2026-11-19); the wording of the agent waits for the review of the user. Revision 6, an agent draft of the decisions of the user of 2026-11-19 that waits for the review of the user: each action becomes an event of the telemetry log (telemetry.md), which takes the place of the usage file; the requirements of the file move there. Revision 5 is the approved and implemented revision."
---

# Usage counts

zendo counts how often the user does each action, and in which pane. Each action becomes an event of
the [telemetry log](telemetry.md), and the counts come from the events. The user reads them later to
decide which key hints zendo shows, and where. zendo itself does not read them: the hints do not
change while zendo runs. The one exception is the [keymap wizard](cli/keymap-wizard.md), which
ranks its key suggestions by them.

The panes are in [Panes and layouts](panes/panes.md), and the keymap and the key hints in
[Keys, hints and help](panes/keymap.md).

## What zendo counts

An action is an entry of the keymap: `copy`, `paste`, `cut`, `add_to_chat`, `say`, the change of a
pane or a session, and each other action that has a name there. zendo counts actions, not keys: a
count keeps its meaning when the user changes a key. Each count has three parts:

- **The pane**: the keymap pane of the binding that did the action, for example `read`, `chat`,
  `selection` or `global`, also when another pane has the focus.
- **The action**: its name in the keymap.
- **The trigger**: `key` or `mouse`. An action that the user does with the mouse and never with a
  key is a candidate for a hint. The mouse counts where it does what an action does: the copy of a
  selection, a selection for the chat, a click on a file of the tree, on the button of a session
  or on the play button of a shell block.

zendo counts an action where it runs, not where it finds the key. Typed text, keys that go to a
session or a terminal, and the keys of a box or a list are not actions and are not counted.

## Where the counts are

The event of an action holds its pane, its action and its trigger. The events are in the
[telemetry log](telemetry.md) in the data folder of zendo, which rolls at 50 files of 1 MB
(User, 2026-11-19). `stats = false` in `~/.zendo/config.toml` turns the log off.

## Architecturally significant requirements

| ID      | Pattern    | Requirement                                                                                                                                             | Source           |
| ------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| USAGE-1 | Event      | When an action of the keymap runs, the system shall record its keymap pane, its action name and its trigger in the event of the action (TELEM-1). | User, 2026-11-14; User, 2026-11-19 |
| USAGE-6 | Ubiquitous | The system shall not read the telemetry log to change what it shows or does, except to rank the suggestions of the keymap wizard (WIZARD-11). | User, 2026-11-14; User, 2026-11-15 |

### Acceptance tests

**`an_action_counts_under_its_pane_name_and_trigger`** (integration test in `tests/usage.rs`; verifies USAGE-1)

- Given a note in read mode with a selection.
- When the user does `add_to_chat` two times with its key and one time with the mouse, and presses
  a global key.
- Then the log holds three events of `read.add_to_chat`, two with the trigger `key` and one with
  `mouse`, and one event of the global action under `global` with `key`.

**`the_hints_do_not_depend_on_the_counts`** (integration test in `tests/usage.rs`; verifies USAGE-6)

- Given one empty data folder and one with a telemetry log that holds many events.
- When zendo starts with each of them on a terminal of 120×40.
- Then the status bar and the help show the same key hints.

## Detailed requirements

| ID       | Pattern    | Requirement                                                                                                                                  | Source           |
| -------- | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| USAGE-7  | Ubiquitous | The system shall count an action under the keymap pane of the binding that did it, not under the pane that has the focus.                    | Design           |
| USAGE-8  | Ubiquitous | The system shall count nothing that is not an action of the keymap: typed text, keys that go to a session or a terminal, keys of a box or a list. | User, 2026-11-14 |

Tests that verify detailed requirements: `a_count_has_the_pane_the_action_and_the_trigger`
(USAGE-7), `only_actions_of_the_keymap_count` and `typed_text_is_not_counted` in `tests/usage.rs`
(USAGE-8).

Retired in revision 6: USAGE-2, USAGE-3, USAGE-4, USAGE-5, USAGE-9, USAGE-10, USAGE-11, USAGE-12, USAGE-13, USAGE-14, USAGE-15.

The retired requirements were those of the usage file; the telemetry log takes its place (TELEM-2 to
TELEM-12). Retired identifiers are never reused.

## Open points

| Open question                                                                        | Affects  | Answered by |
| ------------------------------------------------------------------------------------ | -------- | ----------- |
| Does zendo delete the old folder `usage/` of the data folder?                        | TELEM-2  | User        |
