//! Claude Code sessions in terminals that zendo owns (docs/zendo/sessions.md, ADR 0005).
//!
//! Each of the five agent sessions can have one Claude Code process. zendo runs it in a pseudo
//! terminal (PTY) and keeps its screen in a terminal emulator in memory, so the process runs on
//! while the user is in zendo. The rest of zendo sees only this type: open, write, resize, stop, the
//! screen, and the state.

mod hooks;
pub mod keys;
mod pty;
mod shells;

pub use pty::Cli;
pub use shells::{TERMINALS, shell};

use crate::app::SESSIONS;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// The command line of the agent: the program and the arguments before the session ID.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
}

/// The state of a running Claude Code session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Works,
    /// Waits for an answer or a permission of the user.
    Waits,
    /// The turn ended.
    Idle,
}

impl State {
    /// The mark on the session button.
    pub fn mark(self) -> char {
        match self {
            State::Works => '•',
            State::Waits => '!',
            State::Idle => '·',
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            State::Works => "works",
            State::Waits => "waits",
            State::Idle => "idle",
        }
    }
}

/// The time to quit again after the question (SESSIONS-19).
pub const QUIT_AGAIN: Duration = Duration::from_secs(5);

/// The Claude Code sessions of the five agent sessions, then the shells of the five terminals
/// (docs/zendo/terminal.md): slot `n` of a terminal is `shell(i)`.
pub struct Sessions {
    /// The command line; `None`: the one of the agent profile.
    pub command: Option<Command>,
    /// The session on screen, if the session view is open.
    pub view: Option<usize>,
    /// When zendo asked whether to stop the sessions on quit.
    pub quit_asked: Option<Instant>,
    /// Wakes the terminal loop when a session prints.
    pub notify: Arc<tokio::sync::Notify>,
    /// The shell of the terminals; `None`: `$SHELL`, else `/bin/sh`.
    pub shell: Option<String>,
    slots: Vec<Option<Cli>>,
    /// Text for a shell when its prompt is ready, and whether `enter` runs it.
    pending: Vec<Option<(String, bool)>>,
    /// The size of the screen of a session: rows, columns.
    size: (u16, u16),
    /// For the spinners.
    born: Instant,
}

impl Default for Sessions {
    fn default() -> Self {
        let all = SESSIONS + TERMINALS;
        Sessions { command: None, view: None, quit_asked: None, notify: Arc::default(), shell: None, slots: (0..all).map(|_| None).collect(), pending: vec![None; all], size: (23, 80), born: Instant::now() }
    }
}

impl Sessions {
    fn cli(&self, n: usize) -> Option<&Cli> {
        self.slots.get(n).and_then(Option::as_ref)
    }

    /// The time since zendo started, for the spinners.
    pub fn since(&self) -> Duration {
        self.born.elapsed()
    }

    pub fn running(&self, n: usize) -> bool {
        self.cli(n).is_some()
    }

    /// A Claude Code session runs, or a terminal runs a command: quit asks first (SESSIONS-6, TERM-18).
    pub fn any_running(&self) -> bool {
        self.count() > 0
    }

    /// The Claude Code sessions that run, and the terminals that run a command.
    pub fn count(&self) -> usize {
        self.slots.iter().enumerate().filter(|(n, s)| s.as_ref().is_some_and(|c| *n < SESSIONS || c.busy())).count()
    }

    pub fn state(&self, n: usize) -> Option<State> {
        self.cli(n).map(Cli::state)
    }

    pub fn pid(&self, n: usize) -> Option<u32> {
        self.cli(n).and_then(Cli::pid)
    }

    /// The text on the screen of session `n`.
    pub fn text(&self, n: usize) -> String {
        self.cli(n).map(|c| c.with_screen(|s| s.contents())).unwrap_or_default()
    }

    /// Run `f` with the screen of session `n`.
    pub fn with_screen<T>(&self, n: usize, f: impl FnOnce(&vt100::Screen) -> T) -> Option<T> {
        self.cli(n).map(|c| c.with_screen(f))
    }

    /// Start the Claude Code session of agent session `n` with its session ID, if it does not run.
    pub fn open(&mut self, n: usize, cmd: &Command, id: &str, root: &Path) -> std::io::Result<()> {
        if self.running(n) {
            return Ok(());
        }
        let dir = root.join(".zendo").join("sessions");
        let cli = Cli::spawn(cmd, id, root, &dir, n, self.size, self.notify.clone())?;
        self.slots[n] = Some(cli);
        Ok(())
    }

    /// Input for session `n`: keys, paste.
    pub fn write(&self, n: usize, bytes: &[u8]) {
        if let Some(c) = self.cli(n) {
            c.write(bytes);
        }
    }

    /// Scroll the screen of session `n` back (`up`) or forward through the lines that left it.
    pub fn scroll(&self, n: usize, up: bool) {
        if let Some(c) = self.cli(n) {
            c.scroll(up);
        }
    }

    /// The size of the screen of each session (SESSIONS-16).
    pub fn resize(&mut self, rows: u16, cols: u16) {
        let size = (rows.max(2), cols.max(10));
        if size == self.size {
            return;
        }
        self.size = size;
        self.slots.iter().flatten().for_each(|c| c.resize(size));
    }

    /// Read the states, and take the sessions that ended: their numbers.
    pub fn tick(&mut self) -> Vec<usize> {
        let ended: Vec<usize> = self.slots.iter_mut().enumerate().filter_map(|(n, s)| s.as_mut().is_some_and(Cli::ended).then_some(n)).collect();
        for n in &ended {
            self.slots[*n] = None;
            self.pending[*n] = None;
        }
        self.slots.iter_mut().flatten().for_each(Cli::read_state);
        self.write_pending();
        if self.view.is_some_and(|n| !self.running(n)) {
            self.view = None;
        }
        ended
    }

    /// Stop each session.
    pub fn stop_all(&mut self) {
        self.slots.iter_mut().filter_map(Option::take).for_each(Cli::stop);
        self.view = None;
    }
}

impl Drop for Sessions {
    fn drop(&mut self) {
        self.stop_all();
    }
}

#[cfg(test)]
#[path = "keys_tests.rs"]
mod keys_tests;
