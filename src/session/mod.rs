//! Claude Code sessions in terminals that zendo owns (docs/zendo/sessions.md, ADR 0005).
//!
//! Each of the five agent sessions can have one Claude Code process. zendo runs it in a pseudo
//! terminal (PTY) and keeps its screen in a terminal emulator in memory, so the process runs on
//! while the user is in zendo. The rest of zendo sees only this type: open, write, resize, stop, the
//! screen, and the state.

mod hooks;
mod marks;
pub mod output;
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
/// (docs/zendo/terminal/terminal.md): slot `n` of a terminal is `shell(i)`.
pub struct Sessions {
    /// The command line; `None`: the one of the agent profile.
    pub command: Option<Command>,
    /// The session on the full screen, if the session view is open.
    pub view: Option<usize>,
    /// The terminal in the chat pane, in place of the transcript (TERM-37).
    pub pane: Option<usize>,
    /// When zendo asked whether to stop the sessions on quit.
    pub quit_asked: Option<Instant>,
    /// Wakes the terminal loop when a session prints.
    pub notify: Arc<tokio::sync::Notify>,
    /// The shell of the terminals; `None`: `$SHELL`, else `/bin/sh`.
    pub shell: Option<String>,
    /// The home directory of the shells; `None`: the one of the user.
    pub home: Option<std::path::PathBuf>,
    /// The terminals that the user viewed, the last one first (TERM-24).
    viewed: Vec<usize>,
    slots: Vec<Option<Cli>>,
    /// Text for a shell when its prompt is ready, and whether `enter` runs it.
    pending: Vec<Option<(String, bool)>>,
    /// The size of the screen of a session: rows, columns.
    size: (u16, u16),
    /// The size of a terminal in the chat pane.
    pane_size: (u16, u16),
    /// The size of each slot now.
    sizes: Vec<(u16, u16)>,
    /// For the spinners.
    born: Instant,
}

impl Default for Sessions {
    fn default() -> Self {
        let all = SESSIONS + TERMINALS;
        let size = (23, 80);
        Sessions { command: None, view: None, pane: None, quit_asked: None, notify: Arc::default(), shell: None, home: None, viewed: Vec::new(), slots: (0..all).map(|_| None).collect(), pending: vec![None; all], size, pane_size: size, sizes: vec![size; all], born: Instant::now() }
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
        self.sizes[n] = self.size;
        Ok(())
    }

    /// Input for session `n`: keys, paste. `enter` in a shell starts a command (TERM-27).
    pub fn write(&self, n: usize, bytes: &[u8]) {
        if let Some(c) = self.cli(n) {
            if n >= SESSIONS && bytes.contains(&b'\r') {
                let line = c.with_screen(|s| s.contents_between(s.cursor_position().0, 0, s.cursor_position().0, s.size().1.get()));
                c.with_recorder(|r| r.enter(&line));
            }
            c.write(bytes);
        }
    }

    /// Scroll the screen of session `n` back (`up`) or forward through the lines that left it.
    pub fn scroll(&self, n: usize, up: bool) {
        if let Some(c) = self.cli(n) {
            c.scroll(up);
        }
    }

    /// The terminal in the chat pane, unless it is on the full screen.
    fn in_pane(&self) -> Option<usize> {
        self.pane.filter(|p| self.view != Some(*p))
    }

    /// The size of the screen of each session that is not in the chat pane (SESSIONS-16, TERM-40).
    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.size = (rows.max(2), cols.max(10));
        let pane = self.in_pane();
        (0..self.slots.len()).filter(|n| Some(*n) != pane).for_each(|n| self.fit(n, self.size));
    }

    /// The size of a terminal in the chat pane (TERM-40).
    pub fn resize_pane(&mut self, rows: u16, cols: u16) {
        self.pane_size = (rows.max(2), cols.max(10));
        if let Some(n) = self.in_pane() {
            self.fit(n, self.pane_size);
        }
    }

    /// Give slot `n` the size `size`, if it has another one.
    fn fit(&mut self, n: usize, size: (u16, u16)) {
        if self.sizes[n] == size {
            return;
        }
        self.sizes[n] = size;
        if let Some(c) = self.cli(n) {
            c.resize(size);
        }
    }

    /// Read the states, and take the sessions that ended: their numbers.
    pub fn tick(&mut self) -> Vec<usize> {
        let ended: Vec<usize> = self.slots.iter_mut().enumerate().filter_map(|(n, s)| s.as_mut().is_some_and(Cli::ended).then_some(n)).collect();
        for n in &ended {
            self.slots[*n] = None;
            self.pending[*n] = None;
        }
        self.slots.iter_mut().flatten().for_each(Cli::read_state);
        self.free_shells();
        self.write_pending();
        if self.view.is_some_and(|n| !self.running(n)) {
            self.view = None;
        }
        if self.pane.is_some_and(|n| !self.running(n)) {
            self.pane = None;
        }
        ended
    }

    /// The sessions whose turn or command ended since the last call (ATTN-1).
    pub fn finished(&mut self) -> Vec<usize> {
        self.slots.iter_mut().enumerate().filter_map(|(n, s)| s.as_mut().is_some_and(Cli::finished).then_some(n)).collect()
    }

    /// Stop each session.
    pub fn stop_all(&mut self) {
        self.slots.iter_mut().filter_map(Option::take).for_each(Cli::stop);
        self.view = None;
        self.pane = None;
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
