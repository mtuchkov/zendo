//! Application state and key handling. Drawing is in `ui`.

mod at;
mod chat;
mod chat_keys;
pub mod cursor_map;
pub mod chips;
mod events;
mod files;
mod files_new;
mod files_ops;
mod info;
mod input;
mod modes;
mod mouse;
mod new;
mod read;
mod say;
mod selection;
mod thread;
mod tree;
mod write;

pub use at::{AT_ROWS, AtPick};
pub use say::Say;
pub use thread::*;

use crate::agent::{self, AgentHandle, Attachment, Choice, Cmd, DiffView, Event};
use crate::text::editor::Editor;
use crate::docs::notes::{self, Note, Store};
use crate::docs::search::{self, Group};
use crate::setup::state::{self, State};
use crate::ui::theme::{self, THEMES, Theme};
use crate::agent::work::RECAP_PREFIX;
use chrono::Local;
use crate::text::select::{Region, Selection};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Read,
    Write,
    Notes,
    Search,
    Theme,
    Help,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Focus {
    Tree,
    #[default]
    Note,
    Delta,
    Claude,
}

/// What the delta compares the note with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Base {
    /// The text at the end of your last visit that counted as work.
    #[default]
    LastVisit,
    GitHead,
}

/// One stay on a note. It counts as work after an edit or enough scrolls and clicks.
pub struct Visit {
    pub path: PathBuf,
    pub interactions: u32,
    pub edited: bool,
}

/// Scrolls and clicks that make a visit count, when there is no edit.
pub const ENGAGED: u32 = 12;
/// Claude writes recaps only after the user is idle this long.
pub const IDLE: Duration = Duration::from_secs(45);
/// At most this many automatic recaps per hour.
pub const AUTO_PER_HOUR: usize = 6;

pub struct RecapJob {
    pub rel: String,
    pub hash: String,
    pub text: String,
    pub buf: String,
    pub auto: bool,
    pub failed: Option<String>,
    /// Not a recap: a title and front matter for the file.
    pub meta: Option<crate::agent::work::Meta>,
}

#[derive(Default)]
pub struct Config {
    pub root: PathBuf,
    pub notes_dir: PathBuf,
    pub agent: String,
    pub agent_env: std::collections::BTreeMap<String, String>,
    /// Started with --offline: no Claude in this run.
    pub offline: bool,
    /// The private notes repo, when it is on and `--notes` did not override it.
    pub notes_repo: Option<crate::setup::config::NotesRepo>,
}

#[derive(smart_default::SmartDefault)]
pub struct App {
    pub cfg: Config,
    /// Commits and pushes the notes repo in the background.
    pub syncer: Option<crate::setup::sync::Syncer>,
    pub store: Store,
    pub notes: Vec<Note>,
    pub current: Option<PathBuf>,
    pub mode: Mode,
    pub back: Mode,
    /// The pane whose keys the help shows.
    pub help_pane: crate::keys::Pane,
    pub theme: usize,
    pub theme_before: usize,
    pub state: State,
    pub branch: Option<String>,

    pub scroll: usize,
    pub editor: Editor,
    pub editor_path: Option<PathBuf>,
    #[default(Instant::now())]
    pub last_edit: Instant,
    pub saved_at: Option<chrono::DateTime<Local>>,

    pub notes_sel: usize,
    pub query: String,
    pub results: Vec<Group>,
    pub result_sel: usize,

    pub panel: bool,
    /// A thread shown regardless of the note, e.g. a question across all notes.
    pub pinned: Option<String>,
    pub focus: Focus,
    /// The chat input.
    pub input: Editor,
    /// Selections added to the chat: the label in the input, and the text that goes as context.
    pub chips: Vec<Chip>,
    /// Files and images for the next prompt.
    pub pending_attach: Vec<Attachment>,
    /// The agent takes images in a prompt (from its ACP capabilities).
    pub agent_images: bool,
    /// The time of the last scroll event: a fast run of them is a trackpad.
    last_scroll: Option<std::time::Instant>,
    /// Width of the chat input, for up and down moves in it.
    pub input_width: usize,
    /// The keymap of every pane, and what is wrong in `[keys]` of the config.
    pub keys: crate::keys::Keymap,
    pub key_problems: Vec<String>,
    /// The last text that zendo copied, for paste when the system clipboard cannot be read.
    pub last_copy: Option<String>,
    pub threads: HashMap<String, Thread>,
    pub agent: Option<AgentHandle>,
    pub agent_events: Option<tokio::sync::mpsc::UnboundedReceiver<Event>>,
    pub agent_name: Option<String>,
    pub agent_down: Option<String>,
    pub details: bool,
    pub panel_scroll: usize,

