//! The soft drums and the vinyl of the synth (ADR 0006, LOFI-5): a kick from a falling sine, a
//! snare and a hat from filtered noise, and the crackle of a record.

use super::dsp::{samples, sine, Env, Osc, Rng, Svf, Voice};

/// A kick: a sine that falls from 115 Hz to 42 Hz in 0.14 s, and decays in 0.42 s.
#[derive(Clone, Copy, Default)]
pub(crate) struct Kick {
    osc: Osc,
    env: Env,
    freq: f32,
    fall: f32,
    falling: u32,
    rate: f32,
}

impl Kick {
    pub(crate) fn start(&mut self, rate: f32, wait: u32, vel: f32) {
        let falling = samples(0.14, rate).max(1);
        *self = Kick {
            osc: Osc::new(115.0, rate),
            freq: 115.0,
            fall: (42.0f32 / 115.0).powf(1.0 / falling as f32),
            falling,
            rate,
            ..Kick::default()
        };
        self.env.hit(wait, vel, samples(0.42, rate));
    }
}

impl Voice for Kick {
    fn tick(&mut self, _rng: &mut Rng) -> f32 {
        if !self.env.active() {
            return 0.0;
        }
        let e = self.env.tick();
        if e == 0.0 {
            return 0.0;
        }
        if self.falling > 0 {
            self.falling -= 1;
            self.freq *= self.fall;
            self.osc.set(self.freq, self.rate);
        }
        sine(self.osc.step()) * e
    }

    fn busy(&self) -> f32 {
        self.env.busy()
    }
}

/// The level of white noise at `rate` that has the spectrum of white noise at ±1 at 48 kHz, as in
/// the first implementation: the same drums and vinyl at any rate.
fn white(rate: f32) -> f32 {
    (rate / 48000.0).sqrt()
}

/// A hit of filtered noise: the snare through a band-pass, the hat through a high-pass.
#[derive(Clone, Copy, Default)]
pub(crate) struct Hit {
    filter: Svf,
    env: Env,
    high: bool,
    white: f32,
}

impl Hit {
    /// The snare: a 1,700 Hz band-pass (Q 0.7), decay 0.2 s.
    pub(crate) fn snare(&mut self, rate: f32, wait: u32, vel: f32) {
        *self = Hit { filter: Svf::new(1700.0, 0.7, rate), high: false, white: white(rate), ..Hit::default() };
        self.env.hit(wait, vel, samples(0.2, rate));
    }

    /// The hat: a 7,600 Hz high-pass, decay 45 ms.
    pub(crate) fn hat(&mut self, rate: f32, wait: u32, vel: f32) {
        *self = Hit { filter: Svf::new(7600.0, 0.75, rate), high: true, white: white(rate), ..Hit::default() };
        self.env.hit(wait, vel, samples(0.045, rate));
    }
}

impl Voice for Hit {
    fn tick(&mut self, rng: &mut Rng) -> f32 {
        if !self.env.active() {
            return 0.0;
        }
        let e = self.env.tick();
        if e == 0.0 {
            return 0.0;
        }
        let b = self.filter.tick(rng.noise() * self.white);
        if self.high { b.high * e } else { b.band * e }
    }

    fn busy(&self) -> f32 {
        self.env.busy()
    }
}

/// The vinyl: a soft hiss at ±0.01 with dusty spikes at ±0.8, through a 2,400 Hz band-pass (Q 0.4),
/// level 0.22. The first implementation loops 4 s of it; this one never repeats.
pub(crate) struct Vinyl {
    filter: Svf,
    spikes: f32,
    hiss: f32,
    spike: f32,
}

impl Vinyl {
    pub(crate) fn new(rate: f32) -> Self {
        // 0.0006 of the samples at 48 kHz: the same spikes per second, of the same area, at any rate.
        let spikes = 0.0006 * 48000.0 / rate;
        let (hiss, spike) = (0.01 * white(rate), 0.8 * rate / 48000.0);
        Vinyl { filter: Svf::new(2400.0, 0.4, rate), spikes, hiss, spike }
    }

    pub(crate) fn tick(&mut self, rng: &mut Rng) -> f32 {
        let level = if rng.chance(self.spikes) { self.spike } else { self.hiss };
        0.22 * self.filter.tick(rng.noise() * level).band
    }

    pub(crate) fn clear(&mut self) {
        self.filter.clear();
    }
}
