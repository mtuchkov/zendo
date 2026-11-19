//! The clock of a tick, a key or a frame: the time of each phase, and a line for the slow-tick log
//! when the whole took longer than the limit of IO-2 (IO-8).
use std::time::{Duration, Instant};

/// A key and a frame take at most this long (IO-2).
pub const LIMIT: Duration = Duration::from_millis(100);

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
