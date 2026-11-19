---
status: "draft"
revision: "1"
date: "2026-11-18"
parent: "zendo.md"
prev: "fonts-tip.md"
level: "how zendo learns to set the font of the terminal: the recipe that Claude writes, and how zendo doctor checks and repairs it"
note: "Agent draft of the decisions of the user on 2026-11-18: a recipe from Claude, made in an interactive Claude session by `zendo setup fonts`, checked by `zendo doctor`, repaired by `zendo doctor --fix --claude`. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document."
---

# Font recipe

Sources: the requests of the user on 2026-11-18; `zendo doctor --fix --claude` in
[Doctor](cli/doctor.md). The fonts are in [Fonts](fonts.md); the font list of the theme picker,
which uses the recipe, is in [Font tip](fonts-tip.md).

## What the user sees and does

zendo cannot set its own font: the terminal draws every character with the font of its config.
Each terminal sets the font in another way. So zendo keeps a recipe for each terminal: how to set a
font in it. Claude writes the recipe once, with the user; after that, the font list of the theme
picker sets fonts by the recipe, without the agent.

**In `zendo setup fonts`.** After the install, while the current terminal has no working recipe,
zendo asks whether it may start Claude for the recipe. On yes, it starts an interactive Claude
session in the same terminal. The first message tells Claude the terminal, the fonts, the file of
the recipe and its format. Claude finds the config of the terminal, asks the user before each
change, writes the recipe, and the user ends the session. Then zendo checks the recipe and names the
result.

**In `zendo doctor`.** Doctor checks the recipe of the current terminal. When the check fails,
`zendo doctor --fix --claude` starts the same Claude session to repair the recipe.

**The recipe.** The file `~/.zendo/terminal-fonts.toml` keeps one recipe for each terminal. A
recipe sets a font in one of three ways, or more of them in turn: it replaces or adds a line of a
config file, it runs a command, or it writes an escape sequence to the terminal. It can also have a
command that makes the terminal load its config again. `{font}` in the recipe stands for the family
name of the font.

## Main design choices

- Design: Claude knows how each terminal sets its font, zendo knows only the three ways. A new
  terminal needs no new code. The options are in
  [ADR 0013](../architecture/decisions/0013-terminal-font-by-a-recipe.md).
- Design: the session with Claude is interactive. Claude asks the user before it changes a file of
  the terminal, so zendo needs no questions of its own about each change.
- Design: zendo runs the commands of the recipe without a shell, so a font name cannot become a
  command.

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| FONT-22 | Event      | When `zendo setup fonts` has installed the fonts in a terminal that has no working recipe, the system shall ask whether it may start Claude, and on yes start an interactive Claude session in the same terminal to write the recipe. | User, 2026-11-18 |
| FONT-23 | Ubiquitous | The system shall set a font of the terminal only by the recipe of that terminal; it shall not change the config of the terminal in another way. | User, 2026-11-18 |
| FONT-24 | Event      | When the user runs `zendo doctor`, the system shall check the recipe of the current terminal. | User, 2026-11-18 |
| FONT-25 | Event      | When the check of the recipe fails and the user runs `zendo doctor --fix --claude`, the system shall start the interactive Claude session of FONT-22 to repair the recipe. | User, 2026-11-18 |

### Acceptance tests

**`setup fonts starts claude for the recipe after yes`** (mock `claude`; verifies FONT-22, FONT-27,
FONT-28)

- Given a test home, a terminal without a recipe, and a mock `claude` that writes a recipe.
- When `zendo setup fonts` runs with the answer `n`, and then with the answer `y`.
- Then the first run starts no `claude`; the second starts it once, with the recipe file and the
  format in its first message, and then names the recipe as working.

**`doctor checks the recipe and repairs it with claude`** (mock `claude`; verifies FONT-24,
FONT-25, FONT-31)

- Given a recipe whose config file is gone.
- When `zendo doctor` runs, and then `zendo doctor --fix --claude`.
- Then `zendo doctor` names the failure and `zendo doctor --fix --claude`, starts no `claude`, and
  changes no file; `--fix --claude` starts the mock `claude` once.

**`a recipe sets the font in three ways`** (unit test; verifies FONT-23, FONT-29, FONT-30)

- Given a recipe with a line of a config file, a command, an escape sequence and a reload command.
- When the system sets the font `Iosevka Term`.
- Then the line of the file has the font and the other lines stay, the command and the reload
  command ran once without a shell, and the escape sequence has the font.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| FONT-26 | Unwanted   | If the input of `zendo setup fonts` is not a terminal, then the system shall not ask and not start Claude; it shall name `zendo setup fonts` to run in the terminal. | Design |
| FONT-27 | Ubiquitous | The first message to Claude shall name the terminal, the fonts and their family names, the recipe file and its format, and tell Claude to ask the user before each change. | Design |
| FONT-28 | Event      | When the Claude session ends, the system shall check the recipe and say whether it works. | Design |
| FONT-29 | Ubiquitous | A recipe shall set a font by the line of a config file, a command, or an escape sequence, and may have a command that loads the config again. | User, 2026-11-18 |
| FONT-30 | Ubiquitous | The system shall run the commands of a recipe without a shell. | Design |
| FONT-31 | Unwanted   | If the recipe of the current terminal has a missing file or a command that is not on the `PATH`, then `zendo doctor` shall name the failure and `zendo doctor --fix --claude`. | Design |
| FONT-32 | Ubiquitous | The system shall keep one recipe for each terminal and find the current terminal from the environment of the process. | Design |
| FONT-33 | Unwanted   | If `claude` is not on the `PATH`, then the system shall not start the session and shall say how to install Claude Code. | Design |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| Inside tmux, the escape sequences need the passthrough of tmux. Is a recipe for tmux enough? | FONT-29, FONT-32 | User |
| Where the agent is Codex, start an interactive `codex` session in the same way? | FONT-22 | User |
