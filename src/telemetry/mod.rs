//! Telemetry log (docs/zendo/telemetry.md): each action of the keymap becomes one event, a line of
//! JSON, in rolling files in `telemetry/` of the data folder (TELEM-1 to TELEM-4, ADR 0025). The
//! events stay in memory; a thread of their own writes them every 15 seconds and at the exit
//! (TELEM-5, TELEM-8, TELEM-9). The log goes nowhere until the user agrees to share it (TELEM-13).

mod log;

use crate::keys::Pane;
use chrono::{DateTime, SecondsFormat, Utc};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The time between two writes of the log (TELEM-8).
pub const EVERY: Duration = Duration::from_secs(15);

/// How long the exit waits for the last write (TELEM-9).
const LAST_WRITE: Duration = Duration::from_secs(1);

/// The folders of the summaries of earlier versions, which go at the start (TELEM-14).
const OLD: [&str; 2] = ["usage", "responsiveness"];

/// What started an action (USAGE-1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    Key,
    Mouse,
}

impl Trigger {
    fn name(self) -> &'static str {
        match self {
            Trigger::Key => "key",
            Trigger::Mouse => "mouse",
        }
    }
}

/// The event of one action of the keymap (TELEM-1, USAGE-1, RESP-1).
#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    /// The time of the input event of the action.
    pub at: DateTime<Utc>,
    pub pane: Pane,
    pub action: &'static str,
    pub trigger: Trigger,
    /// The time that the user waited for the result; `None` when it did not come (RESP-9).
    pub wait: Option<Duration>,
}

impl Event {
    /// One line of JSON, with nothing that identifies the user (TELEM-6, TELEM-7).
    pub fn line(&self, pid: u32) -> String {
        let mut v = serde_json::json!({
            "at": self.at.to_rfc3339_opts(SecondsFormat::Millis, true),
            "zendo": env!("CARGO_PKG_VERSION"),
            "os": std::env::consts::OS,
            "pid": pid,
            "pane": self.pane.name(),
            "action": self.action,
            "trigger": self.trigger.name(),
        });
        if let Some(w) = self.wait {
            v["wait_ms"] = u64::try_from(w.as_millis()).unwrap_or(u64::MAX).into();
        }
        v.to_string()
    }
}

/// The telemetry of this zendo process: the events that wait in memory, and the thread that writes
/// them.
pub struct Telemetry {
    /// The folder of the log; `None` when the log is off.
    dir: Option<PathBuf>,
    pid: u32,
    waiting: Vec<Event>,
    /// The last time that the events went to the writer.
    sent: Instant,
    writer: Option<log::Writer>,
}

impl Default for Telemetry {
    /// Off: an app records only when its start turns the log on.
    fn default() -> Self {
        Telemetry { dir: None, pid: 0, waiting: Vec::new(), sent: Instant::now(), writer: None }
    }
}

impl Telemetry {
    /// The log of process `pid` in `telemetry/` of the data folder `data`.
    pub fn start(data: &Path, pid: u32) -> Telemetry {
        Telemetry::writing(data, pid, log::append)
    }

    /// As `start`, with the function that appends text to a file.
    pub fn writing(data: &Path, pid: u32, append: impl FnMut(&Path, &str) -> std::io::Result<()> + Send + 'static) -> Telemetry {
        remove_old(data);
        let dir = data.join("telemetry");
        let writer = log::Writer::spawn(log::Roll::new(dir.clone(), pid, Box::new(append)));
        Telemetry { dir: Some(dir), pid, writer: Some(writer), ..Telemetry::default() }
    }

    /// The log of this process in the data folder of zendo; off with `stats = false` (TELEM-12).
    pub fn for_process(on: bool) -> Telemetry {
        let data = crate::setup::config::data_dir();
        if on {
            return Telemetry::start(&data, std::process::id());
        }
        remove_old(&data);
        Telemetry::default()
    }

    /// The folder of the log, when the log is on.
    pub fn dir(&self) -> Option<&Path> {
        self.dir.as_deref()
    }

    /// One more event; nothing when the log is off.
    pub fn record(&mut self, event: Event) {
        if self.writer.is_some() {
            self.waiting.push(event);
        }
    }

    /// The events that wait in memory for the next write.
    pub fn waiting(&self) -> &[Event] {
        &self.waiting
    }

    /// Give the events to the writer when the last write is 15 seconds old, also to write again
    /// what a failed write kept (TELEM-8, TELEM-11).
    pub fn tick(&mut self, now: Instant) {
        let Some(w) = &mut self.writer else { return };
        w.settle(Instant::now());
        if (!self.waiting.is_empty() || w.lost()) && now.saturating_duration_since(self.sent) >= EVERY {
            self.send(now);
        }
    }

    /// At the exit: give the events to the writer, and wait at most one second (TELEM-9).
    pub fn finish(&mut self) {
        let until = Instant::now() + LAST_WRITE;
        let Some(w) = &mut self.writer else { return };
        w.settle(until);
        if !self.waiting.is_empty() || w.lost() {
            self.send(Instant::now());
        }
        if let Some(w) = &mut self.writer {
            w.settle(until);
        }
    }

    fn send(&mut self, now: Instant) {
        self.sent = now;
        let lines = self.waiting.drain(..).map(|e| e.line(self.pid)).collect();
        if let Some(w) = &mut self.writer {
            w.send(lines);
        }
    }
}

/// Delete the folders of the summaries of earlier versions (TELEM-14).
fn remove_old(data: &Path) {
    for f in OLD {
        let _ = std::fs::remove_dir_all(data.join(f));
    }
}

/// The events of the log in the data folder `data`, oldest first; a line that does not parse, such
/// as the last line of a crash, is skipped.
pub fn read(data: &Path) -> Vec<serde_json::Value> {
    let dir = data.join("telemetry");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir).map(|d| d.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "jsonl")).collect()).unwrap_or_default();
    files.sort();
    files.iter().filter_map(|f| std::fs::read_to_string(f).ok()).flat_map(|t| t.lines().filter_map(|l| serde_json::from_str(l).ok()).collect::<Vec<_>>()).collect()
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
