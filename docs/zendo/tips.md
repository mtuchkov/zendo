---
status: "approved"
approved: "2026-11-18"
revision: "3"
date: "2026-11-17"
parent: "zendo.md"
prev: "usage.md"
level: "the tip menu at start and the tip in the status bar"
note: "Agent draft from the requests of the user on 2026-11-17, moved out of panes/key-change.md (REKEY-19 to REKEY-24). Revision 2: the option stops only the tip menu, the setting is in the config, the status bar shows a tip at random. Revision 3: every message comes before the tip. Approved by the user on 2026-11-18."
---

# Tips

zendo tells the user about features that are easy to miss. See [Lofi music](lofi.md),
[Change a key](panes/key-change.md) and [Keys, hints and help](panes/keymap.md).

**The tip menu.** At each start, zendo shows a float with tips; at the first start, after the user
answers the music question (LOFI-12). For now it has one tip: "If a key does not work for you,
double-click it to change it." Under the tips is the option `[ ] Do not show tips at start`. Only
the button `[OK]` closes the float: a click on it, or `enter`. The option stops only the tip menu,
not the tip in the status bar. zendo keeps it in `~/.zendo/config.toml`, not in `.zendo/db.json`.

**The tip in the status bar.** The status bar shows a tip to the right of the key hints while there
is no message and the width permits: a tip at random, chosen at each start.

## Architecturally significant requirements

| ID     | Pattern | Requirement | Source |
| ------ | ------- | ----------- | ------ |
| TIPS-1 | Event   | When zendo starts and the config does not have `tips = false`, the system shall show the tip menu; at the first start, after the user answers the music question. | User, 2026-11-17 |
| TIPS-6 | State   | While the status bar shows no message and has room after the key hints and its right side, the system shall show a tip of the tip menu to the right of the key hints; every message comes before the tip, and the tip gives way first. | User, 2026-11-17 |
| TIPS-7 | Ubiquitous | The system shall keep the setting `tips` in `~/.zendo/config.toml`, not in `.zendo/db.json`, and the setting shall stop only the tip menu at start, not the tip in the status bar. | User, 2026-11-17 |

### Acceptance tests

**`the_tip_menu_follows_the_music_question`** (unit test; verifies TIPS-1 to TIPS-4, TIPS-7)

- Given a config without the settings `music` and `tips`.
- When zendo starts, the user answers the music question, turns on `Do not show tips at start`
  and presses `esc`, then clicks `[OK]`.
- Then the tip menu shows after the music question, `esc` does not close it, `[OK]` does, the
  config gets `tips = false`, `.zendo/db.json` does not change, and the next start shows no float
  but the status bar still shows the tip.

**`the_status_bar_shows_a_tip_only_with_room_and_no_message`** (unit test; verifies TIPS-6)

- Given a status bar of 160 columns, then of 80 columns.
- When there is no message, then a message.
- Then only the wide status bar without a message shows the tip.

## Detailed requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| TIPS-2 | Ubiquitous | The system shall show in the tip menu the tip "If a key does not work for you, double-click it to change it." | User, 2026-11-17 |
| TIPS-3 | Event      | When the user clicks the button `[OK]` of the tip menu or presses `enter`, the system shall close the tip menu; other keys and clicks shall not close it. | User, 2026-11-17 |
| TIPS-4 | Event      | When the user closes the tip menu with the option `Do not show tips at start` on, the system shall save `tips = false` in the config. | User, 2026-11-17 |
| TIPS-5 | Event      | When the user clicks the option or presses `space` in the tip menu, the system shall turn the option on or off. | Design |
| TIPS-8 | Event      | When zendo starts, the system shall choose at random the tip that the status bar shows. | User, 2026-11-17 |
