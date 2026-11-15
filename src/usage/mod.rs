//! Usage counts (docs/zendo/usage.md): how often the user does each action of the keymap, by
//! keymap pane, action name and trigger. The counts go to a file of this process in the data
//! folder, every 15 seconds while they change and at the exit. zendo never reads them (USAGE-6).

use crate::keys::{ACTIONS, Pane};
use chrono::{DateTime, SecondsFormat, Utc};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The time between two writes of the usage file (USAGE-3).
pub const EVERY: Duration = Duration::from_secs(15);

/// What started an action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trigger {
    Key,
    Mouse,
}

/// The counts of one action in one pane, for each trigger.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Count {
    key: u64,
    mouse: u64,
}

impl Count {
    fn of(&mut self, t: Trigger) -> &mut u64 {
        match t {
            Trigger::Key => &mut self.key,
            Trigger::Mouse => &mut self.mouse,
        }
    }

    /// `{ key = 12, mouse = 3 }`, without the triggers that have no count.
    fn toml(&self) -> String {
        let parts: Vec<String> = [("key", self.key), ("mouse", self.mouse)].into_iter().filter(|(_, n)| *n > 0).map(|(k, n)| format!("{k} = {n}")).collect();
        format!("{{ {} }}", parts.join(", "))
    }
}

/// The counts of this zendo process and their file.
pub struct Usage {
    counts: BTreeMap<(Pane, &'static str), Count>,
    /// The usage file; `None` when the counting is off (USAGE-14).
    file: Option<PathBuf>,
    started: DateTime<Utc>,
    /// The counts changed since the last write.
    changed: bool,
    /// The last write, or the last try.
    written: Instant,
}

impl Default for Usage {
    /// Off: an app counts only when its start turns the counting on.
    fn default() -> Self {
        Usage { counts: BTreeMap::new(), file: None, started: Utc::now(), changed: false, written: Instant::now() }
    }
}

impl Usage {
    /// The counts of process `pid`, for `usage/<date of the start>-<pid>.toml` in the data folder
    /// `dir` (USAGE-4, USAGE-10).
    pub fn start(dir: &Path, pid: u32) -> Usage {
        let name = format!("{}-{pid}.toml", chrono::Local::now().format("%Y-%m-%d"));
        Usage { file: Some(dir.join("usage").join(name)), ..Usage::default() }
    }

    /// The counts of this process in the data folder of zendo; off with `usage = false` in the
    /// config (USAGE-14).
    pub fn for_process(on: bool) -> Usage {
        if on { Usage::start(&crate::setup::config::data_dir(), std::process::id()) } else { Usage::default() }
    }

    /// The usage file, when the counting is on.
    pub fn file(&self) -> Option<&Path> {
        self.file.as_deref()
    }

    /// One more run of the action `name` of `pane`. Only an action of the keymap counts (USAGE-1,
    /// USAGE-8).
    pub fn count(&mut self, pane: Pane, name: &str, trigger: Trigger) {
        if self.file.is_none() {
            return;
        }
        let Some(act) = ACTIONS.iter().find(|a| a.pane == pane && a.name == name) else { return };
        *self.counts.entry((pane, act.name)).or_default().of(trigger) += 1;
        self.changed = true;
    }

    pub fn get(&self, pane: Pane, name: &str, trigger: Trigger) -> u64 {
        self.counts.get(&(pane, name)).map_or(0, |c| *c.clone().of(trigger))
    }

    /// Write the counts when they changed and the last write is 15 seconds old (USAGE-3).
    pub fn tick(&mut self, now: Instant) {
        if self.changed && now.saturating_duration_since(self.written) >= EVERY {
            self.write_at(now);
        }
    }

    /// Write the counts now, when they changed: at the exit (USAGE-12).
    pub fn write(&mut self) {
        if self.changed {
            self.write_at(Instant::now());
        }
    }

    /// Replace the file whole, through a temporary file and a rename (USAGE-13). A write that fails
    /// says nothing; the counts stay for the next write (USAGE-5).
    fn write_at(&mut self, now: Instant) {
        self.written = now;
        let Some(file) = &self.file else { return };
        let tmp = file.with_extension("toml.tmp");
        let done = file.parent().map_or(Ok(()), std::fs::create_dir_all).and_then(|_| std::fs::write(&tmp, self.text(Utc::now()))).and_then(|_| std::fs::rename(&tmp, file));
        self.changed = done.is_err();
    }

    /// The file: the head, then a table of counts for each pane (USAGE-9, USAGE-11).
    pub fn text(&self, written: DateTime<Utc>) -> String {
        let time = |t: DateTime<Utc>| t.to_rfc3339_opts(SecondsFormat::Secs, true);
        let head = format!("zendo = \"{}\"\nos = \"{}\"\nstarted = \"{}\"\nwritten = \"{}\"\n", env!("CARGO_PKG_VERSION"), std::env::consts::OS, time(self.started), time(written));
        let table = |pane: Pane| {
            let rows: String = self.counts.iter().filter(|((p, _), _)| *p == pane).map(|((_, name), c)| format!("{name} = {}\n", c.toml())).collect();
            Some(rows).filter(|r| !r.is_empty()).map(|r| format!("\n[{}]\n{r}", pane.name()))
        };
        head + &Pane::ALL.into_iter().filter_map(table).collect::<String>()
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
