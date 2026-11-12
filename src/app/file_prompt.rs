//! A question about a file, in the status bar: delete, rename, move, new file or folder.
use std::path::PathBuf;

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

    pub(crate) fn field_mut(&mut self) -> Option<&mut String> {
        match self {
            FilePrompt::Delete(_) => None,
            FilePrompt::Rename(_, t) | FilePrompt::Move(_, t) | FilePrompt::NewFile(t) | FilePrompt::NewFolder(t) => Some(t),
        }
    }
}
