//! The Claude Code sessions of the agent sessions (docs/zendo/sessions.md): open them, the input of
//! the session view, the end of a session, and the question on quit.
use super::*;
use crate::keys::Pane;
use crate::session::{QUIT_AGAIN, keys};

impl App {
    /// The agent session of the chat.
    fn active(&self) -> usize {
        self.state.session.min(SESSIONS - 1)
    }

    /// `ctrl-o`: show the Claude Code session of the active agent session; start it first if it
    /// does not run (SESSIONS-7, SESSIONS-15).
    pub(crate) fn open_cli(&mut self) {
        let Some(cmd) = self.handoff_cli() else {
            return self.flash(format!("{} has no command line with a resume", self.agent_label()));
        };
        let (n, key) = (self.active(), self.key());
        if self.cli.running(n) {
            self.cli.view = Some(n);
            return;
        }
        if self.threads.get(&key).is_some_and(|t| t.busy.is_some()) {
            return self.flash("the chat answers: wait for the answer or press esc, then open Claude Code");
        }
        let id = self.threads.get(&key).and_then(|t| t.session.clone()).or_else(|| self.state.session(&self.cfg.agent, &key));
        let Some(id) = id else {
            return self.flash("no thread yet: ask something first");
        };
        match self.cli.open(n, &cmd, &id, &self.cfg.root) {
            Ok(()) => self.cli.view = Some(n),
            Err(e) => self.flash(format!("could not start {}: {e}", cmd.program)),
        }
    }

    /// A key in the session view: leave, another session, or a key for Claude Code (SESSIONS-9 to
    /// SESSIONS-11).
    pub(crate) fn key_cli(&mut self, n: usize, k: KeyEvent) {
        if self.keys.local(Pane::Session, &k) == Some("leave") {
            self.cli.view = None;
            return;
        }
        let session = self.keys.local(Pane::Global, &k).and_then(|a| a.strip_prefix("session_")).and_then(|s| s.parse::<usize>().ok());
        if let Some(m) = session.and_then(|m| m.checked_sub(1)) {
            self.cli.view = self.cli.running(m).then_some(m);
            return self.switch_session(m);
        }
        let app_cursor = self.cli.with_screen(n, |s| s.application_cursor()).unwrap_or(false);
        self.cli.write(n, &keys::encode(&k, app_cursor));
    }

    /// Paste in the session view.
    pub(crate) fn paste_cli(&mut self, n: usize, s: &str) {
        let bracketed = self.cli.with_screen(n, |s| s.bracketed_paste()).unwrap_or(false);
        self.cli.write(n, &keys::paste(s, bracketed));
    }

    /// The wheel in the session view scrolls through the lines that left the screen.
    pub(crate) fn mouse_cli(&mut self, n: usize, m: MouseEvent) {
        match m.kind {
            MouseEventKind::ScrollUp => self.cli.scroll(n, true),
            MouseEventKind::ScrollDown => self.cli.scroll(n, false),
            _ => {}
        }
    }

    /// The states, and the sessions that ended: zendo goes back, and opens the agent session again,
    /// so the chat shows what happened in Claude Code (SESSIONS-12, SESSIONS-13).
    pub(crate) fn cli_tick(&mut self) {
        for n in self.cli.tick() {
            let key = session_key(n);
            if let Some(a) = &self.agent {
                a.send(Cmd::Forget { key: key.clone() });
            }
            self.threads.remove(&key);
            if self.panel && self.active() == n {
                self.open_session();
            }
            self.flash(format!("Claude Code session {} ended", n + 1));
        }
    }

    /// The chat of an agent session whose Claude Code session runs sends no question (SESSIONS-4).
    /// True when it says so.
    pub(crate) fn cli_blocks(&mut self) -> bool {
        if !self.cli.running(self.active()) {
            return false;
        }
        let k = self.keys.first(Pane::Chat, "handoff");
        let key = self.key();
        self.thread(&key).items.push(Item::Info(format!("this agent session is open in Claude Code: {k} shows it")));
        true
    }

    /// True when zendo can quit now. While Claude Code sessions run, the first quit asks, and a
    /// second one within 5 s stops them (SESSIONS-6, SESSIONS-19).
    pub fn settle_quit(&mut self) -> bool {
        if !self.quit {
            return false;
        }
        if !self.cli.any_running() {
            return true;
        }
        if self.cli.quit_asked.is_some_and(|t| t.elapsed() < QUIT_AGAIN) {
            self.cli.stop_all();
            return true;
        }
        self.quit = false;
        self.cli.quit_asked = Some(Instant::now());
        let n = self.cli.count();
        let (s, run, them) = if n == 1 { ("", "runs", "it") } else { ("s", "run", "them") };
        self.flash(format!("{n} Claude Code session{s} {run}: quit again to stop {them}"));
        false
    }
}
