---
status: "approved"
approved: "2026-11-18"
date: "2026-11-18"
sources:
  - "../../zendo/fonts.md"
  - "The requests of the user on 2026-11-17 and 2026-11-18: ship six recommended fonts (seven with Iosevka Term); a tip in the theme picker to run `zendo setup fonts`; ask the user before the agent installs them; four styles; Iosevka and Iosevka Term; `zendo setup` names the fonts only when the list changed"
---

# 0012 Download the recommended fonts from their releases

## Context

zendo recommends seven monospace fonts (FONT-7). zendo is a terminal program, so it cannot draw a
font itself: the fonts must be in the font folder of the user, and the terminal must use one of
them. The installer builds zendo from the repository with `cargo install`, so each file in the
repository goes to each machine and stays in the history of git. zendo must start and work without
the network (FONT-2).

All seven fonts use the SIL Open Font License 1.1. A change of a file, such as a subset, needs a new
font name where the font has a reserved font name.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| Carry the files in the binary (`include_bytes!`) | Works without the network. One version for each release of zendo. | The files go in the repository and stay in the history of git. Iosevka alone is megabytes for each style. A larger binary for every user, also those who do not want the fonts. |
| Download the releases of the font projects, at pinned versions with checksums | The binary and the repository stay small. Only the users who ask get the fonts. The files come from their authors, without change. | Needs the network at install time. The pins need an update when a font gets a new release. |
| Carry the small fonts, download Iosevka | Most fonts work without the network. | Two code paths, and still font files in the history of git. |
| Only a list of the fonts in the documentation | No code. | The user finds, downloads and installs six fonts by hand. |

How the terminal gets the font:

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| zendo writes the config of known terminals | No agent needed. | Each terminal has another format and place; some keep it in a binary profile. zendo changes a file of the user that it does not own. |
| The agent sets it, after the user agrees | The agent reads the actual config and asks the user (FONT-6). No terminal code in zendo. | Needs a connected agent. Without one, the user sets the font by hand. |

## Decision

`zendo setup fonts` downloads each font from the release of its project, at a version and a
checksum that zendo pins, and installs the four styles with the license in the font folder of the
user (FONT-1, FONT-3, FONT-4). zendo does not change the config of the terminal. The theme picker
shows the command, and, after the user agrees, asks the agent to run it and to set the font of the
terminal with the user (FONT-5, FONT-6, FONT-15).

## Consequences

- The release archives of some fonts hold many styles; the command takes only the four that it
  needs.
- A new release of a font changes nothing for the users until zendo updates its pin.
- Tests use local copies of the archives, so `cargo test` runs without the network.

Notes of the first implementation (zendo 0.96.0):

- The release archives of Iosevka and Cascadia Code are 150 to 180 MB. The command reads the
  central directory of each archive and then only the members that it needs, with HTTP ranges, so
  it does not download the whole archives.
- The downloads go through `curl`: it follows the redirects of GitHub and uses the proxy and the
  certificates of the system, as the installer does.
- Atkinson Hyperlegible Mono has no releases. Its files come from a pinned commit of its project.
- The files go flat into the font folder, with the license as `<Font>-LICENSE.*`: macOS and
  fontconfig both read such files.
