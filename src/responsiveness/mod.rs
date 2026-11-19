//! Responsiveness of actions (docs/zendo/responsiveness.md): the time that the user waits for the
//! result of each action of the keymap, from the read of its input event to the end of the draw of
//! the screen that shows the result. An action that starts work in the background, a search or a
//! question to the agent, waits for the end of that work (RESP-1). The app times an action
//! where it counts it (`App::did`, USAGE-1), so the code of an action holds no measurement code
//! (RESP-2, ADR 0024). For each keymap pane and action name zendo keeps the number of runs and a
//! bucket count of their times, in a file of this process in the data folder (RESP-3, RESP-5). A
//! thread of its own writes the file (RESP-4).

use crate::keys::{ACTIONS, Pane};
use crate::stats::{Flush, LAST_WRITE};
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The time between two writes of the file, as for the usage counts (USAGE-3).
pub use crate::stats::EVERY;

/// The upper bounds of the buckets, in milliseconds: up to 512 ms for the screen, then seconds
/// for the wait for work in the background. A longer run goes to the last bucket.
const BOUNDS: [u64; 20] = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1_000, 2_000, 4_000, 8_000, 16_000, 32_000, 64_000, 128_000, 256_000, 512_000];

/// The longest wait for a result: after it, the action goes into the last bucket.
const LONGEST: Duration = Duration::from_millis(BOUNDS[BOUNDS.len() - 1]);

/// One piece of work in the background that the user waits for (RESP-1). Each piece has its own
/// identity: a new search or the next question is other work.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Work {
    /// The walk of the files of the open find panel.
    Walk,
    /// The search of query `n` of the find panel.
    Search(u64),
    /// The answer of the agent to question `n` (`Outgoing::n`), in its turn or in the queue. The
    /// number is unique in the process, so a rename of the thread does not change the work.
    Answer(usize),
}

/// The times of one action in one pane: the runs in each bucket (RESP-5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Times {
    buckets: [u64; BOUNDS.len() + 1],
}

impl Times {
    fn add(&mut self, took: Duration) {
        let ms = took.as_millis();
        let i = BOUNDS.iter().position(|b| ms <= u128::from(*b)).unwrap_or(BOUNDS.len());
        self.buckets[i] += 1;
    }

    /// The number of runs.
    pub fn runs(&self) -> u64 {
        self.buckets.iter().sum()
    }

    /// The runs that took at most `ms` milliseconds; `ms` is one of the bounds of the buckets.
    pub fn within(&self, ms: u64) -> u64 {
        BOUNDS.iter().zip(self.buckets).take_while(|(b, _)| **b <= ms).map(|(_, n)| n).sum()
    }

    /// `{ runs = 3, upto_8ms = 1, upto_2s = 2 }`, without the empty buckets (RESP-6).
    fn toml(&self) -> String {
        let span = |b: u64| if b < 1_000 { format!("{b}ms") } else { format!("{}s", b / 1_000) };
        let name = |i: usize| BOUNDS.get(i).map_or(format!("over_{}", span(LONGEST.as_millis() as u64)), |b| format!("upto_{}", span(*b)));
        let parts: Vec<String> = self.buckets.iter().enumerate().filter(|(_, n)| **n > 0).map(|(i, n)| format!("{} = {n}", name(i))).collect();
        format!("{{ runs = {}, {} }}", self.runs(), parts.join(", "))
    }
}

/// An action that waits for the end of the work that it started.
struct Held {
    pane: Pane,
    name: &'static str,
    from: Instant,
    works: BTreeSet<Work>,
}

