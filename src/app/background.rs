//! The background processes of the agent and of the Claude Code sessions (docs/zendo/background.md,
//! ADR 0016): zendo finds them in its own process tree, shows them, stops one, and asks at quit.
use super::*;
use crate::procs::{self, Proc};
use std::sync::mpsc::{Receiver, channel};

/// A background process that the chat shows.
#[derive(Clone, Debug)]
pub struct Bg {
    pub pgid: u32,
    /// What it runs, as the background line says it (BG-1).
    pub label: String,
    pub started: Instant,
    /// The thread of the agent session that it belongs to.
    pub key: String,
    /// The user stopped it at this time; after `FORCE_AFTER` it ends by force (BG-9).
    pub stopping: Option<Instant>,
}

#[derive(Default)]
pub struct Background {
    pub list: Vec<Bg>,
    scans: Option<Receiver<Vec<Proc>>>,
    /// The quit asked about the background processes at this time (BG-4).
    pub quit_asked: Option<Instant>,
}

impl Background {
    /// The background processes of the thread `key`.
    pub fn of<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a Bg> + 'a {
        self.list.iter().filter(move |b| b.key == key)
    }
}

/// A stopped process that does not end within this time ends by force (BG-9).
const FORCE_AFTER: Duration = Duration::from_secs(5);
/// A process that starts this soon after a step belongs to that step.
const SLACK: Duration = Duration::from_secs(2);

/// The scans of the process tree, off the UI thread (ADR 0009, ADR 0016).
fn scanner() -> Receiver<Vec<Proc>> {
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        while tx.send(procs::list()).is_ok() {
            std::thread::sleep(Duration::from_secs(1));
        }
    });
    rx
}

/// Add `line` to `.zendo/agent.log` of `root`.
fn log_line(root: &std::path::Path, line: &str) {
    use std::io::Write;
    let path = state::dir(root).join("agent.log");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{line}");
    }
}

/// How long a process ran, as a line says it: `40 s`, `14 min`.
pub fn ran(d: Duration) -> String {
    let s = d.as_secs();
    if s < 60 { format!("{s} s") } else { format!("{} min", s / 60) }
}

/// The start of the newest step of `t` that runs commands and started before `at` (with slack);
/// `running` keeps only the steps that have not ended.
fn exec_step(t: &Thread, at: Instant, running: bool) -> Option<Instant> {
    t.items
        .iter()
        .filter_map(|it| if let Item::Tools { uses, .. } = it { Some(uses) } else { None })
        .flatten()
        .filter(|u| u.kind == "execute" && !(running && u.state.ended()))
        .filter_map(|u| u.started)
        .filter(|s| *s <= at + SLACK)
        .max()
}

impl App {
    /// Take the last scan of the process tree; the first call starts the scans.
    pub(crate) fn background_tick(&mut self) {
        let rx = self.bg.scans.get_or_insert_with(scanner);
        if let Some(p) = rx.try_iter().last() {
            self.on_procs(&p);
        }
    }

    /// Scan the process tree now, on this thread.
    pub fn scan_background_now(&mut self) {
        self.on_procs(&procs::list());
    }

    fn on_procs(&mut self, procs: &[Proc]) {
        let terminals: Vec<u32> = (SESSIONS..SESSIONS + crate::session::TERMINALS).filter_map(|n| self.cli.pid(n)).collect();
        let now = Instant::now();
        for g in procs::groups(procs, std::process::id(), &terminals) {
            let started = now.checked_sub(Duration::from_secs(g.secs)).unwrap_or(now);
            let key = self.owner(g.under, started);
            let seen = self.bg.list.iter().any(|b| b.pgid == g.pgid);
            // A command that a step runs now is not in the background yet.
            let step = self.threads.get(&key).and_then(|t| exec_step(t, started, true));
            if !seen && step.is_none() {
                self.bg.list.push(Bg { pgid: g.pgid, label: g.label, started, key, stopping: None });
            }
        }
        let (alive, ended): (Vec<Bg>, Vec<Bg>) = std::mem::take(&mut self.bg.list).into_iter().partition(|b| procs::alive(procs, b.pgid));
        self.bg.list = alive;
        for b in ended {
            self.thread(&b.key).push(Item::Info(format!("{} ended · {}", b.label, ran(b.started.elapsed()))));
        }
        let late: Vec<u32> = self.bg.list.iter().filter(|b| b.stopping.is_some_and(|t| t.elapsed() >= FORCE_AFTER)).map(|b| b.pgid).collect();
        late.into_iter().for_each(|pgid| self.signal(pgid, true));
    }

