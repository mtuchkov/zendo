//! Usage counts: what the dispatch of a key or of the mouse tells the counts (docs/zendo/usage.md).
use super::*;
use crate::keys::Pane;
use crate::usage::Trigger;

/// The global actions that choose what the chat pane shows: an agent session or a terminal.
pub(crate) fn session_action(act: &str) -> bool {
    matches!(act, "next_session" | "prev_session" | "terminal" | "terminal_run") || act.starts_with("session_")
}

impl App {
    /// A key ran the action `act` of the keymap pane `pane` (USAGE-1, USAGE-7).
    pub(crate) fn did(&mut self, pane: Pane, act: &str) {
        self.usage.count(pane, act, Trigger::Key);
    }

    /// A key ran the action of `pane` that it has there, if it has one.
    pub(crate) fn did_some(&mut self, pane: Pane, act: Option<&str>) {
        if let Some(a) = act {
            self.did(pane, a);
        }
    }

    /// The mouse did what the action `act` of `pane` does (USAGE-1).
    pub(crate) fn did_by_mouse(&mut self, pane: Pane, act: &str) {
        self.usage.count(pane, act, Trigger::Mouse);
    }

    /// The mouse selection is final: a copy, or with alt the text for the chat (PANES-44). The
    /// pane is the one whose keymap has that action for the text under the mouse.
    pub(crate) fn count_selection(&mut self, region: Region, to_chat: bool) {
        let pane = match region {
            Region::Note if self.mode == Mode::Write => Pane::Write,
            Region::Note => Pane::Read,
            Region::Panel => Pane::Transcript,
            Region::Delta => Pane::Delta,
        };
        self.did_by_mouse(pane, if to_chat { "add_to_chat" } else { "copy" });
    }

    /// A click on the button of an agent session or of the `F5` terminal.
    pub(crate) fn count_session_button(&mut self, n: usize) {
        let act = if n < SESSIONS { format!("session_{}", n + 1) } else { "terminal".to_string() };
        if n <= SESSIONS {
            self.did_by_mouse(Pane::Global, &act);
        }
    }
}
