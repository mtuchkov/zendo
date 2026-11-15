//! The tonal voices of the synth (ADR 0006, LOFI-5): the old electric piano, the soft bass, and the
//! bell of the chime (LOFI-13).

use super::dsp::{cents, mtof, samples, sine, triangle, Env, Osc, Rng, Svf, Voice, FLOOR, Q_SOFT};

/// The filter of the piano moves to a new frequency every so many samples.
const RETUNE: u32 = 16;

/// A note of the piano: a triangle at −6 cents, a sine at +6 cents and a quiet sine one octave up,
/// all with the tape wow of the note, through a low-pass that closes from 2,400 Hz to 800 Hz.
#[derive(Clone, Copy, Default)]
pub(crate) struct Piano {
    tri: Osc,
    sine: Osc,
    octave: Osc,
    env: Env,
    filter: Svf,
    cutoff: f32,
    close: f32,
    closing: u32,
    rate: f32,
}

impl Piano {
    /// Start `note` after `wait` samples for `dur` seconds at the level `vel`; `wow` in cents.
    pub(crate) fn start(&mut self, rate: f32, note: u8, wait: u32, dur: f32, vel: f32, wow: f32) {
        let f = mtof(f32::from(note));
        let closing = samples(dur + 0.5, rate).max(1);
        *self = Piano {
            tri: Osc::new(f * cents(wow - 6.0), rate),
            sine: Osc::new(f * cents(wow + 6.0), rate),
            octave: Osc::new(2.0 * f * cents(wow), rate),
            filter: Svf::new(2400.0, Q_SOFT, rate),
            cutoff: 2400.0,
            close: (800.0f32 / 2400.0).powf(1.0 / closing as f32),
            closing,
            rate,
            ..Piano::default()
        };
        let ramps = [
            (vel, samples(0.02, rate)),
            (vel * 0.45, samples(dur * 0.6, rate)),
            (FLOOR, samples(dur * 0.4 + 1.1, rate)),
        ];
        self.env.start(wait, ramps);
    }

    fn close_filter(&mut self) {
        if self.closing == 0 {
            return;
        }
        self.closing -= 1;
        self.cutoff *= self.close;
        if self.closing.is_multiple_of(RETUNE) {
            self.filter.tune(self.cutoff, Q_SOFT, self.rate);
        }
    }
}

impl Voice for Piano {
    fn tick(&mut self, _rng: &mut Rng) -> f32 {
        if !self.env.active() {
            return 0.0;
        }
        let e = self.env.tick();
        if e == 0.0 {
            return 0.0;
        }
        self.close_filter();
        let x = triangle(self.tri.step()) + sine(self.sine.step()) + 0.12 * sine(self.octave.step());
        self.filter.tick(x).low * e
    }

    fn busy(&self) -> f32 {
        self.env.busy()
    }
}

/// A note of the bass: a triangle through a 380 Hz low-pass.
#[derive(Clone, Copy, Default)]
pub(crate) struct Bass {
    osc: Osc,
    env: Env,
    filter: Svf,
}

impl Bass {
    /// Start `note` after `wait` samples for `dur` seconds.
    pub(crate) fn start(&mut self, rate: f32, note: u8, wait: u32, dur: f32) {
        self.osc = Osc::new(mtof(f32::from(note)), rate);
        self.filter = Svf::new(380.0, Q_SOFT, rate);
        let ramps = [
            (0.32, samples(0.015, rate)),
            (0.32 * 0.45, samples(dur * 0.7, rate)),
            (FLOOR, samples(0.25, rate)),
        ];
        self.env.start(wait, ramps);
    }
}

impl Voice for Bass {
    fn tick(&mut self, _rng: &mut Rng) -> f32 {
        if !self.env.active() {
            return 0.0;
        }
        let e = self.env.tick();
        if e == 0.0 {
            return 0.0;
        }
        self.filter.tick(triangle(self.osc.step())).low * e
    }

    fn busy(&self) -> f32 {
        self.env.busy()
    }
}

/// The level of the chime (ADR 0006).
const CHIME: f32 = 0.12;

/// The bell of the chime: a sine with a quiet partial at 2.76 times its frequency, which fades
/// twice as fast; attack 5 ms, decay 1.8 s.
#[derive(Clone, Copy, Default)]
pub(crate) struct Chime {
    base: Osc,
    partial: Osc,
    env: Env,
}

impl Chime {
    pub(crate) fn start(&mut self, rate: f32, note: u8, wait: u32) {
        let f = mtof(f32::from(note));
        self.base = Osc::new(f, rate);
        self.partial = Osc::new(2.76 * f, rate);
        let ramps = [(CHIME, samples(0.005, rate)), (CHIME, 1), (FLOOR, samples(1.8, rate))];
        self.env.start(wait, ramps);
    }
}

impl Voice for Chime {
    fn tick(&mut self, _rng: &mut Rng) -> f32 {
        if !self.env.active() {
            return 0.0;
        }
        let e = self.env.tick();
        let partial = 0.25 * sine(self.partial.step()) * e / CHIME;
        (sine(self.base.step()) + partial) * e
    }

    fn busy(&self) -> f32 {
        self.env.busy()
    }
}
