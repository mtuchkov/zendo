---
status: "approved"
approved: "2026-11-14"
date: "2026-11-14"
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

For the audio output (LOFI-14: no package that the user installs for the music):

| Option | Benefit | Cost or constraint |
| ------ | ------- | ------------------ |
| `cpal` on both platforms | One code path. | On Linux it needs the ALSA headers to build; `install.sh` builds zendo and cannot install them without `sudo` and a question. |
| `cpal` on macOS, a system player on Linux | Nothing to install on either platform: CoreAudio is in macOS; `paplay`, `aplay` or `pw-cat` comes with PipeWire, PulseAudio or ALSA. | Two output paths. A pipe to a child process adds some latency, which a background music does not feel. |
| `rodio` | A mixer and file decoders. | zendo plays no files (LOFI-1), so the decoders are weight; the same ALSA problem as `cpal`. |

## Decision

zendo makes the music with its own synth. On macOS it plays
the samples through `cpal` (CoreAudio), a dependency only for `target_os = "macos"`. On Linux it
writes 16-bit stereo samples at 48 kHz to the stdin of the first player that it finds: `paplay`
(it also plays through PipeWire), `aplay`, `pw-cat`, each with a latency of 100 ms, so that a stop
or a chime comes at once. Over SSH, or without an output, the music is not available (LOFI-3).

The output opens on its own thread, so the screen does not wait for it. The thread reports whether
it plays; when the output dies, as a sound server that restarts, the music becomes not available
and the bell of ATTN-1 rings again. While the music is off or paused, the output rests.

The music module shall render samples in the audio thread from a seeded random generator, so that
it needs no scheduler and no other thread waits for it. The rest of zendo shall see only commands
(start, stop, chime) and one state (plays, stopped, not available, which includes "still opening").

The chime of a turn (LOFI-13) is a voice of the synth: a soft bell (sine with a quiet partial at 2.76
times the frequency, attack 5 ms, decay 1.8 s, level 0.12, room send 0.5) on the ninth or the fifth
of the current chord, two octaves above the root, on the next eighth note.

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

- A new dependency on macOS only: `cpal`. Linux builds as before; `install.sh` does not change.
- On Linux the music needs one of `paplay`, `aplay`, `pw-cat`. `zendo doctor` reports which one it
  uses, or that the music is not available.
- The room is not a convolution with a 2.6 s impulse, as on the landing page: that costs too much in
  the audio thread. A Freeverb-style reverb takes its place: for each channel 8 damped combs and 4
  all-passes, a decay of 1.8 s, the right channel 0.52 ms longer. Ten seconds of music render in
  about 0.1 s in a release build.
- The landing page and zendo keep two copies of the synth: JavaScript and Rust. A change of the music
  changes both, and this record.

## Verification

- `lofi renders without gaps` (LOFI-2, LOFI-7): the synth with a fixed seed renders eight minutes
  without an audio device; no second is silent and no sample clips.
- `lofi without audio` (LOFI-3), `lofi toggles` (LOFI-8, LOFI-10), `lofi asks at the first start`
  (LOFI-11, LOFI-12) and `lofi chime in the key` (LOFI-13): tests with a mock audio output.
- A listen by the user, next to the landing page, before this record becomes `approved`.

## Decision record

Draft by the agent on 2026-11-13, at the request of the user. Revised on 2026-11-14 with the answers
of the user: no file of the user, `cpal` only on macOS, a system player on Linux, the chime in the
synth. Approved by the user on 2026-11-14.
