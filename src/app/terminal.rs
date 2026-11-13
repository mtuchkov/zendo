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

    /// Show terminal `i` in the chat pane; start its shell first if it does not run (TERM-12,
    /// TERM-37, TERM-42). Offline there is no chat pane: the terminal takes the full screen.
    pub(crate) fn open_terminal(&mut self, i: usize) {
        if let Err(e) = self.cli.open_shell(i, &self.cfg.root) {
            return self.flash(format!("could not start the shell: {e}"));
        }
        self.cli.view = None;
        if self.cfg.offline {
            self.cli.view = Some(shell(i));
            return;
        }
        if self.mode != Mode::Write {
            self.mode = Mode::Read;
        }
        (self.panel, self.focus) = (true, Focus::Claude);
        self.cli.pane = Some(shell(i));
    }

    /// The terminal in the chat pane has the keys: the chat pane is on the screen and has the focus.
    pub(crate) fn pane_terminal(&self) -> Option<usize> {
        let shown = self.panel && self.focus == Focus::Claude && matches!(self.mode, Mode::Read | Mode::Write);
        self.cli.pane.filter(|_| shown && self.cli.view.is_none())
    }

    /// A key for the terminal in the chat pane: leave, the full screen, another session or
    /// terminal, or a key for the shell (TERM-13, TERM-22, TERM-38).
    pub(crate) fn key_pane_terminal(&mut self, n: usize, k: KeyEvent) {
        use crate::keys::Pane;
        if self.keys.local(Pane::Session, &k) == Some("leave") {
            self.cli.pane = None;
            return;
        }
        if self.keys.local(Pane::Chat, &k) == Some("handoff") {
            self.cli.view = Some(n);
            return;
        }
        match self.keys.local(Pane::Global, &k) {
            Some("next_session") => self.next_in_chat(),
            Some(a @ ("terminal" | "terminal_run")) => self.terminal_key(a == "terminal_run"),
            Some(a) if a.starts_with("session_") => self.switch_session(a[8..].parse::<usize>().map_or(0, |m| m - 1)),
            _ => {
                let app_cursor = self.cli.with_screen(n, |s| s.application_cursor()).unwrap_or(false);
                self.cli.write(n, &crate::session::keys::encode(&k, app_cursor));
            }
        }
    }

    /// `alt-n`: the next item for the chat pane: the agent sessions, the `F5` terminal, the `[>]`
    /// terminals that run, then the first agent session again (TERM-41).
    pub(crate) fn next_in_chat(&mut self) {
        let all: Vec<usize> = (0..SESSIONS + TERMINALS).filter(|n| *n <= SESSIONS || self.cli.running(*n)).collect();
        let now = self.cli.pane.unwrap_or(self.state.session);
        let next = all.iter().position(|n| *n == now).map_or(0, |i| all[(i + 1) % all.len()]);
        match next.checked_sub(SESSIONS) {
            Some(i) => self.open_terminal(i),
            None => self.switch_session(next),
        }
    }
}
