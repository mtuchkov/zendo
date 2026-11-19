//! The clock of a tick, a key or a frame: the time of each phase, and a line for the slow-tick log
//! when the whole took longer than the limit of IO-2 (IO-8).
use std::time::{Duration, Instant};

/// A key and a frame take at most this long (IO-2).
pub const LIMIT: Duration = Duration::from_millis(100);
/// After this long without an answer, zendo says that it reads (IO-6).
pub const READING_AFTER: Duration = Duration::from_millis(350);

/// "reading" with dots that move each 250 ms, after `READING_AFTER`; None before (IO-6).
pub fn reading(waited: Duration) -> Option<String> {
    let after = waited.checked_sub(READING_AFTER)?;
    let dots = (after.as_millis() / 250 % 3) as usize + 1;
    Some(format!("reading{:<3}", ".".repeat(dots)))
}

pub struct Clock {
    what: &'static str,
    start: Instant,
    phases: Vec<(&'static str, Duration)>,
}

impl Clock {
    /// Start the clock of `what`: "tick", "key", "draw".
    pub fn start(what: &'static str) -> Clock {
        Clock { what, start: Instant::now(), phases: Vec::new() }
    }

    /// Run `f` as the phase `name`, and keep its time.
    pub fn time<R>(&mut self, name: &'static str, f: impl FnOnce() -> R) -> R {
        let t = Instant::now();
        let r = f();
        self.phases.push((name, t.elapsed()));
        r
    }

    /// A line for the log, when the whole took longer than the limit: the time, the whole, and the
    /// slowest phase.
    pub fn report(&self) -> Option<String> {
        let all = self.start.elapsed();
        if all <= LIMIT {
            return None;
        }
        let (name, took) = self.phases.iter().max_by_key(|(_, d)| *d).copied().unwrap_or(("-", all));
        let at = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f");
        Some(format!("{at} {} {} ms, slowest {name} {} ms\n", self.what, all.as_millis(), took.as_millis()))
    }
}

#[cfg(test)]
#[path = "clock_tests.rs"]
mod tests;
