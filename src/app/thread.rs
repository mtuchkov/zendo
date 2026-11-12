//! A chat thread and its items.
use super::*;

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
}

impl Thread {
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
