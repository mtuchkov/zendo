//! Responsiveness of actions (docs/zendo/responsiveness.md): the time of each action of the keymap,
//! from the read of its input event to the end of the draw of its screen. The app times an action
//! where it counts it (`App::did`, USAGE-1), so the code of an action holds no measurement code
//! (RESP-2, ADR 0024). For each keymap pane and action name zendo keeps the number of runs and a
//! bucket count of their times, in a file of this process in the data folder (RESP-3, RESP-5). A
//! thread of its own writes the file (RESP-4).

use crate::keys::{ACTIONS, Pane};
use chrono::{DateTime, SecondsFormat, Utc};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// The time between two writes of the file, as for the usage counts (USAGE-3).
pub const EVERY: Duration = Duration::from_secs(15);

/// The upper bounds of the buckets, in milliseconds; a longer run goes to the last bucket.
const BOUNDS: [u64; 11] = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];

/// How long the exit waits for the last write.
const LAST_WRITE: Duration = Duration::from_secs(1);

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

    /// `{ runs = 3, upto_8ms = 1, upto_16ms = 2 }`, without the empty buckets (RESP-6).
    fn toml(&self) -> String {
        let name = |i: usize| BOUNDS.get(i).map_or(format!("over_{}ms", BOUNDS[BOUNDS.len() - 1]), |b| format!("upto_{b}ms"));
        let parts: Vec<String> = self.buckets.iter().enumerate().filter(|(_, n)| **n > 0).map(|(i, n)| format!("{} = {n}", name(i))).collect();
        format!("{{ runs = {}, {} }}", self.runs(), parts.join(", "))
    }
}

/// The function that writes the text to the file.
type Write = Box<dyn FnMut(&Path, &str) -> std::io::Result<()> + Send>;

/// The thread that writes the file, and the answers that it sends after each write.
struct Writer {
    texts: mpsc::Sender<String>,
    done: mpsc::Receiver<()>,
    /// The writes that the thread has not answered yet.
    open: usize,
}

impl Writer {
    fn spawn(file: PathBuf, mut write: Write) -> Writer {
        let (texts, rx) = mpsc::channel::<String>();
        let (done_tx, done) = mpsc::channel();
        std::thread::spawn(move || {
            for text in rx {
                // RESP-7: a write that fails says nothing; the next write has all the runs.
                let _ = write(&file, &text);
                let _ = done_tx.send(());
            }
        });
        Writer { texts, done, open: 0 }
    }

    fn send(&mut self, text: String) {
        if self.texts.send(text).is_ok() {
            self.open += 1;
        }
    }

    /// Wait for the open writes, at most until `until`.
    fn settle(&mut self, until: Instant) {
        while self.open > 0 && self.done.recv_timeout(until.saturating_duration_since(Instant::now())).is_ok() {
            self.open -= 1;
        }
    }
}

/// Replace the file whole, through a temporary file and a rename.
fn write_file(file: &Path, text: &str) -> std::io::Result<()> {
    let tmp = file.with_extension("toml.tmp");
    file.parent().map_or(Ok(()), std::fs::create_dir_all)?;
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, file)
}

/// The times of the actions of this zendo process and their file.
pub struct Responsiveness {
    times: BTreeMap<(Pane, &'static str), Times>,
    /// The file; `None` when the measurements are off.
    file: Option<PathBuf>,
    writer: Option<Writer>,
    started: DateTime<Utc>,
    /// The read of the input event that the next actions answer.
    input: Option<Instant>,
    /// The actions since that input, which wait for the draw.
    waiting: Vec<(Pane, &'static str, Instant)>,
    changed: bool,
    written: Instant,
}

impl Default for Responsiveness {
    /// Off: an app measures only when its start turns the measurements on.
    fn default() -> Self {
        Responsiveness { times: BTreeMap::new(), file: None, writer: None, started: Utc::now(), input: None, waiting: Vec::new(), changed: false, written: Instant::now() }
    }
}

impl Responsiveness {
    /// The measurements of process `pid`, for `responsiveness/<date of the start>-<pid>.toml` in
    /// the data folder `dir`.
    pub fn start(dir: &Path, pid: u32) -> Responsiveness {
        Responsiveness::writing(dir, pid, write_file)
    }

    /// As `start`, with the function that writes the file.
    pub fn writing(dir: &Path, pid: u32, write: impl FnMut(&Path, &str) -> std::io::Result<()> + Send + 'static) -> Responsiveness {
        let name = format!("{}-{pid}.toml", chrono::Local::now().format("%Y-%m-%d"));
        let file = dir.join("responsiveness").join(name);
        Responsiveness { writer: Some(Writer::spawn(file.clone(), Box::new(write))), file: Some(file), ..Responsiveness::default() }
    }

    /// The measurements of this process in the data folder of zendo; off with `usage = false`.
    pub fn for_process(on: bool) -> Responsiveness {
        if on { Responsiveness::start(&crate::setup::config::data_dir(), std::process::id()) } else { Responsiveness::default() }
    }

    pub fn file(&self) -> Option<&Path> {
        self.file.as_deref()
    }

    /// The loop read an input event at `at`.
    pub fn input(&mut self, at: Instant) {
        self.input = Some(at);
    }

    /// The action `name` of `pane` ran for the last input. Only an action of the keymap counts
    /// (RESP-1).
    pub fn ran(&mut self, pane: Pane, name: &str) {
        let Some(at) = self.input.filter(|_| self.file.is_some()) else { return };
        let Some(act) = ACTIONS.iter().find(|a| a.pane == pane && a.name == name) else { return };
        self.waiting.push((pane, act.name, at));
    }

    /// The draw of the screen ended at `at`: the waiting actions get their times (RESP-1).
    pub fn drawn(&mut self, at: Instant) {
        self.input = None;
        for (pane, name, from) in std::mem::take(&mut self.waiting) {
            self.times.entry((pane, name)).or_default().add(at.saturating_duration_since(from));
            self.changed = true;
        }
    }

    pub fn times(&self, pane: Pane, name: &str) -> Option<Times> {
        self.times.iter().find(|((p, n), _)| *p == pane && *n == name).map(|(_, t)| *t)
    }

    /// Send the measurements to the writer when they changed and the last write is 15 seconds old.
    pub fn tick(&mut self, now: Instant) {
        if self.changed && now.saturating_duration_since(self.written) >= EVERY {
            self.send(now);
        }
    }

    /// At the exit: send the measurements when they changed, and wait a short time for the writes.
    pub fn finish(&mut self) {
        if self.changed {
            self.send(Instant::now());
        }
        if let Some(w) = &mut self.writer {
            w.settle(Instant::now() + LAST_WRITE);
        }
    }

    fn send(&mut self, now: Instant) {
        self.written = now;
        self.changed = false;
        let text = self.text(Utc::now());
        if let Some(w) = &mut self.writer {
            w.send(text);
        }
    }

    /// The file: the head, then a table of times for each pane (RESP-5, RESP-6).
    pub fn text(&self, written: DateTime<Utc>) -> String {
        let time = |t: DateTime<Utc>| t.to_rfc3339_opts(SecondsFormat::Secs, true);
        let head = format!("zendo = \"{}\"\nos = \"{}\"\nstarted = \"{}\"\nwritten = \"{}\"\n", env!("CARGO_PKG_VERSION"), std::env::consts::OS, time(self.started), time(written));
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