    pub delta_open: bool,
    pub delta_base: Base,
    pub delta_scroll: usize,
    pub visit: Option<Visit>,
    pub recap_job: Option<RecapJob>,
    pub recap_queue: VecDeque<String>,
    /// Files to get a title or front matter from the agent, after write mode.
    pub meta_queue: VecDeque<String>,
    pub auto_recaps: Vec<Instant>,
    #[default(Instant::now())]
    pub last_input: Instant,
    /// Text attached to the next question, e.g. the delta.
    pub ask_context: Option<String>,
    /// The question sent when the input is empty and there is a context.
    pub ask_default: Option<String>,
    /// What the context is, for the input placeholder: "the changes", "the selection".
    pub ask_about: &'static str,
    /// Read mode: the cursor row and the selection anchor, in rendered rows.
    pub read_cursor: usize,
    pub read_anchor: Option<(usize, usize)>,
    /// The column of the read cursor in its row, in chars; and the column it wants on up and down.
    pub read_col: usize,
    pub read_want: usize,
    /// The word limit of the open file, its words, and the row of the word after the limit.
    pub word_limit: usize,
    pub word_count: usize,
    pub over_limit: Option<usize>,
    /// The rendered rows of the read view (plain text) and the row of each source line, from the last frame.
    pub read_text: Vec<String>,
    pub read_rows: Vec<usize>,
    /// Panes in the last frame, left to right.
    pub visible: Vec<Focus>,
    /// Panes in the last frame with their areas, for resizing.
    pub pane_rects: Vec<(Focus, Rect)>,
    /// A pane edge being dragged: the pane, its fixed edge, and whether it grows to the right.
    pub drag: Option<(Focus, u16, bool)>,
    /// A newer commit on main (short sha), found on start.
    pub update: Option<String>,
    pub update_rx: Option<tokio::sync::oneshot::Receiver<Option<String>>>,
    /// Quit, upgrade, and start again.
    pub upgrade_now: bool,
    pub tree: crate::docs::tree::Tree,
    /// Why `project-structure.toml` does not parse, if it does not.
    pub structure_error: Option<String>,
    pub structure_seen: Option<std::time::SystemTime>,
    /// Files opened from the tree outside the notes folder.
    pub extra: Vec<PathBuf>,
    /// Tree rows on screen in the last frame, for mouse clicks.
    pub tree_rows: Vec<(Rect, usize)>,
    /// `delete`, `f2` or `m` on a file: the question in the status bar.
    pub file_prompt: Option<FilePrompt>,
    /// The width of the text column from the config, when the repository sets none.
    pub width_default: u16,
    /// The screen row of the cursor in the note at the last draw: a mode change keeps it.
    pub cursor_y: usize,
    /// After a mode change: the screen row the cursor keeps at the next draw.
    pub keep_y: Option<usize>,
    /// `ctrl-h`: the arrow in the note, until the answer ends or `esc`.
    pub say: Option<Say>,
    /// `@` in the chat input: the file suggestions under it.
    pub at: Option<AtPick>,
    /// The files of the repository while the suggestions are open.
    pub at_files: Option<Vec<PathBuf>>,
    /// `ctrl-t`: the next suggestions show only templates.
    pub at_templates: bool,
    /// Notes folder fingerprint (count and newest change), checked every second.
    pub disk: (usize, Option<std::time::SystemTime>),
    #[default(Instant::now())]
    pub disk_checked: Instant,

    pub flash: Option<(String, Instant)>,
    pub quit: bool,
    /// Set when the user asks to continue the thread in Claude Code.
    pub handoff: Option<String>,

    /// Panels on screen in the last frame, for the mouse.
    pub regions: Vec<(Region, Rect)>,
    pub sel: Option<Selection>,
    /// Copy the selection on the next frame.
    pub copy_pending: bool,
    /// Text to put on the clipboard after the frame.
    pub clip: Option<String>,

    /// Layout facts from the last frame, for keys that move by screen.
    pub view_height: usize,
    pub text_width: usize,
}

pub const ALL_KEY: &str = "(all notes)";

/// The word limit of a document when project-structure.toml does not set one.
pub const DEFAULT_WORDS: usize = 1000;
pub const SPLIT_PROMPT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/split-document.md"));

/// A question about a file, in the status bar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilePrompt {
    /// Delete this file? `y` deletes it.
    Delete(PathBuf),
    /// The new name of this file, as the user types it.
    Rename(PathBuf, String),
    /// The folder to move this file to, from the repository root, as the user types it.
    Move(PathBuf, String),
    /// The path of a new file, from the repository root.
    NewFile(String),
    /// The path of a new folder, from the repository root.
    NewFolder(String),
}

impl FilePrompt {
    /// The text field of the question, when it has one.
    pub fn field(&self) -> Option<&String> {
        match self {
            FilePrompt::Delete(_) => None,
            FilePrompt::Rename(_, t) | FilePrompt::Move(_, t) | FilePrompt::NewFile(t) | FilePrompt::NewFolder(t) => Some(t),
        }
    }

    fn field_mut(&mut self) -> Option<&mut String> {
        match self {
            FilePrompt::Delete(_) => None,
            FilePrompt::Rename(_, t) | FilePrompt::Move(_, t) | FilePrompt::NewFile(t) | FilePrompt::NewFolder(t) => Some(t),
        }
    }
}

pub const DELTA_QUESTION: &str = "What do these changes break or leave open?";
