//! Usage counts and the times of the actions: what the dispatch of a key or of the mouse tells the
//! counts (docs/zendo/usage.md) and the measurements (docs/zendo/responsiveness.md, RESP-2).
use super::*;
use crate::keys::Pane;
use crate::responsiveness::Work;
use crate::usage::Trigger;
use std::collections::BTreeSet;

/// The global actions that choose what the chat pane shows: an agent session or a terminal.
pub(crate) fn session_action(act: &str) -> bool {
    matches!(act, "next_session" | "prev_session" | "terminal" | "terminal_run") || act.starts_with("session_")
}

impl App {
    /// The input thread read an input event at `at`: the time of its actions starts (RESP-1).
    pub fn input_at(&mut self, at: Instant) {
        let now = self.in_flight();
        self.responsiveness.input(at, &now);
    }

    /// The draw of the screen ended at `at`: the actions whose work ended get their times (RESP-1).
    pub fn drawn_at(&mut self, at: Instant) {
        let now = self.in_flight();
        self.responsiveness.drawn(at, &now);
    }

    /// The work in the background that the user waits for: the walk and the search of the find
    /// panel, and the answer to each question of each thread.
    fn in_flight(&self) -> BTreeSet<Work> {
        let f = &self.files.find;
        let walk = f.walking(self.mode == Mode::Find).then_some(Work::Walk);
        let search = f.searching().map(Work::Search);
        let answers = self.threads.values().flat_map(|t| t.waiting()).map(Work::Answer);
        walk.into_iter().chain(search).chain(answers).collect()
    }

    /// A key ran the action `act` of the keymap pane `pane` (USAGE-1, USAGE-7).
    pub(crate) fn did(&mut self, pane: Pane, act: &str) {
        self.ran(pane, act, Trigger::Key);
    }

    /// The one place that every action of the keymap passes: its count and its time (USAGE-1,
    /// RESP-1, RESP-2).
    fn ran(&mut self, pane: Pane, act: &str, trigger: Trigger) {
        self.usage.count(pane, act, trigger);
        self.responsiveness.ran(pane, act);
    }

    /// A key ran the action of `pane` that it has there, if it has one.
    pub(crate) fn did_some(&mut self, pane: Pane, act: Option<&str>) {
        if let Some(a) = act {
            self.did(pane, a);
        }
    }

    /// The mouse did what the action `act` of `pane` does (USAGE-1).
    pub(crate) fn did_by_mouse(&mut self, pane: Pane, act: &str) {
        self.ran(pane, act, Trigger::Mouse);
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
        let act = if n < SESSIONS { format!("session_{n}") } else { "terminal".to_string() };
        if n <= SESSIONS {
            self.did_by_mouse(Pane::Global, &act);
        }
    }
}