    /// Send the signal to the group `pgid` and write it into `.zendo/agent.log`. A group that
    /// `procs::may_stop` refuses gets no signal and leaves the list (BG-3, BG-9).
    fn signal(&mut self, pgid: u32, force: bool) {
        let label = self.bg.list.iter().find(|b| b.pgid == pgid).map(|b| b.label.clone()).unwrap_or_default();
        let sent = procs::stop(pgid, force);
        let (sig, what) = (if force { "KILL" } else { "TERM" }, if sent { "sent" } else { "refused" });
        log_line(&self.cfg.root, &format!("zendo: {what} SIG{sig} to the group {pgid} ({label}), zendo {}", std::process::id()));
        if !sent {
            self.bg.list.retain(|b| b.pgid != pgid);
            self.flash(format!("zendo did not stop {label}: its group {pgid} is not a background process"));
        }
    }

    /// The thread of a process: the Claude Code session that it runs under, or the agent session
    /// whose last command step started before it.
    fn owner(&self, under: u32, started: Instant) -> String {
        if let Some(n) = (0..SESSIONS).find(|n| self.cli.pid(*n) == Some(under)) {
            return session_key(n);
        }
        (0..SESSIONS)
            .map(session_key)
            .filter_map(|k| self.threads.get(&k).and_then(|t| exec_step(t, started, false)).map(|s| (k, s)))
            .max_by_key(|(_, s)| *s)
            .map_or_else(|| self.key(), |(k, _)| k)
    }

    /// Stop one background process and the processes that it started (BG-3).
    pub fn stop_background(&mut self, pgid: u32) {
        if let Some(b) = self.bg.list.iter_mut().find(|b| b.pgid == pgid && b.stopping.is_none()) {
            b.stopping = Some(Instant::now());
            self.signal(pgid, false);
        }
    }

    /// The whole command of a background process, in its chat (BG-8).
    pub fn show_background(&mut self, pgid: u32) {
        if let Some(b) = self.bg.list.iter().find(|b| b.pgid == pgid).cloned() {
            self.thread(&b.key).push(Item::Info(format!("runs in the background since {} ago: {}", ran(b.started.elapsed()), b.label)));
        }
    }

    /// True when the quit may go on. While background processes run, the first quit asks; a second
    /// quit or `enter` stops them, `esc` leaves them running (BG-4).
    pub(crate) fn settle_background_quit(&mut self) -> bool {
        if self.bg.list.is_empty() || self.bg.quit_asked.is_some_and(|t| t.elapsed() < crate::session::QUIT_AGAIN) {
            let all: Vec<u32> = self.bg.list.iter().map(|b| b.pgid).collect();
            all.into_iter().for_each(|pgid| self.signal(pgid, false));
            return true;
        }
        self.quit = false;
        self.bg.quit_asked = Some(Instant::now());
        let n = self.bg.list.len();
        let (s, run, them) = if n == 1 { ("", "runs", "it") } else { ("es", "run", "them") };
        self.flash(format!("{n} background process{s} {run}: quit again or enter stops {them}, esc leaves {them} running"));
        false
    }

    /// `enter` or `esc` right after the question of BG-4. True when the key went to the question.
    pub(crate) fn background_quit_key(&mut self, k: &KeyEvent) -> bool {
        if !self.bg.quit_asked.is_some_and(|t| t.elapsed() < crate::session::QUIT_AGAIN) {
            return false;
        }
        match k.code {
            KeyCode::Enter => self.quit = true,
            KeyCode::Esc => {
                self.bg.list.clear();
                self.quit = true;
            }
            _ => {
                self.bg.quit_asked = None;
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
#[path = "background_tests.rs"]
mod tests;
