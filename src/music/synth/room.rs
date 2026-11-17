//! The room of the synth (ADR 0006): a stereo reverb of the Freeverb kind, eight damped combs and
//! four all-passes in each channel. It stands in for the 2.6 s convolution of the first
//! implementation, which costs too much in the audio thread (LOFI-4).

/// The delays of the combs and the all-passes of Freeverb, in ms.
const COMBS: [f32; 8] = [25.31, 26.94, 28.96, 30.75, 32.24, 33.81, 35.31, 36.67];
const ALLPASSES: [f32; 4] = [12.61, 10.0, 7.73, 5.1];
/// The right channel is longer by this many ms, so that the channels differ.
const SPREAD: f32 = 0.52;
/// The time in which the room falls by 60 dB, near the tail of the noise impulse.
const DECAY: f32 = 1.8;
/// The damping of the high frequencies in the combs.
const DAMP: f32 = 0.2;
/// The power gain of the convolution in Web Audio for white noise: its impulse is normalized.
const POWER: f32 = 0.18;

/// A delay line; its buffer is allocated once, in `new`.
struct Line {
    buf: Vec<f32>,
    pos: usize,
}

impl Line {
    fn new(ms: f32, rate: f32) -> Self {
        let len = ((ms * rate / 1000.0) as usize).max(1);
        Line { buf: vec![0.0; len], pos: 0 }
    }

    /// The oldest sample out, `x` in.
    fn swap(&mut self, x: impl FnOnce(f32) -> f32) -> f32 {
        let out = self.buf[self.pos];
        self.buf[self.pos] = x(out);
        self.pos = if self.pos + 1 == self.buf.len() { 0 } else { self.pos + 1 };
        out
    }

    fn clear(&mut self) {
        self.buf.fill(0.0);
    }
}

struct Comb {
    line: Line,
    feedback: f32,
    store: f32,
}

impl Comb {
    fn tick(&mut self, x: f32) -> f32 {
        let (store, feedback) = (&mut self.store, self.feedback);
        self.line.swap(|out| {
            *store = out * (1.0 - DAMP) + *store * DAMP;
            x + *store * feedback
        })
    }
}

/// One channel of the room.
struct Channel {
    combs: [Comb; 8],
    allpasses: [Line; 4],
}

impl Channel {
    fn new(rate: f32, spread: f32) -> Self {
        let comb = |ms: f32| {
            let ms = ms + spread;
            Comb { line: Line::new(ms, rate), feedback: 10f32.powf(-3.0 * ms / 1000.0 / DECAY), store: 0.0 }
        };
        Channel {
            combs: COMBS.map(comb),
            allpasses: ALLPASSES.map(|ms| Line::new(ms + spread, rate)),
        }
    }

    fn tick(&mut self, x: f32) -> f32 {
        let wet: f32 = self.combs.iter_mut().map(|c| c.tick(x)).sum();
        self.allpasses.iter_mut().fold(wet, |y, line| line.swap(|b| y + 0.5 * b) - y)
    }

    /// The power gain of the combs for white noise, without the damping.
    fn power(&self) -> f32 {
        self.combs.iter().map(|c| 1.0 / (1.0 - c.feedback * c.feedback)).sum()
    }

    fn clear(&mut self) {
        self.combs.iter_mut().for_each(|c| {
            c.line.clear();
            c.store = 0.0;
        });
        self.allpasses.iter_mut().for_each(Line::clear);
    }
}

/// The stereo room: a mono send in, a stereo wet signal out.
pub(crate) struct Room {
    left: Channel,
    right: Channel,
    gain: f32,
}

impl Room {
    pub(crate) fn new(rate: f32) -> Self {
        let left = Channel::new(rate, 0.0);
        let gain = (POWER / left.power()).sqrt();
        Room { left, right: Channel::new(rate, SPREAD), gain }
    }

    pub(crate) fn tick(&mut self, send: f32) -> (f32, f32) {
        let x = send * self.gain;
        (self.left.tick(x), self.right.tick(x))
    }

    pub(crate) fn clear(&mut self) {
        self.left.clear();
        self.right.clear();
    }
}
