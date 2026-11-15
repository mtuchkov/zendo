//! The synth of the lofi music (docs/zendo/lofi.md, ADR 0006): it renders the samples in the audio
//! thread from a seeded random generator, so no other thread waits for it (LOFI-4). All buffers are
//! allocated in [`Synth::new`]; [`Synth::render`] allocates nothing, takes no lock and does no I/O.

mod band;
mod drums;
mod dsp;
mod master;
mod piano;
mod room;

use band::{chime_note, chord_of, Band, Beat, STEP};
use dsp::Rng;
use master::Master;

/// The sixteenths of the music, sample by sample.
struct Clock {
    /// The samples in a sixteenth; not a whole number.
    step: f64,
    /// Samples until the next sixteenth.
    until: f64,
    /// The sixteenths played since the start.
    played: u64,
    /// Samples since the start.
    elapsed: u64,
}

impl Clock {
    fn new(rate: f32) -> Self {
        Clock { step: f64::from(rate) * f64::from(STEP), until: 0.0, played: 0, elapsed: 0 }
    }

    /// The sixteenth that starts at this sample, if one does.
    fn tick(&mut self, rate: f32) -> Option<Beat> {
        let time = (self.elapsed as f64 / f64::from(rate)) as f32;
        self.elapsed += 1;
        self.until -= 1.0;
        if self.until > 0.0 {
            return None;
        }
        self.until += self.step;
        let n = self.played;
        self.played += 1;
        Some(Beat { bar: n / 16, step: (n % 16) as u8, time })
    }

    /// The bar of the last sixteenth played.
    fn bar(&self) -> u64 {
        self.played.saturating_sub(1) / 16
    }
}

/// The music generator: a loop of four jazz chords on piano, bass, drums with swing, and vinyl
/// (LOFI-5), endless and without breaks (LOFI-7).
pub struct Synth {
    rate: f32,
    rng: Rng,
    clock: Clock,
    band: Band,
    master: Master,
    chime: bool,
    last_chime: Option<u8>,
}

impl Synth {
    /// A synth at `rate` samples per second, its random choices from `seed`; it starts silent.
    /// Rates below 1,000 Hz count as 1,000 Hz.
    pub fn new(rate: u32, seed: u64) -> Self {
        let rate = rate.max(1000) as f32;
        Synth {
            rate,
            rng: Rng::new(seed),
            clock: Clock::new(rate),
            band: Band::new(rate),
            master: Master::new(rate),
            chime: false,
            last_chime: None,
        }
    }

    /// Fill `out` with interleaved stereo samples (left, right), each in -1..=1.
    pub fn render(&mut self, out: &mut [f32]) {
        let (frames, rest) = out.as_chunks_mut::<2>();
        for frame in frames {
            let (l, r) = if self.silent() { (0.0, 0.0) } else { self.frame() };
            *frame = [l, r];
        }
        rest.fill(0.0);
    }

    fn frame(&mut self) -> (f32, f32) {
        if let Some(beat) = self.clock.tick(self.rate) {
            self.band.play(&beat, &mut self.rng);
            self.ring(&beat);
        }
        let (l, r) = self.band.tick(&mut self.rng);
        self.master.tick(l, r)
    }

    /// A chime that waits sounds on the next eighth note (LOFI-13).
    fn ring(&mut self, beat: &Beat) {
        if !self.chime || beat.step % 2 == 1 {
            return;
        }
        let note = chime_note(beat.bar, &mut self.rng);
        self.band.chime(note);
        self.last_chime = Some(note);
        self.chime = false;
    }

    /// Fade in (true) with a time constant of 0.6 s, or out (false) with 0.35 s. From silence the
    /// music starts again at the first bar.
    pub fn set_playing(&mut self, on: bool) {
        if on && self.silent() {
            self.clock = Clock::new(self.rate);
            self.band.clear();
            self.master.clear();
            self.chime = false;
        }
        self.master.set_playing(on);
    }

    /// Faded out completely: the output may stop.
    pub fn silent(&self) -> bool {
        self.master.silent()
    }

    /// Ring the chime on the next eighth note, on a note of the current chord (LOFI-13). While the
    /// music fades out or is silent, there is no chime.
    pub fn chime(&mut self) {
        self.chime |= self.master.playing();
    }

    /// The MIDI notes of the chord of the current bar.
    pub fn chord(&self) -> [u8; 5] {
        chord_of(self.clock.bar()).keys
    }

    /// The MIDI note of the last chime that sounded, for the tests.
    pub fn last_chime(&self) -> Option<u8> {
        self.last_chime
    }
}

#[cfg(test)]
#[path = "synth_tests.rs"]
mod tests;
