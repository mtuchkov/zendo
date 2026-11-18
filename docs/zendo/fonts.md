---
status: "implemented"
approved: "2026-11-18"
revision: "3"
date: "2026-11-18"
parent: "zendo.md"
prev: "tips.md"
next: "fonts-tip.md"
level: "which fonts zendo recommends, and what `zendo setup fonts` does on the machine"
note: "Agent draft of the decisions of the user on 2026-11-17 and 2026-11-18. Revision 2: the answers of the user on 2026-11-18 (Iosevka and Iosevka Term, Fira Code without italic); the theme picker tip moved to fonts-tip.md. Approved by the user on 2026-11-18. Revision 3: implemented in zendo 0.96.0; the test names as in the code."
---

# Fonts

Sources: the requests and answers of the user on 2026-11-17 and 2026-11-18.

## What the user sees and does

The terminal draws the font, not zendo. zendo recommends seven monospace fonts and helps the user
get them: JetBrains Mono, Atkinson Hyperlegible Mono, IBM Plex Mono, Iosevka, Iosevka Term, Fira
Code, and Cascadia Code. The fonts are optional.

`zendo setup fonts` downloads each font from the release of its project, at a version and a
checksum that zendo pins. It installs Regular, Bold, Italic and Bold Italic of each font, with its
license, in the font folder of the user. Fira Code has no italic, so it gets Regular and Bold. The
command needs no admin rights and asks nothing, so an agent can run it. A second run changes
nothing. At its end, it names the installed fonts and tells how to open the theme picker and change
the font there.

How the user finds the command, and how the agent sets the font of the terminal, is in
[Font tip](fonts-tip.md).

## Main design choices

- Design: zendo pulls the fonts; it does not carry them. The binary and the history of git keep no
  font files. The options are in
  [ADR 0012](../architecture/decisions/0012-fonts-from-their-releases.md).
- Design: zendo installs the files, and the agent changes the terminal. Each terminal keeps its
  font in another place and format; the agent can read the config and ask the user.
- All seven fonts use the SIL Open Font License 1.1. zendo installs the files without change, so no
  reserved font name needs a new name.

## Architecturally significant requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| FONT-1 | Ubiquitous | The system shall not carry font files in its binary or in its repository; `zendo setup fonts` shall download them from the releases of their projects, at pinned versions. | User, 2026-11-18 |
| FONT-2 | Ubiquitous | The system shall start and work without the recommended fonts and without the network. | Design |
| FONT-3 | Unwanted   | If a downloaded file does not have its pinned checksum, then the system shall not install any file of that font. | Design |
| FONT-4 | Ubiquitous | `zendo setup fonts` shall write only in the font folder of the user and need no admin rights. | Design |

### Acceptance tests

**`setup fonts installs the pinned styles`** (unit test; verifies FONT-1, FONT-4, FONT-7, FONT-8,
FONT-9, FONT-11, FONT-20)

- Given an empty font folder in the test folder and local copies of the release archives.
- When `zendo setup fonts` runs two times.
- Then the folder has the four styles and the license of each font, and Regular and Bold of Fira
  Code; the second run changes no file; the output names the key that opens the theme picker.

**`a bad checksum installs nothing of the font`** (unit test; verifies FONT-3, FONT-10)

- Given local release archives, one with a changed file.
- When `zendo setup fonts` runs.
- Then no file of that font is installed, the other fonts are, and the exit status is not zero.

**`zendo works without the fonts`** (unit test; verifies FONT-2)

- Given a machine without the recommended fonts, and zendo offline.
- When zendo starts and the user opens the theme picker.
- Then zendo shows the note and the themes as before.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| FONT-7  | Event      | When `zendo setup fonts` runs, the system shall install Regular, Bold, Italic and Bold Italic of JetBrains Mono, Atkinson Hyperlegible Mono, IBM Plex Mono, Iosevka, Iosevka Term, Fira Code, and Cascadia Code. | User, 2026-11-17; User, 2026-11-18 |
| FONT-8  | Unwanted   | If a font has no italic style, as Fira Code, then the system shall install only its Regular and Bold. | User, 2026-11-18 |
| FONT-9  | Unwanted   | If a font is already in the font folder of the user, then the system shall not install it again and shall say so. | Design |
| FONT-10 | Unwanted   | If the download of a font fails, then the system shall name the font and the failure, install the other fonts, and end with an exit status that is not zero. | Design |
| FONT-11 | Ubiquitous | The system shall install the license of each font beside its files. | Design |
| FONT-12 | Event      | When the system installs a font on Linux, the system shall refresh the font cache of the user. | Design |
| FONT-17 | Ubiquitous | `zendo setup fonts` shall not wait for input. | Design |

Tests that verify detailed requirements: `a_failed_download_names_the_failure_and_installs_the_others`
(FONT-10), `a_font_on_the_machine_is_not_installed_again` (FONT-9),
`the_catalog_has_the_seven_fonts_and_their_styles` (FONT-7, FONT-8, FONT-11),
`a_zip_gives_a_stored_and_a_deflated_member` (FONT-1).
