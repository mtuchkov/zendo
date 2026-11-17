---
status: "approved"
approved: "2026-11-14"
revision: "2"
date: "2026-11-14"
parent: "zendo.md"
prev: "site.md"
next: "usage.md"
level: "what the user hears when zendo plays lofi music, and the decisions of the user"
note: "From the requests of the user on 2026-11-13 and the answers of 2026-11-14: the music button, on by default, the question at the first start, the chime in the music, both platforms, the pause without focus. LOFI-9 is retired. Approved by the user on 2026-11-14. The notes of the first implementation are in ADR 0006."
---

# Lofi music

Sources: the requests of the user on 2026-11-13 for the [landing page](site.md), the answers of the
user on 2026-11-14, and `site/index.html` in zendo 0.49.5.

## What the user hears

zendo plays calm lofi music while the user reads and writes. The music is an old electric piano on
four jazz chords, a soft bass, soft drums with swing, and the crackle of a vinyl record. It plays
without breaks until the user stops it. zendo makes the music itself, so no one owns it and it needs
no network.

The music is on by default. At the first start of zendo, the music plays and a float asks the user
whether it shall play at each start. The button `[music]` in the top line, and the key `alt-m`, stop
and start it. When a turn of an agent ends while the music plays, the call of
[Attention](attention.md) is a soft chime in the key of the music, not a system sound.

## Main design choices

- zendo makes the music from oscillators and noise. It plays no recording and no samples.
- A fixed loop of four chords keeps the mood. Random small changes in each bar keep it from
  sounding like a loop.
- The music is a background: it pauses while the terminal of zendo has no focus, and it never waits
  for a key. Only the question at the first start takes
  the focus, once.
- The chime of a turn is a part of the music, so that it sounds in tune with the chords.
- How zendo makes and plays the music on macOS and on Linux, with the notes of the first
  implementation, is in [ADR 0006](../architecture/decisions/0006-lofi-music-from-a-synth.md).

## Architecturally significant requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| LOFI-1 | Ubiquitous | zendo shall play only music that zendo makes. | User, 2026-11-13 |
| LOFI-2 | Ubiquitous | zendo shall make the music on the machine of the user, without the network. | Design |
| LOFI-3 | Unwanted   | If the machine has no audio output, then zendo shall work as before, show the music button in gray, and, when the user presses it, say that there is no audio output. | Design |
| LOFI-4 | Ubiquitous | The music shall not slow a key, a frame of the screen, or a turn of an agent. | Design |
| LOFI-14 | Optional  | Where zendo runs on macOS, or on Linux with PipeWire, PulseAudio or ALSA, and not over SSH, zendo shall play the music without a package that the user installs for it. | User, 2026-11-14 |

### Acceptance tests

**`lofi renders without gaps`** (unit test; verifies LOFI-2, LOFI-7)

- Given the music generator with a fixed random seed.
- When it renders eight minutes of samples without an audio device.
- Then no second of the output is silent, and no sample clips.

**`lofi without audio`** (mock audio output; verifies LOFI-3)

- Given a machine with no audio output.
- When the user starts zendo and clicks the music button.
- Then zendo opens as usual, the button is gray, and the status line says that there is no audio
  output.

**`lofi toggles`** (mock audio output; verifies LOFI-8, LOFI-10)

- Given zendo with a test audio output and the music on.
- When the user clicks the music button three times.
- Then the music stops, plays, and stops; the button is gray, blue, and gray; and
  `~/.zendo/config.toml` holds `music = false`.

**`lofi asks at the first start`** (mock audio output; verifies LOFI-11, LOFI-12)

- Given a config without the setting `music`.
- When zendo starts, and the user answers that the music shall not play at each start.
- Then the music played while the float was open, it stops, the config holds `music = false`, and the
  next start shows no float.

**`lofi chime in the key`** (unit test; verifies LOFI-13)

- Given the music generator in the bar of Fmaj9.
- When a turn ends.
- Then the generator plays a chime on a note of Fmaj9, and zendo does not start the system sound.

## Detailed requirements

| ID      | Pattern    | Requirement | Source |
| ------- | ---------- | ----------- | ------ |
| LOFI-5  | Ubiquitous | The music shall be an electric piano on four jazz chords, a soft bass, soft drums with swing, and vinyl crackle. | User, 2026-11-13 |
| LOFI-6  | Ubiquitous | The music shall have no koto and no other plucked melody. | User, 2026-11-13 |
| LOFI-7  | Ubiquitous | The music shall play without breaks until the user stops it: no bars without drums, no pause at the end of the loop. | User, 2026-11-13 |
| LOFI-8  | Event      | When the user clicks the music button or presses the music key (default `alt-m`), zendo shall stop the music if it plays, start it otherwise, and save the setting `music` in `~/.zendo/config.toml`. | User, 2026-11-14 |
| LOFI-9  | Retired    | Retired on 2026-11-14: the user did not ask for a music file of the user. | User, 2026-11-14 |
| LOFI-10 | Ubiquitous | zendo shall show the music button `[music]` left of the sound button in the top line: in the blue of the theme while the music plays, in the gray of the theme while it is stopped or not available. | User, 2026-11-14 |
| LOFI-11 | Optional   | Where `~/.zendo/config.toml` has `music = false`, zendo shall start with the music stopped; with `music = true` or without the setting, with the music playing. | User, 2026-11-14 |
| LOFI-12 | Event      | When zendo starts and `~/.zendo/config.toml` has no setting `music`, zendo shall play the music, ask in a float whether the music shall play at each start, and save the answer as `music`. | User, 2026-11-14 |
| LOFI-15 | State      | While the terminal of zendo has no focus, zendo shall pause the music; when the terminal gets the focus, zendo shall play it again, where the music is on. | User, 2026-11-14 |
| LOFI-13 | Event      | When a turn ends while the music plays and the sound is on (ATTN-1), zendo shall play a soft chime on a note of the current chord in the music, in place of the sound of ATTN-12. | User, 2026-11-14 |
