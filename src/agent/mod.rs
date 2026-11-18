//! The Claude side, over the Agent Client Protocol (ACP).
//!
//! `zendo` starts an ACP agent process (by default the Claude Code adapter)
//! and talks to it on a background task. The UI sends `Cmd`s and gets `Event`s.

mod acp;
pub mod api;
mod model;
mod sessions;
mod questions;
mod tool_events;
pub use questions::{Pick, Question};
pub mod work;

use acp::*;
use sessions::*;

use agent_client_protocol::schema::ProtocolVersion;
use agent_client_protocol::v1::schema::{
    CancelNotification, CloseSessionRequest, ContentBlock, InitializeRequest, LoadSessionRequest, NewSessionRequest,
    PermissionOptionKind, PromptRequest, RequestPermissionOutcome, RequestPermissionRequest,
    RequestPermissionResponse, SelectedPermissionOutcome, SessionId, SessionNotification,
    SessionUpdate, TextContent, AvailableCommandsUpdate, SessionConfigOption, SetSessionConfigOptionRequest, AvailableCommandInput, ToolCallContent, ToolKind, ImageContent, ResourceLink,
};
use agent_client_protocol::{AcpAgent, Agent, Client};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// A file or an image that goes with a prompt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Attachment {
    File(PathBuf),
    Image(PathBuf),
}

impl Attachment {
    pub fn path(&self) -> &std::path::Path {
        match self {
            Attachment::File(p) | Attachment::Image(p) => p,
        }
    }

    /// An image by its file name; anything else is a file.
    pub fn of(path: PathBuf) -> Attachment {
        match image_mime(&path) {
            Some(_) => Attachment::Image(path),
            None => Attachment::File(path),
        }
    }
}

pub fn image_mime(p: &std::path::Path) -> Option<&'static str> {
    match p.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

/// The content blocks of a prompt: the text, then each attachment. An image goes as an image when
/// the agent takes images, else as a link to its file, like any other file.
fn blocks(text: String, attach: &[Attachment], can_image: bool) -> Vec<ContentBlock> {
    let mut out = vec![ContentBlock::Text(TextContent::new(text))];
    for a in attach {
        let p = a.path();
        let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        if let (Attachment::Image(_), true, Some(mime)) = (a, can_image, image_mime(p))
            && let Ok(bytes) = std::fs::read(p)
        {
            out.push(ContentBlock::Image(ImageContent::new(crate::text::select::base64(&bytes), mime)));
            continue;
        }
        out.push(ContentBlock::ResourceLink(ResourceLink::new(name, format!("file://{}", p.display()))));
    }
    out
}

#[derive(Debug)]
pub enum Cmd {
    /// Send a prompt in the thread of `key` (a note path).
    /// `known` is a session id saved from an earlier run.
    /// `attach`: files and images that go with the prompt.
    Prompt { key: String, known: Option<String>, text: String, attach: Vec<Attachment> },
    /// Open the session of `key` before the first prompt: an earlier session replays its
    /// messages, and the agent sends its commands.
    Open { key: String, known: Option<String> },
    /// Answer a permission request. `None` cancels it.
    Answer { request: u64, option: Option<String> },
    /// Answer a question of the agent with the picks of the user. `None` cancels it (ACT-4).
    Reply { request: u64, picks: Option<Vec<Pick>> },
    Cancel { key: String },
    /// Forget the thread of `key`; the next prompt starts a new session.
    Forget { key: String },
    /// Start a new session for `key` with the model of the agent that `model` names, if the agent
    /// has one; else with its default model (NAMES-12, NAMES-13).
    Model { key: String, model: String },
}

#[derive(Clone, Debug)]
pub struct DiffView {
    pub path: PathBuf,
    pub old: Option<String>,
    pub new: String,
}

#[derive(Clone, Debug)]
pub struct Choice {
    pub id: String,
    pub name: String,
    pub allow: bool,
    pub always: bool,
}

/// The state of a tool call (ACT-1, ACT-8).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToolState {
    #[default]
    Pending,
    Running,
    Done,
    Failed,
}

impl ToolState {
    /// The call ended, well or not.
    pub fn ended(self) -> bool {
        matches!(self, ToolState::Done | ToolState::Failed)
    }
}

/// What a tool call tells besides its state: its first file and line, the first line of its
/// error, its input, its output, and the exit code of a command (ACT-6, ACT-8, INTER-2).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToolInfo {
    pub place: Option<String>,
    pub line: Option<u32>,
    pub error: Option<String>,
    pub input: Option<String>,
    /// The lines of the output or of the diff; an update without output leaves it empty.
    pub output: Vec<String>,
    pub exit: Option<i32>,
}

impl ToolInfo {
    /// Take what an update tells; what it leaves out stays.
    pub fn merge(&mut self, new: ToolInfo) {
        self.place = new.place.or(self.place.take());
        self.line = new.line.or(self.line);
        self.error = new.error.or(self.error.take());
        self.input = new.input.or(self.input.take());
        self.exit = new.exit.or(self.exit);
        if !new.output.is_empty() {
            self.output = new.output;
        }
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    Ready { agent: String, can_load: bool, can_image: bool },
    Session { key: String, id: String, resumed: bool },
    UserText { key: String, text: String },
    /// History replayed while an earlier thread loads.
    Replay { key: String, user: bool, text: String },
    Text { key: String, text: String },
    Thought { key: String },
    /// A tool call or its update. An update leaves out what did not change.
    Tool { key: String, id: String, title: String, kind: String, state: ToolState, info: ToolInfo },
    Permission { key: String, request: u64, title: String, kind: String, diffs: Vec<DiffView>, choices: Vec<Choice> },
    /// The agent asks the user questions with choices; `Cmd::Reply` answers them (ACT-4).
    Question { key: String, request: u64, questions: Vec<Question> },
    /// A shell command was blocked by policy.
    Blocked { key: String, title: String },
    TurnEnd { key: String, stop: String },
    Error { key: Option<String>, text: String },
    Exited { text: String },
    /// The slash commands of the agent: name, description, and the hint of its input.
    Commands { list: Vec<Command> },
}

/// A slash command of the agent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Command {
    pub name: String,
    pub description: String,
    /// What to type after the name, when the command takes input.
    pub hint: Option<String>,
}

pub struct AgentHandle {
    tx: UnboundedSender<Cmd>,
}

impl AgentHandle {
    pub fn send(&self, cmd: Cmd) {
        let _ = self.tx.send(cmd);
    }
}

type Answer = Box<dyn FnOnce(Option<String>) + Send>;

pub fn spawn(command: String, env: BTreeMap<String, String>, cwd: PathBuf, log: Option<PathBuf>) -> (AgentHandle, UnboundedReceiver<Event>) {
    let (tx, rx) = unbounded_channel();
    let (etx, erx) = unbounded_channel();
    let shared = Arc::new(Shared {
        events: etx,
        keys: Mutex::new(HashMap::new()),
        answers: Mutex::new(HashMap::new()),
        replies: Mutex::new(HashMap::new()),
        next: AtomicU64::new(1),
        loading: Mutex::new(None),
    });
    tokio::spawn(async move {
        let s = shared.clone();
        let text = match run(command, env, cwd, log, rx, shared).await {
            Ok(()) => "agent stopped".to_string(),
            Err(e) => format!("agent stopped: {e}"),
        };
        s.emit(Event::Exited { text });
    });
    (AgentHandle { tx }, erx)
}
