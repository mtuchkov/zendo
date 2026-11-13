//! The terminals of zendo (docs/zendo/terminal/shell-blocks.md): `F5` puts the shell block under the cursor of
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
    pub fn put_into_terminal(&mut self, command: &str, run: bool) {
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
        self.cli.touch(shell(i));
    }

    /// `alt-o` in the input of a chat: the result of the terminal viewed last goes into the input;
    /// `alt-o` again takes the next terminal instead (TERM-24, TERM-28, TERM-31).
    pub(crate) fn insert_output(&mut self) {
        let order = self.cli.results();
        let pick = self.output_pick.take();
        if order.is_empty() {
            return self.flash("no terminal has the result of a command yet");
        }
        let next = match pick {
            Some((i, lines, row, col)) => {
                (self.input.lines, self.input.row, self.input.col) = (lines, row, col);
                (i + 1) % order.len()
            }
            None => 0,
        };
        self.output_pick = Some((next, self.input.lines.clone(), self.input.row, self.input.col));
        let Some(o) = self.cli.result(order[next]) else { return };
        let name = crate::ui::cli::name(self, order[next]);
        self.input.type_str(&output_text(&name, &o));
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
            Some("next_session") => self.step_in_chat(true),
            Some("prev_session") => self.step_in_chat(false),
            Some(a @ ("terminal" | "terminal_run")) => self.terminal_key(a == "terminal_run"),
            Some(a) if a.starts_with("session_") => self.switch_session(a[8..].parse::<usize>().map_or(0, |m| m - 1)),
            _ => {
                let app_cursor = self.cli.with_screen(n, |s| s.application_cursor()).unwrap_or(false);
                self.cli.write(n, &crate::session::keys::encode(&k, app_cursor));
            }
        }
    }

    /// `alt-]` and `alt-[`: the next or the previous item for the chat pane, in a ring: the agent
    /// sessions, the `F5` terminal, the `[>]` terminals that run (TERM-41, TERM-49).
    pub(crate) fn step_in_chat(&mut self, forward: bool) {
        let all: Vec<usize> = (0..SESSIONS + TERMINALS).filter(|n| *n <= SESSIONS || self.cli.running(*n)).collect();
        let now = self.cli.pane.unwrap_or(self.state.session);
        let step = if forward { 1 } else { all.len() - 1 };
        let next = all.iter().position(|n| *n == now).map_or(0, |i| all[(i + step) % all.len()]);
        match next.checked_sub(SESSIONS) {
            Some(i) => self.open_terminal(i),
            None => self.switch_session(next),
        }
    }
}

/// A result as text for the input: the terminal, the command, the exit code, the output
/// (TERM-27, TERM-30, TERM-32, TERM-33).
fn output_text(name: &str, o: &crate::session::output::Output) -> String {
    let state = match (o.done, o.exit) {
        (false, _) => "runs, not complete: the output so far".to_string(),
        (true, Some(code)) => format!("exit {code}"),
        (true, None) => "exit code unknown".to_string(),
    };
    let dropped = if o.dropped > 0 { format!("\n({} earlier lines dropped)", o.dropped) } else { String::new() };
    format!("Terminal {name}: `{}` · {state}{dropped}\n```text\n{}\n```\n", o.command, o.text)
}
