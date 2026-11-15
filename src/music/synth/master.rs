//! The master of the synth (ADR 0006): the level 0.6 with the fade in and out, then a 4,200 Hz
//! low-pass and a compressor (−20 dB, ratio 3), as the first implementation.

use super::dsp::{Svf, Q_SOFT};

const LEVEL: f32 = 0.6;
/// The time constants of the fade in and of the fade out, in seconds.
const FADE_IN: f32 = 0.6;
const FADE_OUT: f32 = 0.35;
/// Below this fade the music has stopped: −60 dB.
const GONE: f32 = 1e-3;

/// The compressor of Web Audio, in short: the threshold and the ratio, a soft knee of 30 dB above
/// the threshold, attack 3 ms, release 250 ms, and the makeup gain that Web Audio adds by itself.
const THRESHOLD: f32 = -20.0;
const KNEE: f32 = 30.0;
const SLOPE: f32 = 1.0 - 1.0 / 3.0;

/// The gain reduction in dB of a level in dB.
fn reduction(db: f32) -> f32 {
    let over = db - THRESHOLD;
    if over <= 0.0 {
        0.0
    } else if over < KNEE {
        SLOPE * over * over / (2.0 * KNEE)
    } else {
        SLOPE * (over - KNEE / 2.0)
    }
}

/// A sample in -1..=1, and never NaN.
fn safe(x: f32) -> f32 {
    if x.is_finite() { x.clamp(-1.0, 1.0) } else { 0.0 }
}

fn gain_of(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

struct Compressor {
    level: f32,
    attack: f32,
    release: f32,
    makeup: f32,
}

impl Compressor {
    fn new(rate: f32) -> Self {
        let coef = |secs: f32| 1.0 - (-1.0 / (secs * rate)).exp();
        // Web Audio: the makeup is the gain at 0 dB, inverted, to the power 0.6.
        let makeup = gain_of(0.6 * reduction(0.0));
        Compressor { level: 0.0, attack: coef(0.003), release: coef(0.25), makeup }
    }

    fn tick(&mut self, l: f32, r: f32) -> (f32, f32) {
        let peak = l.abs().max(r.abs());
        let coef = if peak > self.level { self.attack } else { self.release };
        self.level += (peak - self.level) * coef;
        let db = 20.0 * (self.level + 1e-9).log10();
        let g = self.makeup * gain_of(-reduction(db));
        (l * g, r * g)
    }
}

pub(crate) struct Master {
    rate: f32,
    fade: f32,
    target: f32,
    fade_in: f32,
    fade_out: f32,
    tone: [Svf; 2],
    comp: Compressor,
}

impl Master {
    pub(crate) fn new(rate: f32) -> Self {
        let coef = |secs: f32| 1.0 - (-1.0 / (secs * rate)).exp();
        Master {
            rate,
            fade: 0.0,
            target: 0.0,
            fade_in: coef(FADE_IN),
            fade_out: coef(FADE_OUT),
            tone: [Svf::new(4200.0, Q_SOFT, rate); 2],
            comp: Compressor::new(rate),
        }
    }

    pub(crate) fn set_playing(&mut self, on: bool) {
        self.target = if on { 1.0 } else { 0.0 };
    }

    pub(crate) fn playing(&self) -> bool {
        self.target > 0.0
    }

    /// Faded out completely.
    pub(crate) fn silent(&self) -> bool {
        self.target == 0.0 && self.fade == 0.0
    }

    /// Forget the past samples before the music plays again.
    pub(crate) fn clear(&mut self) {
        *self = Master { target: self.target, ..Master::new(self.rate) };
    }

    /// One stereo sample of the mix through the master, each in -1..=1.
    pub(crate) fn tick(&mut self, l: f32, r: f32) -> (f32, f32) {
        let coef = if self.target > self.fade { self.fade_in } else { self.fade_out };
        self.fade += (self.target - self.fade) * coef;
        if self.target == 0.0 && self.fade < GONE {
            self.fade = 0.0;
            return (0.0, 0.0);
        }
        let g = LEVEL * self.fade;
        let [tl, tr] = &mut self.tone;
        let (l, r) = self.comp.tick(tl.tick(l * g).low, tr.tick(r * g).low);
        (safe(l), safe(r))
    }
}
