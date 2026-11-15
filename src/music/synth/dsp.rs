//! The small parts of the synth (ADR 0006): a random generator, oscillator shapes, an envelope of
//! exponential ramps, and a state-variable filter. Nothing here allocates (LOFI-4).

use std::f32::consts::{PI, TAU};

/// The level where an exponential ramp starts and ends, as `0.0001` in the first implementation.
pub(crate) const FLOOR: f32 = 1e-4;
/// The resonance of a low-pass or a high-pass filter: near the default of Web Audio.
pub(crate) const Q_SOFT: f32 = 0.75;

/// The frequency of a MIDI note, in Hz.
pub(crate) fn mtof(note: f32) -> f32 {
    440.0 * ((note - 69.0) / 12.0).exp2()
}

/// A time in seconds as a count of samples.
pub(crate) fn samples(secs: f32, rate: f32) -> u32 {
    (secs * rate).max(0.0) as u32
}

/// The ratio of frequencies of an offset in cents.
pub(crate) fn cents(c: f32) -> f32 {
    (c / 1200.0).exp2()
}

/// A xorshift generator, seeded through splitmix64 so that near seeds give unrelated music.
pub(crate) struct Rng(u64);

impl Rng {
    pub(crate) fn new(seed: u64) -> Self {
        let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        Rng((z ^ (z >> 31)) | 1)
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// A number in 0..1.
    pub(crate) fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u32 << 24) as f32
    }

    /// A number in -1..1: white noise.
    pub(crate) fn noise(&mut self) -> f32 {
        self.unit() * 2.0 - 1.0
    }

    /// True with the probability `p`.
    pub(crate) fn chance(&mut self, p: f32) -> bool {
        self.unit() < p
    }

    /// A number in `lo..hi`.
    pub(crate) fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.unit()
    }
}

/// The phase of an oscillator, in 0..1.
#[derive(Clone, Copy, Default)]
pub(crate) struct Osc {
    phase: f32,
    inc: f32,
}

impl Osc {
    pub(crate) fn new(freq: f32, rate: f32) -> Self {
        Osc { phase: 0.0, inc: (freq / rate).min(0.5) }
    }

    pub(crate) fn set(&mut self, freq: f32, rate: f32) {
        self.inc = (freq / rate).min(0.5);
    }

    /// The phase of this sample; the oscillator moves on by one sample.
    pub(crate) fn step(&mut self) -> f32 {
        let p = self.phase;
        self.phase += self.inc;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        p
    }
}

pub(crate) fn sine(phase: f32) -> f32 {
    (phase * TAU).sin()
}

/// A triangle that starts at zero and rises, as the triangle of Web Audio.
pub(crate) fn triangle(phase: f32) -> f32 {
    1.0 - 4.0 * ((phase + 0.25).fract() - 0.5).abs()
}

/// An envelope of three exponential ramps (target, length in samples) after a wait, like
/// `exponentialRampToValueAtTime` in the first implementation. It starts and ends at [`FLOOR`].
#[derive(Clone, Copy, Default)]
pub(crate) struct Env {
    value: f32,
    mult: f32,
    left: u32,
    wait: u32,
    next: usize,
    ramps: [(f32, u32); 3],
}

impl Env {
    /// Wait `wait` samples, then follow `ramps` from [`FLOOR`].
    pub(crate) fn start(&mut self, wait: u32, ramps: [(f32, u32); 3]) {
        *self = Env { value: FLOOR, mult: 1.0, left: 0, wait, next: 0, ramps };
    }

    /// A hit: the level `peak` at once, then a ramp down to [`FLOOR`] in `decay` samples.
    pub(crate) fn hit(&mut self, wait: u32, peak: f32, decay: u32) {
        self.start(wait, [(peak, 1), (peak, 1), (FLOOR, decay)]);
    }

    /// The envelope still sounds, or waits to sound.
    pub(crate) fn active(&self) -> bool {
        self.wait > 0 || self.left > 0 || self.next < self.ramps.len()
    }

