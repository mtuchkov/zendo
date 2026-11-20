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
    /// does not run. While the chat answers, the session view waits for the end of the turn
    /// (SESSIONS-7, HANDOFF-1).
    pub(crate) fn open_cli(&mut self) {
        if self.handoff_cli().is_none() {
            return self.flash(format!("{} has no command line with a resume", self.agent_label()));
        }
        let (n, key) = (self.active(), self.key());
        if self.cli.running(n) {
            self.cli.view = Some(n);
            return;
        }
        if self.threads.get(&key).is_some_and(|t| t.busy.is_some()) {
            return self.hold_cli(n);
        }
        self.start_cli(n, None);
    }

    /// Start the Claude Code session of agent session `n` and show it; `prompt`: its first prompt
    /// (HANDOFF-5).
    pub(crate) fn start_cli(&mut self, n: usize, prompt: Option<&str>) {
        let Some(cmd) = self.handoff_cli() else { return };
        let key = session_key(n);
        let id = self.threads.get(&key).and_then(|t| t.session.clone()).or_else(|| self.state.session(&self.cfg.threads_key(), &key));
        let Some(id) = id else {
            return self.flash("no thread yet: ask something first");
        };
        match self.cli.open(n, &cmd, crate::session::Resume { id: &id, prompt }, &self.cfg.root) {
            Ok(()) => self.cli.view = Some(n),
            Err(e) => self.flash(format!("could not start {}: {e}", cmd.program)),
        }
    }

    /// A key in the session view: leave, another session, or a key for Claude Code (SESSIONS-9 to
    /// SESSIONS-11).
    pub(crate) fn key_cli(&mut self, n: usize, k: KeyEvent) {
        if self.keys.local(Pane::Session, &k) == Some("leave") {
            self.did(Pane::Session, "leave");
            // A terminal on the full screen leaves the chat pane too (TERM-22, TERM-42).
            if self.cli.pane == Some(n) {
                self.cli.pane = None;
            }
            self.cli.view = None;
            return;
        }
        let global = self.keys.local(Pane::Global, &k).filter(|a| *a != "terminal_run" && count::session_action(a));
        self.did_some(Pane::Global, global);
        if let Some(a @ ("next_session" | "prev_session")) = global {
            return self.step_full_screen(a == "next_session");
        }
        if global == Some("terminal") {
            return self.next_terminal();
        }
        let session = global.and_then(|a| a.strip_prefix("session_")).and_then(|s| s.parse::<usize>().ok());
        if let Some(m) = session {
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
        self.cli_messages();
        for n in self.cli.tick().into_iter().filter(|n| *n < SESSIONS) {
            let key = session_key(n);
            self.link.send(Cmd::Forget { key: key.clone() });
            self.threads.remove(&key);
            if self.threads.shown() && self.active() == n {
                self.open_session();
            }
            self.flash(format!("Claude Code session {} ended", n + 1));
        }
    }

    /// The prompts, the steps and the answers of the Claude Code sessions go to the chats of their
    /// agent sessions; a question that the chat sent shows once (CLIMSG-1, CLIMSG-4, CLIMSG-8).
    fn cli_messages(&mut self) {
        use crate::session::{Message, plain};
        for (n, m) in self.cli.messages() {
            if let Message::Session { id, clear } = m {
                self.claude_session_started(n, id, clear);
                continue;
            }
            let t = self.thread(&session_key(n));
            match m {
                Message::Prompt(p) => match t.sent_to_cli.iter().position(|s| *s == plain(&p)) {
                    Some(i) => drop(t.sent_to_cli.remove(i)),
                    None => t.push(Item::User(plain(&p))),
                },
                Message::Answer(a) => {
                    t.push(Item::Agent(a));
                    self.board_turn_ended(n);
                }
                Message::Text(a) => t.push(Item::Agent(a)),
                // The tool calls between two texts are one item, as the tools of the agent (CLIMSG-9).
                Message::Tool { id, title, kind } => match t.items.last_mut() {
                    Some(Item::Tools { uses, .. }) => uses.push(cli_tool(id, title, kind)),
                    _ => t.push(Item::Tools { uses: vec![cli_tool(id, title, kind)], started: Instant::now(), took: None }),
                },
                Message::Session { .. } => {}
            }
        }
    }

    /// `enter` in the chat of an agent session whose Claude Code session runs: the question goes to
    /// Claude Code as typed, and Claude Code queues it while it works (SESSIONS-4). Each chip goes
    /// at its place: a text as `⟦` its text `⟧`, a file as `@path`, the file of a red chip as its
    /// label (CLAUDE-57, CLAUDE-59, CLAUDE-60).
    pub(crate) fn send_to_cli(&mut self) {
        let n = self.active();
        let (q, chips) = self.composer_mut().take_message();
        let root = self.cfg.root.clone();
        let cli_chip = |c: &Chip| match (&c.context, &c.attach) {
            (Some(ctx), _) => format!("{}{ctx}{}", chips::OPEN, chips::CLOSE),
            (None, Some(a)) if a.path().exists() => format!("@{}", a.path().strip_prefix(&root).unwrap_or(a.path()).display()),
            _ => c.label.clone(),
        };
        let text = chips::inline(&q, &chips, cli_chip).trim().to_string();
        let shown = chips::inline(&q, &chips, crate::app::composer::shown_chip);
        if text.is_empty() {
            return;
        }
        let bracketed = self.cli.with_screen(n, |s| s.bracketed_paste()).unwrap_or(false);
        // Without bracketed paste a new line would send the question: alt-enter is a new line.
        let sent = crate::session::plain(&text);
        let body = if bracketed { text } else { text.replace('\n', "\x1b\r") };
        self.cli.write(n, &keys::paste(&body, bracketed));
        self.cli.write(n, b"\r");
        self.board_user_asked();
        let key = self.key();
        let t = self.thread(&key);
        t.sent_to_cli.push(sent);
        t.push(Item::User(shown));
        t.push(Item::Info("sent to Claude Code; it queues the message while it works".into()));
        self.threads.to_end();
        self.save_drafts();
    }

    /// Other questions of the chat of an agent session whose Claude Code session runs, such as a
    /// split or a question about the delta, do not go (SESSIONS-4). True when it says so.
    pub(crate) fn cli_blocks(&mut self) -> bool {
        if !self.cli.running(self.active()) {
            return false;
        }
        let shows = self.keys.key(Pane::Chat, "handoff").map(|k| format!(": {k} shows it")).unwrap_or_default();
        let key = self.key();
        self.thread(&key).push(Item::Info(format!("this agent session is open in Claude Code{shows}")));
        true
    }

    /// zendo quits: the state keeps the place in the last document of the session (FILES-54).
    fn quit_now(&mut self) -> bool {
        self.track_recent();
        self.save_drafts();
        true
    }

    /// True when zendo can quit now. While Claude Code sessions run, the first quit asks, and a
    /// second one within 5 s stops them (SESSIONS-6, SESSIONS-19).
    pub fn settle_quit(&mut self) -> bool {
        if !self.quit || !self.settle_background_quit() {
            return false;
        }
        if !self.cli.any_running() {
            return self.quit_now();
        }
        if self.cli.quit_asked.is_some_and(|t| t.elapsed() < QUIT_AGAIN) {
            self.cli.stop_all();
            return self.quit_now();
        }
        self.quit = false;
        self.cli.quit_asked = Some(Instant::now());
        let n = self.cli.count();
        let (s, run, them) = if n == 1 { ("", "runs", "it") } else { ("s", "run", "them") };
        self.flash(format!("{n} Claude Code session{s} or terminal command{s} {run}: quit again to stop {them}"));
        false
    }
}

/// A tool call of a Claude Code session: its transcript tells no state, so it counts as done.
fn cli_tool(id: String, title: String, kind: String) -> ToolUse {
    ToolUse { id, title, kind, state: crate::agent::ToolState::Done, ..Default::default() }
}
