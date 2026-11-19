//! The start of a Claude Code session or of the shell of a terminal, on a task: it writes the hooks
//! and starts a program, and an endpoint product can make a start slow (ADR 0023, IO-1). The slot
//! gets the session when the task answers; the view stays open while it starts.
use super::*;
use crate::worker::Task;

/// A start takes at most this long.
const LIMIT: Duration = Duration::from_secs(30);

/// A start: its slot, the size that its program starts with, and the task.
type Start = (usize, (u16, u16), Task<std::io::Result<Cli>>);

/// The starts that wait, by slot.
#[derive(Default)]
pub(super) struct Starts {
    tasks: Vec<Start>,
    /// The starts that failed, with why, for the app to say once.
    failed: Vec<(usize, String)>,
    /// Input for a program that starts: it goes when the program runs (IO-6).
    early: std::sync::Mutex<std::collections::HashMap<usize, Vec<u8>>>,
}

impl Sessions {
    /// Start the Claude Code session of agent session `n` with its session ID, if it does not run
    /// or start; returns at once (IO-1).
    pub fn open(&mut self, n: usize, cmd: &Command, resume: Resume, root: &Path) -> std::io::Result<()> {
        if self.running(n) {
            return Ok(());
        }
        let (cmd, id, prompt) = (cmd.clone(), resume.id.to_string(), resume.prompt.map(str::to_string));
        let (root, size, notify) = (root.to_path_buf(), self.size, self.notify.clone());
        self.start_slot(n, size, move || {
            let dir = root.join(".zendo").join("sessions");
            Cli::spawn(&cmd, Resume { id: &id, prompt: prompt.as_deref() }, &root, &dir, n, size, notify)
        });
        Ok(())
    }

    /// Start terminal `i` in `root`, if its shell does not run or start (TERM-12); returns at once.
    pub fn open_shell(&mut self, i: usize, root: &Path) -> std::io::Result<()> {
        if self.running(shell(i)) {
            return Ok(());
        }
        let program = self.shell.clone().or_else(|| std::env::var("SHELL").ok().filter(|s| !s.is_empty())).unwrap_or_else(|| "/bin/sh".into());
        let (root, home, size, notify) = (root.to_path_buf(), self.home.clone(), self.pane_size, self.notify.clone());
        self.start_slot(shell(i), size, move || Cli::spawn_shell(&program, &root, home.as_deref(), size, notify));
        Ok(())
    }

    fn start_slot(&mut self, n: usize, size: (u16, u16), start: impl FnOnce() -> std::io::Result<Cli> + Send + 'static) {
        let mut task = Task::new(LIMIT);
        task.start(start);
        self.starts.tasks.push((n, size, task));
        self.sizes[n] = size;
    }

    /// Slot `n` starts its program now.
    pub fn starting(&self, n: usize) -> bool {
        self.starts.tasks.iter().any(|(s, _, t)| *s == n && t.running())
    }

    /// A start waits for its program.
    pub fn starting_any(&self) -> bool {
        self.starts.tasks.iter().any(|(_, _, t)| t.running())
    }

    /// The starts that answered: their slots get the sessions; a failure waits for `failed`.
    pub fn poll_starts(&mut self) {
        let mut waiting = Vec::new();
        for (n, size, mut t) in std::mem::take(&mut self.starts.tasks) {
            match (t.overdue(), t.poll()) {
                (true, _) => self.starts.failed.push((n, "the program did not start in 30 s".into())),
                (_, Some(Ok(cli))) => {
                    // The screen changed size while the program started.
                    if self.sizes[n] != size {
                        cli.resize(self.sizes[n]);
                    }
                    if let Some(bytes) = self.starts.early.lock().ok().and_then(|mut e| e.remove(&n)) {
                        cli.write(&bytes);
                    }
                    self.slots[n] = Some(cli);
                }
                (_, Some(Err(e))) => self.starts.failed.push((n, e.to_string())),
                (_, None) => waiting.push((n, size, t)),
            }
        }
        self.starts.tasks = waiting;
    }

    /// Keep `bytes` for slot `n`, whose program starts: they go when it runs.
    pub(super) fn write_early(&self, n: usize, bytes: &[u8]) {
        if let Ok(mut e) = self.starts.early.lock() {
            e.entry(n).or_default().extend_from_slice(bytes);
        }
    }

    /// The starts that failed since the last call, each once.
    pub fn failed(&mut self) -> Vec<(usize, String)> {
        std::mem::take(&mut self.starts.failed)
    }
}
