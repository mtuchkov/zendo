---
status: "implemented"
approved: "2026-11-14"
revision: "3"
date: "2026-11-15"
parent: "zendo.md"
prev: "lofi.md"
next: "fonts.md"
level: "what zendo counts about the actions of the user, where it keeps the counts, and what it never does with them"
note: "Agent draft of the decisions of the user on 2026-11-14, approved by the user. Revision 2: the test names and kinds as in the code; what the mouse counts. Revision 3: USAGE-6 lets the keymap wizard rank its suggestions by the counts (User, 2026-11-15)."
---

# Usage counts

zendo counts how often the user does each action, and in which pane. The counts stay on the disk of
the user. The user reads them later to decide which key hints zendo shows, and where. zendo itself
does not read them: the hints do not change while zendo runs. The one exception is the
[keymap wizard](cli/keymap-wizard.md), which ranks its key suggestions by them.

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

Each zendo process writes one file, `usage/<date of the start>-<process id>.toml`, in the data
folder of zendo (`~/.local/share/zendo`, or `$XDG_DATA_HOME/zendo`). Two processes never write the
same file, and one file for each run shows how the use changes from day to day. The head of the file
says which version of zendo the action names are from:

```toml
zendo = "0.58.0"
os = "macos"
started = "2026-11-14T09:12:40Z"
written = "2026-11-14T11:03:10Z"

[read]
add_to_chat = { key = 12, mouse = 3 }
```

zendo writes the file every 15 seconds while the counts changed, and when it exits. A crash loses at
most the last 15 seconds. `usage = false` in `~/.zendo/config.toml` turns the counting off.

## Architecturally significant requirements

| ID      | Pattern    | Requirement                                                                                                                                             | Source           |
| ------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| USAGE-1 | Event      | When an action of the keymap runs, the system shall add one to the count of its keymap pane, its action name and its trigger.                           | User, 2026-11-14 |
| USAGE-2 | Ubiquitous | The system shall keep the counts only in files in the data folder of zendo on the local disk, and shall send them nowhere.                               | User, 2026-11-14 |
| USAGE-3 | State      | While zendo runs and the counts changed since the last write, the system shall write the counts to the usage file every 15 seconds.                     | User, 2026-11-14 |
| USAGE-4 | Ubiquitous | The system shall write the counts of each zendo process to a file of that process only.                                                                 | Design           |
| USAGE-5 | Unwanted   | If the write of the usage file fails, then the system shall continue with no message and no delay of the screen.                                        | Design           |
| USAGE-6 | Ubiquitous | The system shall not read the counts to change what it shows or does, except to rank the suggestions of the keymap wizard (WIZARD-11). | User, 2026-11-14; User, 2026-11-15 |

### Acceptance tests

**`an_action_counts_under_its_pane_name_and_trigger`** (integration test in `tests/usage.rs`; verifies USAGE-1)

- Given a note in read mode with a selection.
- When the user does `add_to_chat` two times with its key and one time with the mouse, and presses
  a global key.
- Then the counts hold `read.add_to_chat` with `key = 2` and `mouse = 1`, and the global action
  under `global` with `key = 1`.

**`the_usage_file_is_local_and_of_one_process`** (unit test; verifies USAGE-2, USAGE-4)

- Given an empty data folder and two sets of counts with different process ids.
- When each set is written.
- Then `usage/` in the data folder holds two files, each with the counts of one process, and no
  file changes outside the data folder.

**`the_counts_reach_the_disk_within_15_seconds`** (unit test with a test clock; verifies USAGE-3)

- Given an empty data folder.
- When the user does one action and the clock moves 15 seconds, two times; then the clock moves
  on with no action.
- Then the usage file holds the count after each of the first two steps, and the third writes no
  file.

**`a_failed_write_changes_nothing_on_the_screen`** (integration test in `tests/usage.rs`; verifies USAGE-5)

- Given a data folder that zendo cannot write.
- When the user does an action and the clock moves 15 seconds.
- Then the screen shows no message, and the next action works.

**`the_hints_do_not_depend_on_the_counts`** (integration test in `tests/usage.rs`; verifies USAGE-6)

- Given one empty data folder and one with usage files that hold large counts.
- When zendo starts with each of them on a terminal of 120×40.
- Then the status bar and the help show the same key hints.

## Detailed requirements

| ID       | Pattern    | Requirement                                                                                                                                  | Source           |
| -------- | ---------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| USAGE-7  | Ubiquitous | The system shall count an action under the keymap pane of the binding that did it, not under the pane that has the focus.                    | Design           |
| USAGE-8  | Ubiquitous | The system shall count nothing that is not an action of the keymap: typed text, keys that go to a session or a terminal, keys of a box or a list. | User, 2026-11-14 |
| USAGE-9  | Ubiquitous | The system shall write to the usage file no text of the user, no file name, no path and no time of a single action.                          | Design           |
| USAGE-10 | Ubiquitous | The system shall name the usage file `usage/<date of the start>-<process id>.toml` in the data folder of zendo.                              | Design           |
| USAGE-11 | Ubiquitous | The system shall write in the usage file the version of zendo, the system, the time of the start and the time of the last write.             | Design           |
| USAGE-12 | Event      | When zendo exits, the system shall write the counts.                                                                                         | Design           |
| USAGE-13 | Ubiquitous | The system shall replace the usage file whole, through a temporary file and a rename.                                                        | Design           |
| USAGE-14 | Optional   | Where `usage = false` is in the config, the system shall count nothing and write no usage file.                                              | Design           |

Tests that verify detailed requirements: `a_count_has_the_pane_the_action_and_the_trigger`
(USAGE-7), `only_actions_of_the_keymap_count` and `typed_text_is_not_counted` in `tests/usage.rs`
(USAGE-8), `the_file_has_a_head_and_only_counts` (USAGE-9, USAGE-11, USAGE-13),
`the_exit_writes_the_counts` (USAGE-12), `off_counts_nothing_and_writes_nothing` and
`the_config_turns_the_counting_off` (USAGE-14). USAGE-10 is in
`the_usage_file_is_local_and_of_one_process`.

## Open points

| Open question                                                                        | Affects  | Answered by |
| ------------------------------------------------------------------------------------ | -------- | ----------- |
| Does zendo delete old usage files, and after how long?                               | USAGE-10 | User        |
