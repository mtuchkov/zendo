---
status: "draft"
revision: "1"
date: "2026-11-13"
parent: "zendo.md"
prev: "site.md"
level: "what the user hears when zendo plays lofi music, the decisions of the user, and the notes of the first implementation on the landing page"
note: "Agent draft from the requests of the user on 2026-11-13. zendo does not play music yet: the landing page does. Not reviewed by the user. The requirement IDs are proposals until the user reviews this document. The implementation notes are kept on request of the user until the code of zendo has them."
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

## Implementation notes

From `site/index.html` (Web Audio). Not requirements: a guide for the port to zendo. In zendo, the
synth renders samples in the audio callback with a seeded random generator, so it needs no
look-ahead scheduler.

| Part | First implementation |
| ---- | -------------------- |
| Time | 72 BPM, a bar of 16 sixteenths. Each odd sixteenth is late by 0.28 of a sixteenth (swing). |
| Chords | One chord per bar, in a loop: Fmaj9 (MIDI 53 57 60 64 67), Em9 (52 55 59 62 66), Dm9 (50 53 57 60 64), Cmaj9 (48 52 55 59 62). |
| Piano | Triangle at −6 cents, sine at +6 cents, sine one octave up at 0.12. Tape wow: ±7 cents at 0.27 Hz. Low-pass from 2,400 Hz to 800 Hz over the note. Attack 20 ms, decay to 45 %, release about 1.1 s. |
| Piano part | Sixteenth 0: the full chord, 18 ms between notes, 2.2 beats, level 0.075. Sixteenth 10, in 60 % of bars: the upper four notes, 1 beat, level 0.05. |
| Bass | Triangle through a 380 Hz low-pass, level 0.32. Root an octave below the chord (MIDI 41, 40, 38, 36) on sixteenth 0 for 1.6 beats; root, or in 30 % of bars the fifth, on sixteenth 8; in 35 % of bars the octave on sixteenth 14. |
| Kick | Sine from 115 Hz to 42 Hz in 0.14 s, decay 0.42 s. Sixteenths 0 and 10 at 0.8; sixteenth 7 in 40 % of bars at 0.45; sixteenth 14 in each fourth bar. |
| Snare | Noise through a 1,700 Hz band-pass (Q 0.7), decay 0.2 s, level 0.22, on sixteenths 4 and 12. |
| Hat | Noise through a 7,600 Hz high-pass, decay 45 ms. Even sixteenths at 0.05 to 0.1; odd sixteenths in 15 % of cases at 0.035. |
| Vinyl | A 4 s loop of noise at ±0.01 with spikes of ±0.8 at a rate of 0.0006, through a 2,400 Hz band-pass (Q 0.4), level 0.22. |
| Room | A 2.6 s stereo noise impulse that decays as (1 − t)^2.4, wet level 0.3. Piano send 0.35, snare send 0.4. |
| Master | Level 0.6, fade in with a time constant of 0.6 s and out with 0.35 s, then a 4,200 Hz low-pass and a compressor (−20 dB, ratio 3). |
| Removed | A koto on the C major pentatonic, and two bars without drums at the start and in every 16 bars. The user removed both on 2026-11-13. |

## Open points

| Open question | Affects | Answered by |
| ------------- | ------- | ----------- |
| Which key and which button start the music? | LOFI-8 | User |
| Does zendo start with the music on or off? The landing page starts with it on. | LOFI-8 | User |
| Where does the user set the CC0 file, and is the setting per repository or per user? | LOFI-9 | User |
| Does the bell of [Attention](attention.md) play over the music, or does the music duck? | LOFI-4 | User |
| Which audio crate plays the samples? This needs a decision record. | LOFI-2, LOFI-3 | User |