    /// The level for the choice of a voice to reuse: a waiting voice is not free.
    pub(crate) fn busy(&self) -> f32 {
        match (self.active(), self.wait > 0) {
            (false, _) => 0.0,
            (true, true) => f32::MAX,
            (true, false) => self.value,
        }
    }

    /// The level of this sample; 0 while the envelope waits and after its end.
    pub(crate) fn tick(&mut self) -> f32 {
        if self.wait > 0 {
            self.wait -= 1;
            return 0.0;
        }
        if self.left == 0 {
            let Some(&(target, len)) = self.ramps.get(self.next) else {
                self.value = 0.0;
                return 0.0;
            };
            self.next += 1;
            self.left = len.max(1);
            self.mult = (target / self.value.max(FLOOR)).powf(1.0 / self.left as f32);
        }
        self.left -= 1;
        self.value *= self.mult;
        self.value
    }
}

/// The outputs of a [`Svf`] for one sample.
pub(crate) struct Bands {
    pub(crate) low: f32,
    /// Band-pass with a gain of 1 at its centre, as the band-pass of Web Audio.
    pub(crate) band: f32,
    pub(crate) high: f32,
}

/// A state-variable filter (the trapezoidal form of Simper): stable while its frequency moves.
#[derive(Clone, Copy, Default)]
pub(crate) struct Svf {
    a1: f32,
    a2: f32,
    a3: f32,
    k: f32,
    s1: f32,
    s2: f32,
}

impl Svf {
    pub(crate) fn new(freq: f32, q: f32, rate: f32) -> Self {
        let mut f = Svf::default();
        f.tune(freq, q, rate);
        f
    }

    /// A new frequency and resonance; below the Nyquist frequency at any rate.
    pub(crate) fn tune(&mut self, freq: f32, q: f32, rate: f32) {
        let g = (PI * freq.min(0.45 * rate) / rate).tan();
        self.k = 1.0 / q;
        self.a1 = 1.0 / (1.0 + g * (g + self.k));
        self.a2 = g * self.a1;
        self.a3 = g * self.a2;
    }

    pub(crate) fn tick(&mut self, x: f32) -> Bands {
        let v3 = x - self.s2;
        let v1 = self.a1 * self.s1 + self.a2 * v3;
        let v2 = self.s2 + self.a2 * self.s1 + self.a3 * v3;
        self.s1 = 2.0 * v1 - self.s1;
        self.s2 = 2.0 * v2 - self.s2;
        Bands { low: v2, band: self.k * v1, high: x - self.k * v1 - v2 }
    }

    /// Forget the past samples.
    pub(crate) fn clear(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
    }
}

/// A voice of the synth: the voices of one part share a [`Pool`].
pub(crate) trait Voice: Copy + Default {
    fn tick(&mut self, rng: &mut Rng) -> f32;
    /// 0 when the voice is free; a pool reuses the voice with the lowest level.
    fn busy(&self) -> f32;
}

/// The voices of one part, allocated once, so that a note never allocates (LOFI-4).
pub(crate) struct Pool<V, const N: usize>([V; N]);

impl<V: Voice, const N: usize> Pool<V, N> {
    pub(crate) fn new() -> Self {
        Pool([V::default(); N])
    }

    /// Start a note on a free voice, or on the quietest one.
    pub(crate) fn play(&mut self, start: impl FnOnce(&mut V)) {
        let quietest = self.0.iter_mut().reduce(|a, b| if b.busy() < a.busy() { b } else { a });
        if let Some(v) = quietest {
            start(v);
        }
    }

    pub(crate) fn tick(&mut self, rng: &mut Rng) -> f32 {
        self.0.iter_mut().map(|v| v.tick(rng)).sum()
    }

    pub(crate) fn clear(&mut self) {
        self.0 = [V::default(); N];
    }
}
