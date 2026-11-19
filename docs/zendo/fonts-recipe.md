---
status: "implemented"
approved: "2026-11-18"
revision: "1"
date: "2026-11-18"
parent: "zendo.md"
prev: "fonts-tip.md"
next: "fonts-recipe-format.md"
level: "how zendo learns to set the font of the terminal: the recipe that the agent writes, and how zendo doctor checks and repairs it"
note: "Agent draft of the decisions of the user on 2026-11-18: a recipe from Claude, made in an interactive Claude session by `zendo setup fonts`, checked by `zendo doctor`, repaired by `zendo doctor --fix --agent`; Codex in the same way as Claude. Approved by the user on 2026-11-18. Implemented in zendo 0.113.0."
---

# Font recipe

Sources: the requests of the user on 2026-11-18; `zendo doctor --fix --agent` in
[Doctor](cli/doctor.md). The fonts are in [Fonts](fonts.md); the font list of the theme picker,
which uses the recipe, is in [Font tip](fonts-tip.md).

## What the user sees and does

zendo cannot set its own font: the terminal draws every character with the font of its config.
Each terminal sets the font in another way. So zendo keeps a recipe for each terminal: how to set a
font in it. The agent of the user, Claude or Codex, writes the recipe once, with the user; after that, the font list of the theme
picker sets fonts by the recipe, without the agent.

**In `zendo setup fonts`.** After the install, while the current terminal has no working recipe,
zendo asks whether it may start the agent for the recipe. On yes, it starts an interactive session
of the agent in the same terminal: `claude` for Claude, `codex` for Codex. The first message tells
the agent the terminal, the fonts, the file of
the recipe and its format. The agent finds the config of the terminal, asks the user before each
change, writes the recipe, and the user ends the session. Then zendo checks the recipe and names the
result.

**In `zendo doctor`.** Doctor checks the recipe of the current terminal. When the check fails,
`zendo doctor --fix --agent` starts the same session to repair the recipe.

**The recipe.** The file `~/.zendo/terminal-fonts.toml` keeps one recipe for each terminal. A
recipe sets a font by a file, a command, or an escape sequence. Its format, the examples for Ghostty,
COSMIC, GNOME and Alacritty, and how zendo finds the current terminal are in
[Font recipe format](fonts-recipe-format.md).

## Main design choices

- Design: the agent knows how each terminal sets its font, zendo knows only the three ways. A new
  terminal needs no new code. The options are in
  [ADR 0013](../architecture/decisions/0013-terminal-font-by-a-recipe.md).
- Design: the session with the agent is interactive. The agent asks the user before it changes a file of
  the terminal, so zendo needs no questions of its own about each change.

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| FONT-22 | Event      | When `zendo setup fonts` has installed the fonts in a terminal that has no working recipe, the system shall ask whether it may start the agent of the user, and on yes start an interactive session of the agent in the same terminal to write the recipe. | User, 2026-11-18 |
| FONT-23 | Ubiquitous | The system shall set a font of the terminal only by the recipe of that terminal; it shall not change the config of the terminal in another way. | User, 2026-11-18 |
| FONT-24 | Event      | When the user runs `zendo doctor`, the system shall check the recipe of the current terminal. | User, 2026-11-18 |
| FONT-25 | Event      | When the check of the recipe fails and the user runs `zendo doctor --fix --agent`, the system shall start the interactive Claude session of FONT-22 to repair the recipe. | User, 2026-11-18 |

### Acceptance tests

**`setup fonts starts the agent for the recipe after yes`** (mock `claude`; verifies FONT-22, FONT-27,
FONT-28)

- Given a test home, a terminal without a recipe, and a mock `claude` that writes a recipe.
- When `zendo setup fonts` runs with the answer `n`, and then with the answer `y`.
- Then the first run starts no `claude`; the second starts it once, with the recipe file and the
  format in its first message, and then names the recipe as working.

**`doctor checks the recipe and repairs it with claude`** (mock `claude`; verifies FONT-23, FONT-24,
FONT-25, FONT-31)

- Given a recipe whose config file is gone.
- When `zendo doctor` runs, and then `zendo doctor --fix --agent`.
- Then `zendo doctor` names the failure and `zendo doctor --fix --agent`, starts no `claude`, and
  changes no file; `--fix --agent` starts the mock `claude` once.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| FONT-26 | Unwanted   | If the input of `zendo setup fonts` is not a terminal, then the system shall not ask and not start the agent; it shall name `zendo setup fonts` to run in the terminal. | Design |
| FONT-27 | Ubiquitous | The first message to the agent shall name the terminal, the fonts and their family names, the recipe file and its format, and tell Claude to ask the user before each change. | Design |
| FONT-44 | Where      | Where the agent of the user is Codex, the system shall start `codex` for the session; else it shall start `claude`. | User, 2026-11-18 |
| FONT-28 | Event      | When the session of the agent ends, the system shall check the recipe and say whether it works. | Design |
| FONT-31 | Unwanted   | If the recipe of the current terminal has a missing file or a command that is not on the `PATH`, then `zendo doctor` shall name the failure and `zendo doctor --fix --agent`. | Design |
| FONT-33 | Unwanted   | If the command of the agent, `claude` or `codex`, is not on the `PATH`, then the system shall not start the session and shall say how to install it. | Design |

Tests that verify detailed requirements: `a_missing_agent_names_how_to_install_it` (FONT-33),
`the_session_starts_the_command_of_the_agent` (FONT-44).
