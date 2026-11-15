//! The names of the agent sessions (docs/zendo/session-names.md): after the second answer of a
//! Claude session, the agent names the conversation in a throwaway session, so the question for
//! the name is not in the thread (NAMES-9). A name belongs to a session ID (NAMES-3).
use super::*;
use crate::app::{Item, SESSIONS, session_key};

/// The answers of a Claude session before it gets a name (NAMES-1).
const ANSWERS: usize = 2;
/// The longest name: one word of at most this many letters.
const LONGEST: usize = 12;
/// The messages of the conversation in the question for a name, and the length of each.
const MESSAGES: usize = 8;
const MESSAGE_CHARS: usize = 600;

pub const NAME_PROMPT: &str = "Name this conversation between a user and a coding agent with one short lowercase \
word that tells its topic, such as a module, a feature or a problem. Do not use these names: {{TAKEN}}.\n\
Answer with three such words on one line, the best first, and nothing else.\n\n--- conversation ---\n{{MESSAGES}}";

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

    /// The first agent session to name: its Claude session has no name, enough answers, and more
    /// answers than when zendo last asked (NAMES-1, NAMES-10). Its number, session ID, answers and
    /// messages.
    fn to_name(&self) -> Option<(usize, String, usize, String)> {
        (0..SESSIONS).find_map(|n| {
            let id = self.claude_session(n).filter(|id| !self.state.names.contains_key(id))?;
            let t = self.threads.get(&session_key(n)).filter(|t| t.busy.is_none())?;
            let items = t.items.get(t.name_from..)?;
            let answers = answered(items);
            let asked = self.names_asked.get(&id).copied().unwrap_or(0);
            (answers >= ANSWERS && answers > asked).then(|| (n, id, answers, conversation(items)))
        })
    }

    /// Ask the agent for the name of an agent session, if one needs it. True when it asked.
    pub(super) fn name_tick(&mut self) -> bool {
        let Some((n, id, answers, messages)) = self.to_name() else { return false };
        self.names_asked.insert(id.clone(), answers);
        let prompt = NAME_PROMPT.replace("{{TAKEN}}", &self.taken_names(n).join(", ")).replace("{{MESSAGES}}", &messages);
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
    /// session is the same (NAMES-2, NAMES-10).
    pub(super) fn name_done(&mut self, job: RecapJob) {
        let Some((n, id)) = job.name else { return };
        if self.claude_session(n).as_ref() != Some(&id) {
            return;
        }
        if let Some(name) = pick(&job.buf, &self.taken_names(n)) {
            self.state.names.insert(id, name);
            self.state.save(&self.cfg.root);
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

/// The turns of the items that have an answer.
fn answered(items: &[Item]) -> usize {
    items.split(|i| matches!(i, Item::User(_))).skip(1).filter(|turn| turn.iter().any(|i| matches!(i, Item::Agent(_)))).count()
}

/// The first messages of the conversation, for the question for a name.
fn conversation(items: &[Item]) -> String {
    let text = |i: &Item| match i {
        Item::User(s) => Some(format!("user: {}", s.chars().take(MESSAGE_CHARS).collect::<String>())),
        Item::Agent(s) => Some(format!("agent: {}", s.chars().take(MESSAGE_CHARS).collect::<String>())),
        _ => None,
    };
    items.iter().filter_map(text).take(MESSAGES).collect::<Vec<_>>().join("\n\n")
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
