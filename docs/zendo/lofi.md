---
status: "draft"
revision: "1"
date: "2026-11-13"
parent: "zendo.md"
prev: "site.md"
level: "what the user hears when zendo plays lofi music, and the decisions of the user"
note: "Agent draft from the requests of the user on 2026-11-13. zendo does not play music yet: the landing page does. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document. The notes of the first implementation are in ADR 0006."
---

# Lofi music

Sources: the requests of the user on 2026-11-13 for the [landing page](site.md), and
`site/index.html` in zendo 0.49.5.

## What the user hears

zendo can play calm lofi music while the user reads and writes. The music is an old electric piano
on four jazz chords, a soft bass, soft drums with swing, and the crackle of a vinyl record. It plays
without breaks until the user stops it. zendo makes the music itself, so no one owns it and it needs
no network. The user can put a CC0 track in its place.

The landing page plays the same music today. zendo will get it later.

## Main design choices

- zendo makes the music from oscillators and noise. It plays no recording and no samples.
- A fixed loop of four chords keeps the mood. Random small changes in each bar keep it from
  sounding like a loop.
- The music is a background: it never takes the focus and never waits for a key.
- How zendo makes and plays the music, with the notes of the first implementation, is in
  [ADR 0006](../architecture/decisions/0006-lofi-music-from-a-synth.md).

## Architecturally significant requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| LOFI-1 | Ubiquitous | zendo shall play only music that no one owns: music that zendo makes, or a CC0 file of the user. | User, 2026-11-13 |
| LOFI-2 | Ubiquitous | zendo shall make the music on the machine of the user, without the network. | Design |
| LOFI-3 | Unwanted   | If the machine has no audio output, then zendo shall work as before and shall show the music as not available. | Design |
| LOFI-4 | Ubiquitous | The music shall not slow a key, a frame of the screen, or a turn of an agent. | Design |

### Acceptance tests

**`lofi renders without gaps`** (unit test; verifies LOFI-2, LOFI-7)

- Given the music generator with a fixed random seed.
- When it renders eight minutes of samples without an audio device.
- Then no second of the output is silent, and no sample clips.

**`lofi without audio`** (mock agent; verifies LOFI-3)

- Given a machine with no audio output.
- When the user starts zendo and presses the music key.
- Then zendo opens as usual and shows the music as not available.

**`lofi toggles`** (mock agent; verifies LOFI-8)

- Given zendo with a test audio output.
- When the user presses the music key three times.
- Then the music plays, stops, and plays.

## Detailed requirements

| ID     | Pattern    | Requirement | Source |
| ------ | ---------- | ----------- | ------ |
| LOFI-5 | Ubiquitous | The music shall be an electric piano on four jazz chords, a soft bass, soft drums with swing, and vinyl crackle. | User, 2026-11-13 |
| LOFI-6 | Ubiquitous | The music shall have no koto and no other plucked melody. | User, 2026-11-13 |
| LOFI-7 | Ubiquitous | The music shall play without breaks until the user stops it: no bars without drums, no pause at the end of the loop. | User, 2026-11-13 |
| LOFI-8 | Event      | When the user presses the music key, zendo shall stop the music if it plays, and start it otherwise. | User, 2026-11-13 |
| LOFI-9 | Optional   | Where the user sets a CC0 audio file, zendo shall play that file in a loop in place of its own music. | User, 2026-11-13 |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| Which key and which button start the music? | LOFI-8 | User |
| Does zendo start with the music on or off? The landing page starts with it on. | LOFI-8 | User |
| Where does the user set the CC0 file, and is the setting per repository or per user? | LOFI-9 | User |
| Does the bell of [Attention](attention.md) play over the music, or does the music duck? | LOFI-4 | User |
| Is `rodio` the right audio crate? See the draft [ADR 0006](../architecture/decisions/0006-lofi-music-from-a-synth.md). | LOFI-2, LOFI-3 | User |
