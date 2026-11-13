//! `ctrl-o` while the chat answers (docs/zendo/handoff.md): the session view waits for the end of
//! the turn, `ctrl-o` again stops the turn, and then Claude Code starts. One writer for a thread:
//! Claude Code starts only after the agent ends its turn.
use super::*;
use crate::keys::Pane;

/// The time for the second `ctrl-o`, and the time that a stopped turn has to end (HANDOFF-4,
/// HANDOFF-6).
pub const HOLD_AGAIN: Duration = Duration::from_secs(5);

/// The first prompt of Claude Code after a stop (HANDOFF-5).
pub const CONTINUE: &str = "The last turn was stopped. Continue it.";

/// The session view that waits for the end of a turn.
#[derive(Clone, Copy, Debug)]
pub struct Hold {
    /// The agent session.
    pub n: usize,
    /// The last `ctrl-o`.
    pub pressed: Instant,
    /// When the user stopped the turn.
    pub stopped: Option<Instant>,
}

impl App {
    /// `ctrl-o` while the chat of agent session `n` answers: the session view waits (HANDOFF-1,
    /// HANDOFF-2).
    pub(crate) fn hold_cli(&mut self, n: usize) {
        self.hold = Some(Hold { n, pressed: Instant::now(), stopped: None });
    }

    /// A key in the waiting view: `ctrl-o`, the leave key, the keys of the sessions and the
    /// terminal; no other key goes anywhere (HANDOFF-4, HANDOFF-9, HANDOFF-10).
    pub(crate) fn key_hold(&mut self, k: KeyEvent) {
        if self.keys.local(Pane::Chat, &k) == Some("handoff") {
            return self.hold_again();
        }
        if self.keys.local(Pane::Session, &k) == Some("leave") {
            self.hold = None;
            return;
        }
        let global = self.keys.local(Pane::Global, &k);
        if global.is_some_and(|a| a == "next_session" || a == "terminal" || a.starts_with("session_")) {
            self.hold = None;
            self.on_key(k);
        }
    }

    /// `ctrl-o` in the waiting view: a second one within 5 s stops the turn; a later one counts as
    /// the first (HANDOFF-4).
    fn hold_again(&mut self) {
        let Some(h) = self.hold.as_mut().filter(|h| h.stopped.is_none()) else { return };
        if h.pressed.elapsed() > HOLD_AGAIN {
            h.pressed = Instant::now();
            return;
        }
        h.stopped = Some(Instant::now());
        if let Some(a) = &self.agent {
            a.send(Cmd::Cancel { key: session_key(h.n) });
        }
    }

    /// The waiting view is on the thread `key`.
    pub(crate) fn holds(&self, key: &str) -> bool {
        self.hold.is_some_and(|h| session_key(h.n) == key)
    }

    /// The turn ended: the queued questions go to the chat input, and Claude Code starts, with the
    /// prompt to continue after a stop (HANDOFF-3, HANDOFF-5, HANDOFF-11).
    pub(crate) fn hold_turn_ended(&mut self, key: &str) {
        let Some(h) = self.hold.take() else { return };
        self.queue_to_input(key);
        self.start_cli(h.n, h.stopped.map(|_| CONTINUE));
    }

    /// Each tick: a stopped turn that did not end within 5 s; Claude Code starts anyway (HANDOFF-6).
    pub(crate) fn hold_tick(&mut self) {
        let Some(h) = self.hold.filter(|h| h.stopped.is_some_and(|t| t.elapsed() > HOLD_AGAIN)) else { return };
        self.hold_turn_ended(&session_key(h.n));
        self.flash("the turn did not stop within 5 s: Claude Code started anyway");
    }

    /// The text of the bottom line of the waiting view (HANDOFF-7, HANDOFF-8).
    pub fn hold_line(&self) -> String {
        let again = self.keys.first(Pane::Chat, "handoff");
        match self.hold.and_then(|h| h.stopped) {
            Some(_) => "stopping the turn… Claude Code starts when it ends".into(),
            None => format!("Claude Code starts when the turn ends · {again} again: stop the turn and continue in Claude Code"),
        }
    }
}
