//! The terminals of zendo (docs/zendo/terminal.md): `F5` puts the shell block under the cursor of
//! the transcript into a terminal, `alt-F5` also runs it, and `F5` elsewhere shows the `F5`
//! terminal.
use super::*;
use crate::session::{TERMINALS, shell};

impl App {
    /// `F5` (`run`: `alt-F5`): the shell block under the cursor of the transcript goes to the first
    /// terminal that does not run a command; without a block, the `F5` terminal shows (TERM-1,
    /// TERM-4, TERM-11, TERM-21).
    pub(crate) fn terminal_key(&mut self, run: bool) {
        let on_block = self.panel && self.focus == Focus::Claude && self.transcript.on;
        match self.transcript.block().filter(|_| on_block).map(str::to_string) {
            Some(command) => self.put_into_terminal(&command, run),
            None => self.open_terminal(0),
        }
    }

    /// Put `command` into a terminal that does not run a command, and show it.
    pub(crate) fn put_into_terminal(&mut self, command: &str, run: bool) {
        let Some(i) = self.cli.free_terminal() else {
            return self.flash(format!("all {TERMINALS} terminals run a command: wait for one, or stop one"));
        };
        self.open_terminal(i);
        if self.cli.running(shell(i)) {
            self.cli.put(i, command, run);
        }
    }

    /// Show terminal `i`; start its shell first if it does not run (TERM-12).
    pub(crate) fn open_terminal(&mut self, i: usize) {
        match self.cli.open_shell(i, &self.cfg.root) {
            Ok(()) => self.cli.view = Some(shell(i)),
            Err(e) => self.flash(format!("could not start the shell: {e}")),
        }
    }
}
