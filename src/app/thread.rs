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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answered {
    Pending,
    Applied,
    Discarded,
    EditFirst,
}

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
    Proposal { request: u64, title: String, diffs: Vec<DiffView>, choices: Vec<Choice>, state: Answered },
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
}

impl Thread {
    /// The file went with one of the last ATTACH_EVERY questions.
    pub fn sent_lately(&self, rel: &str) -> bool {
        self.files.iter().rev().take(ATTACH_EVERY).any(|f| f.as_deref() == Some(rel))
    }

    pub fn pending(&self) -> Option<(usize, u64)> {
        self.items.iter().enumerate().rev().find_map(|(i, it)| match it {
            Item::Proposal { request, state: Answered::Pending, .. } => Some((i, *request)),
            _ => None,
        })
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
