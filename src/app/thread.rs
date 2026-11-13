//! A chat thread and its items.
use super::*;

/// The key of the first chat thread.
pub const CHAT_KEY: &str = "(chat)";
/// The agent sessions of the chat: F6 to F10.
pub const SESSIONS: usize = 5;

/// The key of the chat thread of session `n`, from 0.
pub fn session_key(n: usize) -> String {
    if n == 0 { CHAT_KEY.into() } else { format!("(chat {})", n + 1) }
}
/// The scope of a question about the note that is not saved yet.
pub const NEW_KEY: &str = "(new note)";
/// A file goes with a question when it did not go with one of the last this many questions.
pub const ATTACH_EVERY: usize = 20;

#[derive(Clone, Debug)]
pub struct ToolUse {
    pub id: String,
    pub title: String,
    pub kind: String,
}

#[derive(Clone, Debug)]
pub enum Item {
    User(String),
    Agent(String),
    Tools { uses: Vec<ToolUse>, started: Instant, took: Option<Duration> },
    Blocked(String),
    Info(String),
    Error(String),
}

#[derive(Default)]
pub struct Thread {
    pub items: Vec<Item>,
    pub busy: Option<Instant>,
    pub session: Option<String>,
    pub thinking: bool,
    /// Context was sent once in this run.
    pub primed: bool,
    /// For each question of this run, the file that went with it.
    pub files: Vec<Option<String>>,
    /// zendo asked the agent to open the session of this thread.
    pub opened: bool,
    /// Questions that wait for the end of the turn (CLAUDE-33).
    pub queue: Vec<Outgoing>,
}

/// A question as it goes to the agent: the text with its context, and the files.
#[derive(Clone, Debug)]
pub struct Outgoing {
    /// What the user asked, as the thread shows it.
    pub question: String,
    pub text: String,
    pub attach: Vec<Attachment>,
    /// The file that went with it, for CLAUDE-26.
    pub sent: Option<String>,
    /// A slash command: no context.
    pub command: bool,
}

impl Thread {
    /// The file went with one of the last ATTACH_EVERY questions.
    pub fn sent_lately(&self, rel: &str) -> bool {
        self.files.iter().rev().take(ATTACH_EVERY).any(|f| f.as_deref() == Some(rel))
    }
}

/// A chip in the chat input: a selection that goes as context, or a file or an image that goes
/// with the prompt.
#[derive(Clone, Debug)]
pub struct Chip {
    pub label: String,
    pub context: Option<String>,
    pub attach: Option<Attachment>,
}

impl Thread {
    /// Adds an item to the thread. The time of the run of tools before it stops: the agent moved on.
    pub fn push(&mut self, it: Item) {
        if let Some(Item::Tools { started, took: took @ None, .. }) = self.items.last_mut() {
            *took = Some(started.elapsed());
        }
        self.items.push(it);
    }
}

#[cfg(test)]
#[path = "thread_tests.rs"]
mod tests;
