//! The names of the agent sessions (docs/zendo/session-names/session-names.md): from the first
//! agent message of a Claude session, also while the turn runs, the agent names the conversation in
//! a throwaway session, so the question for the name is not in the thread (NAMES-1, NAMES-9). Later
//! it checks whether the name still fits (NAMES-14). A name belongs to a session ID (NAMES-3).
use super::*;
use crate::app::{Item, SESSIONS, session_key};

/// More characters of the conversation before the next question for a name (NAMES-10, NAMES-14).
const STEP_CHARS: usize = 200;
/// More time of a running turn before the next question for a name (NAMES-10, NAMES-14).
const STEP_TIME: Duration = Duration::from_secs(30);
/// The kept names after which the thresholds stop doubling.
const MOST_KEEPS: u32 = 8;
/// The longest name: one word of at most this many letters.
const LONGEST: usize = 12;
/// The messages of the conversation in the question for a name, and the length of each.
const MESSAGES: usize = 8;
const MESSAGE_CHARS: usize = 600;

pub const NAME_PROMPT: &str = "Name this conversation between a user and a coding agent with one short lowercase \
word that tells its topic, such as a module, a feature or a problem. Do not use these names: {{TAKEN}}.\n\
If the conversation does not tell its topic yet, answer `-` alone.\n\
Answer with three such words on one line, the best first, and nothing else.\n\n--- conversation ---\n{{MESSAGES}}";

pub const RENAME_PROMPT: &str = "Does this name still fit? A conversation between a user and a coding agent has \
the name `{{NAME}}`. If that word still tells its topic, answer `=` alone. Otherwise name its topic now with \
one short lowercase word, such as a module, a feature or a problem. Do not use these names: {{TAKEN}}.\n\
Answer with `=`, or with three such words on one line, the best first, and nothing else.\n\n\
--- conversation ---\n{{MESSAGES}}";

/// The last question for the name of a Claude session (NAMES-10, NAMES-14).
#[derive(Clone, Copy, Debug)]
pub struct Asked {
    /// The size of the conversation then: see `told`.
    pub size: usize,
    pub at: Instant,
    /// The answers that kept the name; each doubles the thresholds.
    pub keeps: u32,
}

impl App {
    /// The session ID of the Claude session of agent session `n`.
    fn claude_session(&self, n: usize) -> Option<String> {
        self.state.session(&self.cfg.agent, &session_key(n))
    }

    /// The name of agent session `n`, if its Claude session has one (NAMES-3).
    pub fn session_name(&self, n: usize) -> Option<String> {
        self.claude_session(n).and_then(|id| self.state.names.get(&id).cloned())
    }

    /// The names of the other agent sessions (NAMES-2).
    fn taken_names(&self, n: usize) -> Vec<String> {
        (0..SESSIONS).filter(|m| *m != n).filter_map(|m| self.session_name(m)).collect()
    }

    /// The first agent session to ask about its name: its conversation has a question and an agent
    /// message, and it is due (NAMES-1, NAMES-10, NAMES-14). Its number, session ID, size and
    /// messages.
    fn to_name(&self) -> Option<(usize, String, usize, String)> {
        let now = Instant::now();
        (0..SESSIONS).find_map(|n| {
            let id = self.claude_session(n)?;
            let t = self.threads.get(&session_key(n))?;
            let items = t.items.get(t.name_from..)?;
            let size = told(items, t.busy.is_some())?;
            due(size, t.busy, self.names_asked.get(&id), now).then(|| (n, id, size, conversation(items)))
        })
    }

    /// Ask the agent for the name of an agent session, or whether its name still fits, if one is
    /// due. True when it asked.
    pub(super) fn name_tick(&mut self) -> bool {
        let Some((n, id, size, messages)) = self.to_name() else { return false };
        let keeps = self.names_asked.get(&id).map_or(0, |a| a.keeps);
        self.names_asked.insert(id.clone(), Asked { size, at: Instant::now(), keeps });
        let prompt = match self.session_name(n) {
            Some(name) => RENAME_PROMPT.replace("{{NAME}}", &name),
            None => NAME_PROMPT.to_string(),
        };
        let prompt = prompt.replace("{{TAKEN}}", &self.taken_names(n).join(", ")).replace("{{MESSAGES}}", &messages);
        self.recap_job = Some(RecapJob { name: Some((n, id)), ..RecapJob::default() });
        let key = format!("{RECAP_PREFIX}name {}", n + 1);
        if let Some(a) = &self.agent {
            if let Some(model) = self.name_model.clone() {
                a.send(Cmd::Model { key: key.clone(), model });
            }
            a.send(Cmd::Prompt { key, known: None, text: prompt, attach: vec![] });
        }
        true
    }

