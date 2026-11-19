---
status: "implemented"
approved: "2026-11-18"
revision: "1"
date: "2026-11-18"
parent: "zendo.md"
prev: "fonts-recipe.md"
level: "the format of the recipe of the terminal font: the ways to set a font, the size, and how zendo finds the current terminal"
note: "Agent draft of the decisions of the user on 2026-11-18, split from fonts-recipe.md: a whole file for COSMIC, the size for GNOME, the terminal from more than TERM_PROGRAM; no special case for tmux. Approved by the user on 2026-11-18. Implemented in zendo 0.116.0."
---

# Font recipe format

Sources: the requests of the user on 2026-11-18. How Claude writes the recipe and how
`zendo doctor` checks it is in [Font recipe](fonts-recipe.md).

## What the recipe holds

The file `~/.zendo/terminal-fonts.toml` holds one table for each terminal, named by the terminal.
A recipe sets a font in one or more of these ways, in this order:

- **A file.** zendo replaces the first line of the file that starts with `prefix`, or adds the line
  at the end. With `whole = true`, the line is the whole content of the file.
- **A command.** zendo runs a program with its arguments.
- **An escape sequence.** zendo writes it to the terminal.

Then zendo runs the `reload` command, if the recipe has one. In each text of the recipe, `{font}`
stands for the family name of the font and `{size}` for the current size of the font. The `size`
command of the recipe prints the current setting; its last number is the size.

Examples, as Claude can write them:

```toml
[ghostty]
file = "~/.config/ghostty/config"
prefix = "font-family"
line = 'font-family = "{font}"'

[cosmic-term]
file = "~/.config/cosmic/com.system76.CosmicTerm/v1/font_name"
whole = true
line = '"{font}"'

[gnome-terminal]
size = ["gsettings", "get", "org.gnome.Terminal.Legacy.Profile:/org/gnome/terminal/legacy/profiles:/:<uuid>/", "font"]
command = ["gsettings", "set", "org.gnome.Terminal.Legacy.Profile:/org/gnome/terminal/legacy/profiles:/:<uuid>/", "font", "{font} {size}"]

[alacritty]
command = ["alacritty", "msg", "config", 'font.normal.family="{font}"']
```

| Way | Terminals, for example |
| --- | ---------------------- |
| A file | Ghostty, kitty, COSMIC Terminal |
| A command | Alacritty, GNOME Terminal, Ptyxis |
| An escape sequence | xterm, WezTerm and iTerm2 after a preparation that Claude makes once |

**The current terminal.** zendo takes the name from `TERM_PROGRAM`. Where it is not set, zendo
takes it from the variables that terminals set, such as `GHOSTTY_RESOURCES_DIR`, `KITTY_WINDOW_ID`,
`ALACRITTY_WINDOW_ID`, `WEZTERM_PANE`, `GNOME_TERMINAL_SERVICE` and `VTE_VERSION`, and else from
`TERM`. Inside tmux, `TERM_PROGRAM` is `tmux`, and tmux is a terminal like the others. The first
message to Claude names the terminal by this name.

## Architecturally significant requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| FONT-29 | Ubiquitous | A recipe shall set a font by a file, a command, or an escape sequence, in this order, and may have a command that loads the config again. | User, 2026-11-18 |
| FONT-30 | Ubiquitous | The system shall run the commands of a recipe without a shell. | Design |

### Acceptance tests

**`a recipe sets the font in three ways`** (unit test; verifies FONT-23, FONT-29, FONT-30,
FONT-40, FONT-41)

- Given a recipe with a line of a config file, a whole file, a command with `{size}`, an escape
  sequence and a reload command, and a `size` command that prints `'Fira Code 13'`.
- When the system sets the font `Iosevka Term`.
- Then the line of the file names the font and the other lines stay, the whole file is `"Iosevka
  Term"`, the command got `Iosevka Term 13` as one argument, the reload command ran once, and the
  escape sequence names the font.

**`the current terminal comes from the environment`** (unit test; verifies FONT-32, FONT-42)

- Given environments with `TERM_PROGRAM=ghostty`, with only `GNOME_TERMINAL_SERVICE`, with only
  `VTE_VERSION`, and with only `TERM=xterm-256color`.
- When the system names the current terminal.
- Then it names `ghostty`, `gnome-terminal`, a VTE terminal, and `xterm-256color`.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| FONT-32 | Ubiquitous | The system shall keep one recipe for each terminal, by the name of the terminal. | Design |
| FONT-40 | Where      | Where a recipe has `whole = true`, the system shall write its line as the whole content of the file. | User, 2026-11-18 |
| FONT-41 | Where      | Where a text of a recipe has `{size}`, the system shall replace it with the last number that the `size` command of the recipe prints. | User, 2026-11-18 |
| FONT-42 | Ubiquitous | The system shall name the current terminal from `TERM_PROGRAM`, else from the variables that terminals set, else from `TERM`. | User, 2026-11-18 |
| FONT-43 | Unwanted   | If a text of a recipe has `{size}` and the `size` command prints no number, then the system shall not set the font and shall name the failure. | Design |

Tests that verify detailed requirements: `a_missing_line_goes_to_the_end_of_the_file` (FONT-40),
`a_size_without_a_number_sets_nothing` (FONT-43), `the_check_names_a_missing_file_and_command`
(FONT-31), `the_font_that_was_set_stays_in_the_recipe` (FONT-32, FONT-34).
