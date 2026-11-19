//! The chat pane as a container: one chat input for each agent session, and the slash commands of
//! the agent, which are the same for all sessions (ADR 0011). The chat pane does not see the app.
use super::{Chip, Composer, SESSIONS};
use crate::agent::Command;
use crate::setup::state::InputDraft;
use std::collections::BTreeMap;
use std::time::{Duration, Instant};

/// How long the input stays unchanged before its draft goes to the state.
pub const DRAFT_PAUSE: Duration = Duration::from_secs(2);

/// What an agent session has in the chat pane: its chat input.
#[derive(Default)]
pub struct AgentSession {
    composer: Composer,
}

pub struct ChatPane {
    sessions: Vec<AgentSession>,
    commands: Vec<Command>,
    /// The drafts in the state, and the last ones that the pane saw, with the time of their change.
    saved: BTreeMap<usize, InputDraft>,
    seen: BTreeMap<usize, InputDraft>,
    changed: Option<Instant>,
}

impl Default for ChatPane {
    fn default() -> Self {
        Self { sessions: (0..SESSIONS).map(|_| AgentSession::default()).collect(), commands: Vec::new(), saved: BTreeMap::new(), seen: BTreeMap::new(), changed: None }
    }
}

impl ChatPane {
    pub fn composer(&self, n: usize) -> &Composer {
        &self.sessions[n.min(SESSIONS - 1)].composer
    }

    pub fn composer_mut(&mut self, n: usize) -> &mut Composer {
        &mut self.sessions[n.min(SESSIONS - 1)].composer
    }

    /// The slash commands of the agent go to the input of every session.
    pub fn set_commands(&mut self, list: Vec<Command>) {
        self.commands = list;
        for s in &mut self.sessions {
            s.composer.set_commands(self.commands.clone());
            s.composer.update_slash();
        }
    }

    /// The session whose input has a chip that `f` picks.
    pub fn session_with(&self, f: impl Fn(&Chip) -> bool) -> Option<usize> {
        self.sessions.iter().position(|s| s.composer.chips().iter().any(&f))
    }

    /// The drafts of the sessions that have something in their input.
    pub fn drafts(&self) -> BTreeMap<usize, InputDraft> {
        self.sessions.iter().enumerate().filter(|(_, s)| !s.composer.is_blank()).map(|(n, s)| (n, s.composer.draft())).collect()
    }

    /// The inputs from the drafts of the state, at the start.
    pub fn load(&mut self, drafts: &BTreeMap<usize, InputDraft>) {
        for (n, d) in drafts.iter().filter(|(n, _)| **n < SESSIONS) {
            self.sessions[*n].composer = Composer::from_draft(d);
            self.sessions[*n].composer.set_commands(self.commands.clone());
        }
        self.saved = self.drafts();
        self.seen = self.saved.clone();
    }

    /// True when the drafts changed and then stayed for `DRAFT_PAUSE`: time to save them.
    pub fn due(&mut self, now: Instant) -> bool {
        let drafts = self.drafts();
        if drafts != self.seen {
            self.seen = drafts;
            self.changed = Some(now);
            return false;
        }
        self.seen != self.saved && self.changed.is_some_and(|t| now.duration_since(t) >= DRAFT_PAUSE)
    }

    /// The drafts went to the state.
    pub fn mark_saved(&mut self) {
        self.saved = self.drafts();
        self.seen = self.saved.clone();
        self.changed = None;
    }
}

#[cfg(test)]
#[path = "chat_pane_tests.rs"]
mod tests;
