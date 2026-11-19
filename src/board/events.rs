//! The events of the journal of the board, one JSON object on each line (ADR 0018).

use serde::{Deserialize, Serialize};

/// The kind of a message (BOARD-5): an initiation is a question or a task, a response is an
/// answer or a status of a task.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Question,
    Task,
    Answer,
    Status,
}

impl Kind {
    pub fn initiation(self) -> bool {
        matches!(self, Kind::Question | Kind::Task)
    }
    pub fn name(self) -> &'static str {
        match self {
            Kind::Question => "question",
            Kind::Task => "task",
            Kind::Answer => "answer",
            Kind::Status => "status",
        }
    }
}

/// The status of a task: `accepted` or `rejected`, then `done` or `failed`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Accepted,
    Rejected,
    Done,
    Failed,
}

impl Status {
    pub fn parse(s: &str) -> Option<Status> {
        serde_json::from_value(serde_json::Value::String(s.to_string())).ok()
    }
    pub fn name(self) -> &'static str {
        match self {
            Status::Accepted => "accepted",
            Status::Rejected => "rejected",
            Status::Done => "done",
            Status::Failed => "failed",
        }
    }
    /// The last status of a task: no more come after it.
    pub fn last(self) -> bool {
        !matches!(self, Status::Accepted)
    }
}

/// A message from session `from` of the repository `repo`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Msg {
    pub id: String,
    pub repo: String,
    pub from: usize,
    /// The channel: `session:<id>`, `all`, or `<owner>/<name>`. None: the user chooses the
    /// session of a task (BOARD-10).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    pub kind: Kind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<String>,
    /// The channel of the responses; by default the inbox of the sender.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    pub text: String,
}

impl Msg {
    /// The channel of the responses to this message.
    pub fn responses_to(&self) -> String {
        self.reply_to.clone().unwrap_or_else(|| inbox(self.from))
    }
}

/// How a recipient took a message (BOARD-21).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum How {
    Delivered,
    Watched,
}

/// A post about a shared resource (BOARD-13).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Post {
    pub id: String,
    pub repo: String,
    pub session: usize,
    pub resource: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<u64>,
}

/// One event of the journal.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum Event {
    Msg(Msg),
    /// Recipient `to` took the message `msg`; the first claim of the pair wins (BOARD-1).
    Claim { msg: String, to: usize, how: How },
    /// The turn of `to` that took the initiation `msg` ended (BOARD-9).
    Landed { msg: String, to: usize },
    Invite { repo: String, channel: String, session: usize },
    Leave { repo: String, channel: String, session: usize },
    /// The thread of a session ended: its channels close, its posts go to the history (BOARD-7,
    /// BOARD-15).
    ThreadEnd { repo: String, session: usize },
    /// The user chose the session of a task without a target; None: `esc` (BOARD-10).
    Choice { msg: String, to: Option<usize> },
    Post(Post),
    Release { post: String },
    Expired { post: String },
    Renewed { post: String },
    Tombstone { post: String, repo: String, session: usize },
}

/// One line of the journal: the time and the event.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Line {
    pub at: u64,
    #[serde(flatten)]
    pub ev: Event,
}

/// The inbox of session `n`.
pub fn inbox(n: usize) -> String {
    format!("session:{n}")
}

/// The session of an inbox channel.
pub fn inbox_of(channel: &str) -> Option<usize> {
    channel.strip_prefix("session:")?.parse().ok()
}

/// The owner of a named channel `<owner>/<name>`.
pub fn owner_of(channel: &str) -> Option<usize> {
    channel.split_once('/')?.0.parse().ok()
}
