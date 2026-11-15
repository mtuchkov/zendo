//! Application state and key handling. Drawing is in `ui`.

mod at;
mod attention;
mod chat;
mod chat_keys;
mod count;
mod cli;
mod terminal;
mod terminal_mouse;
pub mod cursor_map;
pub mod chips;
mod composer;
mod events;
mod effect;
mod files;
mod files_apply;
mod files_state;
mod find;
mod float;
mod jobs;
mod hold;
mod file_prompt;
mod info;
mod input;
mod modes;
mod mouse;
mod music;
mod new;
mod pointer;
mod read;
mod reader;
mod say;
mod selection;
mod slash;
mod tab;
mod thread;
mod threads;
mod transcript;
mod tree;
mod worktree;
mod write;
pub mod write_rows;
pub mod version;

pub use at::{AT_ROWS, AtKey, AtPick};
pub use composer::Composer;
pub use attention::Attention;
pub use effect::Effect;
pub use file_prompt::FilePrompt;
pub use files_state::Files;
pub use files_state::tree_keys::{FilterKey, Picked};
pub use find::{Find, FindItem};
pub use float::Float;
pub use jobs::*;
pub use hold::{CONTINUE, HOLD_AGAIN, Hold};
pub use say::{SAY_HERE, Say};
pub use slash::Slash;
pub use tab::{TAB_PAUSE, TabRun};
pub use thread::*;
pub use threads::Threads;
pub use transcript::{CHAT_MARK, Transcript};
pub use modes::Mode;
pub use pointer::Pointer;
pub use reader::Reader;
pub use worktree::{RECAP_EVERY, TreeAct, WorktreeUi};

use crate::agent::{self, AgentHandle, Attachment, Cmd, Event};
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
    /// The documents and the open one, the file tree, the find panel, the questions about a file.
    pub files: Files,
    pub mode: Mode,
    pub back: Mode,
    /// The pane whose keys the help shows.
    pub help_pane: crate::keys::Pane,
    pub theme: usize,
    pub theme_before: usize,
    pub state: State,
    pub branch: Option<String>,

    /// The view of the note: the read cursor, the selection, and the scroll.
    pub reader: Reader,
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
    pub focus: Focus,
    /// What goes into the next question: the chat input, its chips, the suggestions, the context.
    pub composer: Composer,
    /// The chat threads, the transcript cursor, the pinned thread, and the scroll of the chat pane.
    pub threads: Threads,
    /// The agent takes images in a prompt (from its ACP capabilities).
    pub agent_images: bool,
    /// The keymap of every pane, and what is wrong in `[keys]` of the config.
    pub keys: crate::keys::Keymap,
    pub key_problems: Vec<String>,
    /// The keymap file that the config names, for the help (PANES-31).
    pub keymap_file: Option<String>,
    pub agent: Option<AgentHandle>,
    pub agent_events: Option<tokio::sync::mpsc::UnboundedReceiver<Event>>,
    pub agent_name: Option<String>,
    pub agent_down: Option<String>,

    pub delta_open: bool,
    pub delta_base: Base,
    pub delta_scroll: usize,
    pub visit: Option<Visit>,
    pub recap_job: Option<RecapJob>,
    pub recap_queue: VecDeque<String>,
    pub reviews: crate::agent::work::Reviews,
    /// Files to get a title or front matter from the agent, after write mode.
    pub meta_queue: VecDeque<String>,
    /// For each session ID: the last question for its name (NAMES-10, NAMES-14).
    pub names_asked: HashMap<String, crate::agent::work::Asked>,
    /// The model that names the agent sessions; none: the default model of the agent (NAMES-12).
    pub name_model: Option<String>,
    /// The pasted images that the name model names: by session key, the file and the answer so far.
    pub image_names: HashMap<String, (PathBuf, String)>,
    pub auto_recaps: Vec<Instant>,
    #[default(Instant::now())]
    pub last_input: Instant,
    /// The run of `tab`s (CLAUDE-36).
    pub tab_run: TabRun,
    /// A newer commit on main (short sha), found on start.
    pub update: Option<String>,
    pub update_rx: Option<tokio::sync::oneshot::Receiver<Option<String>>>,
    /// Quit, upgrade, and start again.
    pub upgrade_now: bool,
    /// The last `alt-o`: the index of its terminal, and the input before it (TERM-28).
    pub(crate) output_pick: Option<(usize, Vec<String>, usize, usize)>,
    /// The width of the text column from the config, when the repository sets none.
    pub width_default: u16,
    /// `alt-n`: the new note in a float; the screen behind waits in it (FLOAT-1).
    pub float: Option<Float>,
    /// `ctrl-shift-s`: the arrow in the note, until the answer ends or `esc`.
    pub say: Option<Say>,

    pub flash: Option<(String, Instant)>,
    /// The counts of the actions of the user; off until the start turns them on (USAGE-1).
    pub usage: crate::usage::Usage,
    pub quit: bool,
    /// The Claude Code sessions of the agent sessions, in terminals that zendo owns.
    pub cli: crate::session::Sessions,
    /// `ctrl-o` while the chat answers: the session view waits for the end of the turn (HANDOFF-1).
    pub hold: Option<Hold>,
    /// The bell and the blinking buttons when a turn ends, and the sound button (ATTN-1 to ATTN-11).
    pub attention: Attention,
    /// The lofi music, its button, and the question at the first start (LOFI-8 to LOFI-15).
    pub music: crate::music::Music,

    /// The mouse: its selection, the pane edge in a drag, and the text that it copies.
    pub pointer: Pointer,
    /// What the last draw left for the mouse and the keys; `ui` builds it, the app only reads it.
    pub frame: crate::ui::frame::Frame,
}

impl App {
    /// Draw the screen, and keep what the draw leaves for the mouse and the keys.
    pub fn draw(&mut self, f: &mut ratatui::Frame) {
        let last = std::mem::take(&mut self.frame);
        self.frame = crate::ui::draw(f, self, last);
    }
}

pub const ALL_KEY: &str = "(all notes)";

/// The word limit of a document when project-structure.toml does not set one.
pub const DEFAULT_WORDS: usize = 1000;
pub const SPLIT_PROMPT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/split-document.md"));

pub const DELTA_QUESTION: &str = "What do these changes break or leave open?";
