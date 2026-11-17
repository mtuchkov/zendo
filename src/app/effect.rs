//! What a sub-state asks the app to do in the other sub-states (ADR 0008). A sub-state changes its
//! own fields directly; an effect is only for what it cannot reach.
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Show a message in the status bar.
    Flash(String),
    /// A file is gone: the thread and the session of its key go with it.
    Deleted(String),
    /// A file has a new path: its thread, its session and its place in the recent files follow.
    /// `done` is the word for the message: "renamed" or "moved".
    Moved { old: PathBuf, target: PathBuf, done: &'static str },
    /// A new file: it opens to write.
    MadeFile(PathBuf),
    /// A new folder: the tree shows it.
    MadeFolder(PathBuf),
}

impl Effect {
    pub(crate) fn flash(text: impl Into<String>) -> Option<Effect> {
        Some(Effect::Flash(text.into()))
    }
}
