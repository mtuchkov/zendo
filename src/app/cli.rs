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
            // A terminal on the full screen leaves the chat pane too (TERM-22, TERM-42).
            if self.cli.pane == Some(n) {
                self.cli.pane = None;
            }
            self.cli.view = None;
            return;
        }
        if self.keys.local(Pane::Global, &k) == Some("next_session") {
            self.cli.view = None;
            return self.next_in_chat();
        }
        if self.keys.local(Pane::Global, &k) == Some("terminal") {
            return self.open_terminal(0);
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

    /// The states, and the sessions that ended: zendo goes back, and opens the agent session again,
    /// so the chat shows what happened in Claude Code (SESSIONS-12, SESSIONS-13).
    pub(crate) fn cli_tick(&mut self) {
        for n in self.cli.tick().into_iter().filter(|n| *n < SESSIONS) {
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

    /// `enter` in the chat of an agent session whose Claude Code session runs: the question goes to
    /// Claude Code as typed, and Claude Code queues it while it works (SESSIONS-4). A file chip goes
    /// as `@path`, a selection chip as its text after the question.
    pub(crate) fn send_to_cli(&mut self) {
        let n = self.active();
        let q = self.input.take();
        let chips: Vec<Chip> = std::mem::take(&mut self.chips).into_iter().filter(|c| q.contains(c.label.as_str())).collect();
        let mut text = q.clone();
        let mut after = Vec::new();
        for c in &chips {
            let with = match (&c.attach, &c.context) {
                (Some(a), _) => format!("@{}", a.path().strip_prefix(&self.cfg.root).unwrap_or(a.path()).display()),
                (None, Some(ctx)) => {
                    after.push(ctx.clone());
                    String::new()
                }
                _ => String::new(),
            };
            text = text.replacen(&c.label, &with, 1);
        }
        let text = std::iter::once(text.trim().to_string()).chain(after).filter(|s| !s.is_empty()).collect::<Vec<_>>().join("\n\n");
        if text.is_empty() {
            return;
        }
        let bracketed = self.cli.with_screen(n, |s| s.bracketed_paste()).unwrap_or(false);
        // Without bracketed paste a new line would send the question: alt-enter is a new line.
        let body = if bracketed { text } else { text.replace('\n', "\x1b\r") };
        self.cli.write(n, &keys::paste(&body, bracketed));
        self.cli.write(n, b"\r");
        let key = self.key();
        let t = self.thread(&key);
        t.push(Item::User(q));
        t.push(Item::Info("sent to Claude Code; it queues the message while it works".into()));
        self.panel_scroll = 0;
    }

    /// Other questions of the chat of an agent session whose Claude Code session runs, such as a
    /// split or a question about the delta, do not go (SESSIONS-4). True when it says so.
    pub(crate) fn cli_blocks(&mut self) -> bool {
        if !self.cli.running(self.active()) {
            return false;
        }
        let k = self.keys.first(Pane::Chat, "handoff");
        let key = self.key();
        self.thread(&key).push(Item::Info(format!("this agent session is open in Claude Code: {k} shows it")));
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
        self.flash(format!("{n} Claude Code session{s} or terminal command{s} {run}: quit again to stop {them}"));
        false
    }
}
