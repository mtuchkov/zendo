//! Application state and key handling. Drawing is in `ui`.

mod at_keys;
mod attention;
mod branch;
mod chat;
mod chat_keys;
mod count;
mod cli;
mod terminal;
mod terminal_mouse;
pub mod cursor_map;
mod composer;
mod events;
mod effect;
mod files;
mod files_apply;
mod files_state;
mod find;
mod float;
mod jobs;
mod key_help;
mod link;
mod lists;
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
mod key_hints;
pub mod rekey;
mod say;
mod spec;
mod selection;
mod tab;
mod thread;
mod threads;
mod transcript;
mod transcript_keys;
mod tree;
mod update;
mod work;
mod worktree;
mod write;
pub mod write_rows;
mod writer;
pub mod version;

pub use at::{AT_ROWS, AtKey, AtPick};
pub use composer::Composer;
pub use composer::chips;
pub(crate) use composer::{at, slash};
pub use attention::Attention;
pub use effect::Effect;
pub use file_prompt::FilePrompt;
pub use files_state::Files;
pub use files_state::tree_keys::{FilterKey, Picked};
pub use find::{Find, FindItem};
pub use float::Float;
pub use jobs::*;
pub use key_help::KeyHelp;
pub use link::AgentLink;
pub use lists::Lists;
pub use update::Update;
pub use hold::{CONTINUE, HOLD_AGAIN, Hold};
pub use say::{SAY_HERE, Say};
pub use spec::{SPEC_BUTTON, spec_question, spec_task};
pub use slash::Slash;
pub use tab::{TAB_PAUSE, TabRun};
pub use thread::*;
pub use threads::Threads;
pub use transcript::{CHAT_MARK, Transcript};
pub use modes::Mode;
pub use pointer::Pointer;
pub use reader::Reader;
pub use work::Work;
pub use writer::{Draft, Writer};
pub use worktree::{RECAP_EVERY, TreeAct, WorktreeUi};

use crate::agent::{self, Attachment, Cmd, Event};
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
    /// The documents and the open one, the file tree, the find panel, the questions about a file.
    pub files: Files,
    pub mode: Mode,
    pub back: Mode,
    pub theme: usize,
    pub theme_before: usize,
    pub state: State,

    /// The view of the note: the read cursor, the selection, and the scroll.
    /// The notes list and the search: the selection, the query, the matches.
    pub lists: Lists,
    /// The check for a newer zendo, and the upgrade at the quit.
    pub update: Update,
    /// The help of the keys: its pane, the keymap file, the problems of the keymap.
    pub key_help: KeyHelp,
    pub reader: Reader,
    /// Write mode: the draft in the editor, the float, the mark of "say it here".
    pub writer: Writer,


    pub focus: Focus,
    /// What goes into the next question: the chat input, its chips, the suggestions, the context.
    pub composer: Composer,
    /// The chat threads, the transcript cursor, the pinned thread, and the scroll of the chat pane.
    pub threads: Threads,
    /// The keymap of every pane, and what is wrong in `[keys]` of the config.
    pub keys: crate::keys::Keymap,

    /// The connection to the agent, and the questions for the names of sessions and images.
    pub link: AgentLink,
    /// The delta, the visit that counts, and the recaps and reviews of the agent.
    pub work: Work,
    /// The run of `tab`s (CLAUDE-36).
    pub tab_run: TabRun,
    /// The width of the text column from the config, when the repository sets none.
    pub width_default: u16,

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

    /// The key button and the key float: the change of a key in place (REKEY-1).
    pub rekey: rekey::Rekey,
    /// The mouse: its selection, the pane edge in a drag, and the text that it copies.
    pub pointer: Pointer,
    /// What the last draw left for the mouse and the keys; `ui` builds it, the app only reads it.
    pub frame: crate::ui::frame::Frame,
}

impl App {
    /// Draw the screen, and keep what the draw leaves for the mouse and the keys.
    pub fn draw(&mut self, f: &mut ratatui::Frame) {
        // FILES-58: while the tree shows, it shows the open document, whatever changed it.
        if self.state.tree {
            self.files.follow_current();
        }
        let last = std::mem::take(&mut self.frame);
        self.frame = crate::ui::draw(f, self, last);
    }
}

pub const ALL_KEY: &str = "(all notes)";

/// The word limit of a document when project-structure.toml does not set one.
pub const DEFAULT_WORDS: usize = 1000;
pub const SPLIT_PROMPT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/split-document.md"));

pub const DELTA_QUESTION: &str = "What do these changes break or leave open?";