    /// The name turn ended: the first free word of the answer names the session, while its Claude
    /// session is the same; an answer without a new word keeps the name and doubles the thresholds
    /// (NAMES-2, NAMES-10, NAMES-14).
    pub(super) fn name_done(&mut self, job: RecapJob) {
        let Some((n, id)) = job.name else { return };
        if self.claude_session(n).as_ref() != Some(&id) {
            return;
        }
        let old = self.state.names.get(&id).cloned();
        let new = pick(&job.buf, &self.taken_names(n)).filter(|w| Some(w) != old.as_ref());
        match (new, old) {
            (Some(name), _) => {
                self.state.names.insert(id, name);
                self.state.save(&self.cfg.root);
            }
            (None, Some(_)) => {
                if let Some(a) = self.names_asked.get_mut(&id) {
                    a.keeps = (a.keeps + 1).min(MOST_KEEPS);
                }
            }
            (None, None) => {}
        }
    }

    /// Claude Code started a Claude session in agent session `n`. After `/clear` it is a new
    /// conversation without a name; a resume keeps the name, also under a new session ID (NAMES-4,
    /// NAMES-7). The agent session continues the new session ID afterwards.
    pub(crate) fn claude_session_started(&mut self, n: usize, id: String, clear: bool) {
        let key = session_key(n);
        let old = self.claude_session(n);
        if old.as_deref() == Some(id.as_str()) {
            return;
        }
        let name = old.and_then(|o| self.state.names.remove(&o)).filter(|_| !clear);
        if let Some(name) = name {
            self.state.names.insert(id.clone(), name);
        }
        self.state.set_session(&self.cfg.agent, &key, id.clone());
        self.state.save(&self.cfg.root);
        let t = self.thread(&key);
        t.session = Some(id);
        if clear {
            t.name_from = t.items.len();
        }
    }
}

/// The size of the conversation in characters, when it has a question of the user and a complete
/// agent message: one that a later item follows, or any one after the turn (NAMES-1).
fn told(items: &[Item], busy: bool) -> Option<usize> {
    let asked = items.iter().any(|i| matches!(i, Item::User(_)));
    let complete = |(k, i): (usize, &Item)| matches!(i, Item::Agent(s) if !s.trim().is_empty()) && (!busy || k + 1 < items.len());
    (asked && items.iter().enumerate().any(complete)).then(|| items.iter().filter_map(content).map(|(_, s)| s.chars().count()).sum())
}

/// True when the next question for a name is due: the first one at once, the next ones after more
/// characters or more time of a running turn with something new, doubled by each kept name
/// (NAMES-10, NAMES-14).
fn due(size: usize, busy: Option<Instant>, last: Option<&Asked>, now: Instant) -> bool {
    let Some(last) = last else { return true };
    let scale = 1usize << last.keeps.min(MOST_KEEPS);
    let ran = busy.is_some_and(|start| now.saturating_duration_since(start.max(last.at)) >= STEP_TIME * scale as u32);
    size >= last.size + STEP_CHARS * scale || (ran && size > last.size)
}

/// The kind and the text of an item for the question for a name: a message, or the titles of the
/// tool calls.
fn content(i: &Item) -> Option<(&'static str, String)> {
    match i {
        Item::User(s) => Some(("user", s.clone())),
        Item::Agent(s) => Some(("agent", s.clone())),
        Item::Tools { uses, .. } => Some(("tools", uses.iter().map(|u| u.title.as_str()).collect::<Vec<_>>().join("; "))),
        _ => None,
    }
}

/// The first message of the conversation and the last ones, for the question for a name.
fn conversation(items: &[Item]) -> String {
    let all: Vec<String> = items.iter().filter_map(content).map(|(k, s)| format!("{k}: {}", s.chars().take(MESSAGE_CHARS).collect::<String>())).collect();
    let last = all.len().saturating_sub(MESSAGES - 1).max(1);
    all.iter().take(1).chain(all.iter().skip(last)).cloned().collect::<Vec<_>>().join("\n\n")
}

/// The first word of the last line of the answer that can be a name and is free (NAMES-2).
pub fn pick(answer: &str, taken: &[String]) -> Option<String> {
    let line = answer.lines().rev().find(|l| !l.trim().is_empty())?;
    line.split(|c: char| !c.is_alphanumeric() && c != '-')
        .map(|w| w.trim_matches('-').to_lowercase())
        .filter(|w| (2..=LONGEST).contains(&w.chars().count()))
        .find(|w| !taken.contains(w))
}

#[cfg(test)]
#[path = "names_tests.rs"]
mod tests;
