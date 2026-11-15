//! The band of the synth and what it plays in each sixteenth (ADR 0006): the piano part, the bass,
//! the drums in every bar (LOFI-7), the vinyl, the chime, and the room that they share. No koto
//! and no other plucked melody (LOFI-6).

use super::drums::{Hit, Kick, Vinyl};
use super::dsp::{samples, Pool, Rng};
use super::piano::{Bass, Chime, Piano};
use super::room::Room;

/// The time of the music: 72 BPM, four sixteenths to a beat.
pub(crate) const BEAT: f32 = 60.0 / 72.0;
pub(crate) const STEP: f32 = BEAT / 4.0;
/// Each odd sixteenth is late by this part of a sixteenth.
const SWING: f32 = 0.28;

/// A chord: the root of the bass, and the five notes of the piano.
pub(crate) struct Chord {
    pub(crate) bass: u8,
    pub(crate) keys: [u8; 5],
}

/// One chord per bar, in a loop: Fmaj9, Em9, Dm9, Cmaj9.
pub(crate) const CHORDS: [Chord; 4] = [
    Chord { bass: 41, keys: [53, 57, 60, 64, 67] },
    Chord { bass: 40, keys: [52, 55, 59, 62, 66] },
    Chord { bass: 38, keys: [50, 53, 57, 60, 64] },
    Chord { bass: 36, keys: [48, 52, 55, 59, 62] },
];

/// The chord of a bar.
pub(crate) fn chord_of(bar: u64) -> &'static Chord {
    &CHORDS[(bar % 4) as usize]
}

/// The chime of a bar (LOFI-13): the fifth or the ninth of the chord, two octaves above the root
/// of the bass.
pub(crate) fn chime_note(bar: u64, rng: &mut Rng) -> u8 {
    chord_of(bar).bass + 24 + if rng.chance(0.5) { 14 } else { 7 }
}

/// A place in the music: the bar, its sixteenth, and the time since the start in seconds.
pub(crate) struct Beat {
    pub(crate) bar: u64,
    pub(crate) step: u8,
    pub(crate) time: f32,
}

pub(crate) struct Band {
    rate: f32,
    piano: Pool<Piano, 12>,
    bass: Pool<Bass, 4>,
    kick: Pool<Kick, 2>,
    snare: Pool<Hit, 2>,
    hat: Pool<Hit, 4>,
    chime: Pool<Chime, 2>,
    vinyl: Vinyl,
    room: Room,
}

impl Band {
    /// The band at `rate`; all its buffers are allocated here (LOFI-4).
    pub(crate) fn new(rate: f32) -> Self {
        Band {
            rate,
            piano: Pool::new(),
            bass: Pool::new(),
            kick: Pool::new(),
            snare: Pool::new(),
            hat: Pool::new(),
            chime: Pool::new(),
            vinyl: Vinyl::new(rate),
            room: Room::new(rate),
        }
    }

    /// Silence every voice and the room, before the music plays again.
    pub(crate) fn clear(&mut self) {
        self.piano.clear();
        self.bass.clear();
        self.kick.clear();
        self.snare.clear();
        self.hat.clear();
        self.chime.clear();
        self.vinyl.clear();
        self.room.clear();
    }

    /// One stereo sample of the band, before the master.
    pub(crate) fn tick(&mut self, rng: &mut Rng) -> (f32, f32) {
        let piano = self.piano.tick(rng);
        let snare = self.snare.tick(rng);
        let hat = self.hat.tick(rng);
        let chime = self.chime.tick(rng);
        let rest = self.bass.tick(rng) + self.kick.tick(rng) + self.vinyl.tick(rng);
        let dry = 0.8 * piano + snare + hat + chime + rest;
        let send = 0.35 * piano + 0.4 * snare + 0.05 * hat + 0.5 * chime;
        let (l, r) = self.room.tick(send);
        (dry + 0.3 * l, dry + 0.3 * r)
    }

    /// Ring the chime `note` now (LOFI-13).
    pub(crate) fn chime(&mut self, note: u8) {
        let rate = self.rate;
        self.chime.play(|v| v.start(rate, note, 0));
    }

    /// Play the notes of the sixteenth `beat`; odd sixteenths swing.
    pub(crate) fn play(&mut self, beat: &Beat, rng: &mut Rng) {
        let swing = if beat.step % 2 == 1 { samples(SWING * STEP, self.rate) } else { 0 };
        self.keys(beat, swing, rng);
        self.bass_line(beat, swing, rng);
        self.drums(beat, swing, rng);
    }

    /// The piano: the full chord on sixteenth 0; the upper four notes on sixteenth 10 in 60 % of
    /// bars. Tape wow: ±7 cents at 0.27 Hz.
    fn keys(&mut self, beat: &Beat, swing: u32, rng: &mut Rng) {
        let (notes, gap, dur, vel) = match beat.step {
            0 => (&chord_of(beat.bar).keys[..], 0.018, 2.2 * BEAT, 0.075),
            10 if rng.chance(0.6) => (&chord_of(beat.bar).keys[1..], 0.022, BEAT, 0.05),
            _ => return,
        };
        let (rate, wow) = (self.rate, 7.0 * (beat.time * std::f32::consts::TAU * 0.27).sin());
        notes.iter().enumerate().for_each(|(i, &n)| {
            let wait = swing + samples(i as f32 * gap, rate);
            self.piano.play(|v| v.start(rate, n, wait, dur, vel, wow));
        });
    }

    /// The bass: the root on sixteenth 0; the root, or in 30 % of bars the fifth, on sixteenth 8;
    /// in 35 % of bars the octave on sixteenth 14.
    fn bass_line(&mut self, beat: &Beat, swing: u32, rng: &mut Rng) {
        let root = chord_of(beat.bar).bass;
        let (note, dur) = match beat.step {
            0 => (root, 1.6 * BEAT),
            8 => (root + if rng.chance(0.3) { 7 } else { 0 }, 0.9 * BEAT),
            14 if rng.chance(0.35) => (root + 12, 0.4 * BEAT),
            _ => return,
        };
        let rate = self.rate;
        self.bass.play(|v| v.start(rate, note, swing, dur));
    }

    /// The drums, in every bar: kick, snare on 4 and 12, hats on the even sixteenths and some odd.
    fn drums(&mut self, beat: &Beat, swing: u32, rng: &mut Rng) {
        let rate = self.rate;
        let kick = match beat.step {
            0 | 10 => Some(0.8),
            7 if rng.chance(0.4) => Some(0.45),
            14 if beat.bar % 4 == 3 => Some(0.8),
            _ => None,
        };
        if let Some(vel) = kick {
            self.kick.play(|v| v.start(rate, swing, vel));
        }
        if beat.step == 4 || beat.step == 12 {
            self.snare.play(|v| v.snare(rate, swing, 0.22));
        }
        let hat = match beat.step % 2 {
            0 => Some(rng.range(0.05, 0.1)),
            _ if rng.chance(0.15) => Some(0.035),
            _ => None,
        };
        if let Some(vel) = hat {
            self.hat.play(|v| v.hat(rate, swing, vel));
        }
    }
}