/// The times of the actions of this zendo process and their file.
pub struct Responsiveness {
    times: BTreeMap<(Pane, &'static str), Times>,
    /// The file; `None` when the measurements are off.
    file: Option<PathBuf>,
    flush: Option<Flush>,
    started: DateTime<Utc>,
    /// The read of the input event that the next actions answer.
    input: Option<Instant>,
    /// The actions of that input.
    waiting: Vec<(Pane, &'static str, Instant)>,
    /// The actions of earlier inputs, which wait for the draw and for the end of their work.
    held: Vec<Held>,
    /// The work that ran when that input came or at the last draw.
    seen: BTreeSet<Work>,
}

impl Default for Responsiveness {
    /// Off: an app measures only when its start turns the measurements on.
    fn default() -> Self {
        Responsiveness { times: BTreeMap::new(), file: None, flush: None, started: Utc::now(), input: None, waiting: Vec::new(), held: Vec::new(), seen: BTreeSet::new() }
    }
}

impl Responsiveness {
    /// The measurements of process `pid`, for `responsiveness/<date of the start>-<pid>.toml` in
    /// the data folder `dir`.
    pub fn start(dir: &Path, pid: u32) -> Responsiveness {
        Responsiveness::writing(dir, pid, crate::stats::write)
    }

    /// As `start`, with the function that writes the file.
    pub fn writing(dir: &Path, pid: u32, write: impl FnMut(&Path, &str) -> std::io::Result<()> + Send + 'static) -> Responsiveness {
        let file = crate::stats::file(dir, "responsiveness", pid);
        Responsiveness { flush: Some(Flush::spawn(file.clone(), Box::new(write))), file: Some(file), ..Responsiveness::default() }
    }

    /// The measurements of this process in the data folder of zendo; off with `stats = false`
    /// (RESP-8).
    pub fn for_process(on: bool) -> Responsiveness {
        if on { Responsiveness::start(&crate::setup::config::data_dir(), std::process::id()) } else { Responsiveness::default() }
    }

    pub fn file(&self) -> Option<&Path> {
        self.file.as_deref()
    }

    /// The input thread read an input event at `at`; the work `now` runs before the loop handles
    /// it. The actions of the input before wait for the work that started since it.
    pub fn input(&mut self, at: Instant, now: &BTreeSet<Work>) {
        if self.file.is_some() {
            self.close(now);
            self.input = Some(at);
        }
    }

    /// The actions of the last input wait for the work that started since that input (RESP-1).
    fn close(&mut self, now: &BTreeSet<Work>) {
        let started: BTreeSet<Work> = now.difference(&self.seen).cloned().collect();
        let mine = std::mem::take(&mut self.waiting).into_iter().map(|(pane, name, from)| Held { pane, name, from, works: started.clone() });
        self.held.extend(mine);
        self.seen = now.clone();
    }

    /// The action `name` of `pane` ran for the last input. Only an action of the keymap counts
    /// (RESP-1).
    pub fn ran(&mut self, pane: Pane, name: &str) {
        let Some(at) = self.input.filter(|_| self.file.is_some()) else { return };
        let Some(act) = ACTIONS.iter().find(|a| a.pane == pane && a.name == name) else { return };
        self.waiting.push((pane, act.name, at));
    }

    /// The draw of the screen ended at `at`, while the work `now` runs in the background. An action
    /// gets its time at the first draw after the end of its work, or when it waited longer than
    /// the last bucket (RESP-1). An action whose result has not come at the exit gets no time.
    pub fn drawn(&mut self, at: Instant, now: &BTreeSet<Work>) {
        if self.file.is_none() {
            return;
        }
        self.close(now);
        self.input = None;
        let (done, held): (Vec<Held>, Vec<Held>) = std::mem::take(&mut self.held).into_iter().partition(|h| h.works.is_disjoint(now) || at.saturating_duration_since(h.from) > LONGEST);
        if let Some(f) = self.flush.as_mut().filter(|_| !done.is_empty()) {
            f.change();
        }
        for h in done {
            self.times.entry((h.pane, h.name)).or_default().add(at.saturating_duration_since(h.from));
        }
        self.held = held;
    }

    pub fn times(&self, pane: Pane, name: &str) -> Option<Times> {
        self.times.iter().find(|((p, n), _)| *p == pane && *n == name).map(|(_, t)| *t)
    }

    /// Send the measurements to the writer when they changed and the last write is 15 seconds old,
    /// or when the last write failed (RESP-7).
    pub fn tick(&mut self, now: Instant) {
        if self.flush.as_mut().is_some_and(|f| f.due(now)) {
            self.send(now);
        }
    }

    /// At the exit: send the measurements when they changed or when the last write failed, and
    /// wait a short time for the writes.
    pub fn finish(&mut self) {
        let until = Instant::now() + LAST_WRITE;
        if self.flush.as_mut().is_some_and(|f| f.pending(until)) {
            self.send(Instant::now());
        }
        if let Some(f) = &mut self.flush {
            f.pending(until);
        }
    }

    fn send(&mut self, now: Instant) {
        let text = self.text(Utc::now());
        if let Some(f) = &mut self.flush {
            f.send(now, text);
        }
    }

    /// The file: the head, then a table of times for each pane (RESP-5, RESP-6).
    pub fn text(&self, written: DateTime<Utc>) -> String {
        let head = crate::stats::head(self.started, written);
        let table = |pane: Pane| {
            let rows: String = self.times.iter().filter(|((p, _), _)| *p == pane).map(|((_, name), t)| format!("{name} = {}\n", t.toml())).collect();
            Some(rows).filter(|r| !r.is_empty()).map(|r| format!("\n[{}]\n{r}", pane.name()))
        };
        head + &Pane::ALL.into_iter().filter_map(table).collect::<String>()
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
