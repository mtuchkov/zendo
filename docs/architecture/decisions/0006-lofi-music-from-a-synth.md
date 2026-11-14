---
status: "draft"
date: "2026-11-13"
sources:
  - "../../zendo/lofi.md"
  - "../../zendo/site.md"
  - "The requests of the user for the landing page on 2026-11-13: music that no one owns, no koto, no breaks; and to keep the notes of the first implementation as a decision record"
  - "site/index.html in zendo 0.53.1: the first implementation, in Web Audio"
---

# 0006 Make the lofi music with a synth in zendo

## Context

zendo will play calm lofi music (LOFI-5). The music must belong to no one (LOFI-1), play without the
network (LOFI-2), play without breaks (LOFI-7), and never slow a key, a frame or a turn (LOFI-4). If
the machine has no audio output, zendo works as before (LOFI-3).

The landing page already plays such music. It makes the music in the browser from oscillators and
noise, so there is no file and no owner. zendo needs the same music in a terminal program in Rust.

## Options

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| A synth in zendo that makes the music | No file, no licence, no network. Small. Never the same bar twice. The landing page proves the sound. | Code to write and tune. Music quality depends on the synth. |
| Ship a CC0 track in zendo | Real recorded music. Little code beyond playback. | Megabytes in the binary or the installer. One track repeats. A check of each licence. |
| Only a file of the user | No music code beyond playback. | No music for most users. |
| A web radio | Endless music. | Needs the network (breaks LOFI-2). Unknown owners (breaks LOFI-1). |

For the audio output:

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| `rodio` (MIT or Apache-2.0) | A mixer and a `Source` trait for the synth; it decodes MP3 and OGG for the CC0 file of the user (LOFI-9). Built on `cpal`. | More dependencies than `cpal` alone. |
| `cpal` (Apache-2.0) | The lowest layer: one audio callback, few dependencies. | zendo must mix and decode files itself. |

## Decision

Proposal, not yet approved by the user: zendo makes the music with its own synth, and plays it, or a
CC0 file of the user, through `rodio`.

The music module shall render samples in the audio thread from a seeded random generator, so that
it needs no scheduler and no other thread waits for it. The rest of zendo shall see only commands
(start, stop) and one state (plays, stopped, not available).

The synth shall follow the first implementation on the landing page:

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

Not in the synth: a koto on the C major pentatonic, and two bars without drums at the start and in
every 16 bars. The user removed both on 2026-11-13 (LOFI-6, LOFI-7).

## Consequences

- New dependencies: `rodio`, and through it `cpal` and a decoder for MP3 and OGG.
- On Linux, `cpal` needs the ALSA headers to build. The installer builds zendo, so it must install
  them or build zendo without music. `zendo doctor` reports when music is not available.
- The room is a convolution with a 2.6 s impulse. If it costs too much in the audio thread, a cheaper
  reverb takes its place; the sound may change a little.
- The landing page and zendo keep two copies of the synth: JavaScript and Rust. A change of the music
  changes both, and this record.

## Verification

- `lofi renders without gaps` (LOFI-2, LOFI-7): the synth with a fixed seed renders eight minutes
  without an audio device; no second is silent and no sample clips.
- `lofi without audio` (LOFI-3) and `lofi toggles` (LOFI-8): mock-agent tests in `cargo test`.
- A listen by the user, next to the landing page, before this record becomes `approved`.

## Decision record

Draft by the agent on 2026-11-13, at the request of the user. Not reviewed. The user approves or
rejects it when zendo gets the music.
