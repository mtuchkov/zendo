//! The terminals of zendo (docs/zendo/terminal.md): the `F5` terminal and up to four `[>]`
//! terminals, each a shell in a terminal that zendo owns.
use super::*;

/// The terminals: `F5` and four `[>]` (TERM-3).
pub const TERMINALS: usize = 5;

/// The slot of terminal `i`: 0 is `F5`, 1 to 4 are `[>]`.
pub fn shell(i: usize) -> usize {
    SESSIONS + i
}

impl Sessions {
    /// Terminal `i` runs a command.
    pub fn busy(&self, i: usize) -> bool {
        self.cli(shell(i)).is_some_and(Cli::busy)
    }

    /// The terminal for a block: `F5` when it does not run a command, then the first `[>]` that
    /// does not, then a new `[>]`; `None` when all five run a command (TERM-1, TERM-4).
    pub fn free_terminal(&self) -> Option<usize> {
        let running = |i: usize| self.running(shell(i));
        (0..TERMINALS).filter(|i| *i == 0 || running(*i)).find(|i| !self.busy(*i)).or_else(|| (1..TERMINALS).find(|i| !running(*i)))
    }

    /// Start terminal `i` in `root`, if its shell does not run (TERM-12).
    pub fn open_shell(&mut self, i: usize, root: &Path) -> std::io::Result<()> {
        if self.running(shell(i)) {
            return Ok(());
        }
        let program = self.shell.clone().or_else(|| std::env::var("SHELL").ok().filter(|s| !s.is_empty())).unwrap_or_else(|| "/bin/sh".into());
        self.slots[shell(i)] = Some(Cli::spawn_shell(&program, root, self.size, self.notify.clone())?);
        Ok(())
    }

    /// Put `command` into the shell of terminal `i` when its prompt is ready; `run`: and press
    /// `enter` (TERM-1, TERM-21).
    pub fn put(&mut self, i: usize, command: &str, run: bool) {
        self.pending[shell(i)] = Some((command.to_string(), run));
        self.write_pending();
    }

    /// Write the text that waits for a shell whose prompt is ready.
    pub(super) fn write_pending(&mut self) {
        let ready: Vec<usize> = (0..self.slots.len()).filter(|n| self.pending[*n].is_some() && self.cli(*n).is_some_and(Cli::ready)).collect();
        for n in ready {
            let (Some((text, run)), Some(c)) = (self.pending[n].take(), self.slots[n].as_ref()) else { continue };
            let bracketed = c.with_screen(|s| s.bracketed_paste());
            let mut bytes = keys::paste(&crate::text::shell::for_shell(&text, bracketed), bracketed);
            if run {
                bytes.push(b'\r');
            }
            c.write(&bytes);
        }
    }

    /// The text that waits for terminal `i`.
    pub fn pending(&self, i: usize) -> Option<&str> {
        self.pending[shell(i)].as_ref().map(|(t, _)| t.as_str())
    }
}
