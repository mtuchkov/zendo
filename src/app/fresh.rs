//! `/new <text>`, a command of zendo: a new thread in the zendo session, then the text as a
//! question with the context of the chat input (HISTORY-8 to HISTORY-10, ADR 0027). `ctrl-n` starts
//! the same new thread.
use super::*;

/// The text after `/new` when a question is the command `/new`.
pub fn strip_new(q: &str) -> Option<&str> {
    let rest = q.strip_prefix("/new")?;
    (rest.is_empty() || rest.starts_with(char::is_whitespace)).then(|| rest.trim_start())
}

impl App {
    /// The thread of `key` ends: the next question starts a new Claude session, and the name of
    /// the old one goes (HISTORY-2, NAMES-4).
    pub(crate) fn fresh_thread(&mut self, key: &str) {
        self.link.send(Cmd::Forget { key: key.to_string() });
        if let Some(id) = self.state.take_session(&self.cfg.threads_key(), key) {
            self.state.names.remove(&id);
        }
        self.state.save(&self.cfg.root);
        self.threads.remove(key);
    }

    /// `/new`: a new thread, then its text, if it has some. The questions that wait for the end of
    /// the turn wait in the new thread (HISTORY-8, HISTORY-9, HISTORY-10).
    pub(crate) fn dispatch_fresh(&mut self, key: String, out: Outgoing) {
        let queue = self.threads.get_mut(&key).map(|t| std::mem::take(&mut t.queue)).unwrap_or_default();
        self.fresh_thread(&key);
        self.thread(&key).queue = queue;
        if !out.text.is_empty() {
            let question = strip_new(&out.question).unwrap_or(&out.question).to_string();
            return self.dispatch(key, Outgoing { fresh: false, question, ..out });
        }
        let t = self.thread(&key);
        if let Some(next) = (!t.queue.is_empty()).then(|| t.queue.remove(0)) {
            self.dispatch(key, next);
        }
    }
}

#[cfg(test)]
#[path = "fresh_tests.rs"]
mod tests;
