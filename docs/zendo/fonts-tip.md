---
status: "draft"
revision: "3"
date: "2026-11-18"
parent: "zendo.md"
prev: "fonts.md"
next: "fonts-recipe.md"
level: "how the user finds the recommended fonts, and how the font list of the theme picker sets the font of the terminal"
note: "Agent draft of the decisions of the user on 2026-11-18, split from fonts.md revision 2. Approved by the user on 2026-11-18. Revision 2: implemented in zendo 0.96.0. Revision 3, a draft of the request of the user on 2026-11-18: a font list in the theme picker in place of the font button of the agent; the agent only in `zendo setup fonts` and `zendo doctor` (fonts-recipe.md). FONT-5, FONT-6, FONT-13 to FONT-16, FONT-20 and FONT-21 are retired. Not reviewed by the user."
---

# Font tip

Sources: the requests and answers of the user on 2026-11-18. The fonts and `zendo setup fonts` are
in [Fonts](fonts.md); the recipe that sets a font in the terminal is in
[Font recipe](fonts-recipe.md).

## What the user sees and does

**In the theme picker.** `t` in read mode opens the theme picker. Under the themes, it shows the
font list: `Font  [ JetBrains Mono ▾ ]`, with the font that zendo set last. `f` or a click opens
the list of the recommended fonts that are on the machine; `j` and `k` move, enter sets the font in
the terminal by the recipe, esc closes the list. No agent takes part.

**The tip.** While a recommended font is not on the machine, or the terminal has no working recipe,
the picker shows in place of the font list: run `zendo setup fonts`. The end of `zendo setup fonts`
tells how to open the theme picker and that its font list sets the font of the terminal. The key
comes from the keymap of the user.

**`zendo setup` and the installer.** The installer runs `zendo setup`. When the list of recommended
fonts changed since the last time that `zendo setup` named it, setup names `zendo setup fonts` and
the tip at its end. Otherwise it says nothing about fonts. The first run of setup names them.

## Architecturally significant requirements

| ID      | Pattern | Requirement | Source |
| ------- | ------- | ----------- | ------ |
| FONT-34 | State   | While the current terminal has a working recipe and a recommended font is on the machine, the theme picker shall show a font list with the font that the system set last. | User, 2026-11-18 |
| FONT-35 | Event   | When the user picks a font in the font list, the system shall set it in the terminal by the recipe, without the agent. | User, 2026-11-18 |

### Acceptance tests

**`the font list sets the font by the recipe`** (unit test; verifies FONT-34, FONT-35, FONT-36)

- Given a test home with a recipe that sets the line of a config file, and two recommended fonts on
  the machine.
- When the user opens the theme picker, presses `f`, `j` and enter.
- Then the line of the file names the second font, the picker shows it in the font list, and no
  request goes to the agent.

**`the picker shows the tip without a recipe`** (unit test; verifies FONT-2, FONT-37)

- Given all recommended fonts on the machine and no recipe, and zendo offline.
- When the user opens the theme picker.
- Then zendo shows the themes and the tip with `zendo setup fonts`, and no font list.

**`setup names the fonts when the list changed`** (unit test; verifies FONT-18, FONT-19, FONT-39)

- Given a config folder in the test folder, without a list of fonts that setup named.
- When `zendo setup` runs two times, and then once with one more recommended font.
- Then the first and the third output name `zendo setup fonts` and the key of the theme picker; the
  second output says nothing about fonts.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| FONT-36 | Event      | When the user presses `f` or clicks the font list, the system shall open the list of the recommended fonts that are on the machine. | User, 2026-11-18 |
| FONT-37 | State      | While a recommended font is not on the machine, or the current terminal has no working recipe, the theme picker shall show the tip with `zendo setup fonts` in place of the font list. | User, 2026-11-18 |
| FONT-38 | Unwanted   | If the recipe fails to set the font, then the system shall show the failure and `zendo doctor --fix --agent` in the flash line. | User, 2026-11-18 |
| FONT-18 | Event      | When `zendo setup` ends and the list of recommended fonts differs from the list that `zendo setup` named the last time, the system shall name `zendo setup fonts` and the tip of FONT-39, and keep the list. | User, 2026-11-18 |
| FONT-19 | Unwanted   | If the list of recommended fonts is the same as the list that `zendo setup` named the last time, then `zendo setup` shall say nothing about fonts. | User, 2026-11-18 |
| FONT-39 | Ubiquitous | The tip shall tell the key that opens the theme picker, from the keymap of the user, and that its font list sets the font of the terminal. | User, 2026-11-18 |

Tests that verify detailed requirements: `a_failed_recipe_names_doctor_in_the_flash_line`
(FONT-38), `the_tip_names_the_key_of_the_theme_picker` (FONT-39).
