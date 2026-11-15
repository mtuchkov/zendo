//! Write mode: the draft in the editor, the new note in a float, the mark of "say it here", and the
//! files that wait for a title or front matter (ADR 0008). The writer does not see the app.
use super::{Float, Say};
use crate::text::editor::Editor;
use chrono::{DateTime, Local};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// A text in the editor and its file. The float and the screen behind it each have one.
pub struct Draft {
    editor: Editor,
    /// The file of the text; none before the first save of a new note.
    path: Option<PathBuf>,
    last_edit: Instant,
    saved_at: Option<DateTime<Local>>,
}

impl Draft {
    /// An empty editor, as before write mode.
    fn empty() -> Self {
        Self { editor: Editor::new(""), path: None, last_edit: Instant::now(), saved_at: None }
    }

    /// A new note, with the cursor after `# `.
    pub fn new_note() -> Self {
        let mut editor = Editor::new("# ");
        editor.move_to_end();
        Self { editor, ..Self::empty() }
    }
}

pub struct Writer {
    draft: Draft,
    /// `alt-n`: the new note in a float; the screen behind waits in it (FLOAT-1).
    float: Option<Float>,
    /// `ctrl-shift-s`: the arrow in the note, until the answer ends or `esc`.
    say: Option<Say>,
    /// Files to get a title or front matter from the agent, after write mode.
    meta_queue: VecDeque<String>,
}

impl Default for Writer {
    fn default() -> Self {
        Self { draft: Draft::empty(), float: None, say: None, meta_queue: VecDeque::new() }
    }
}

impl Writer {
    pub fn editor(&self) -> &Editor {
        &self.draft.editor
    }

    /// The editor, for the keys that edit the text.
    pub fn editor_mut(&mut self) -> &mut Editor {
        &mut self.draft.editor
    }

    /// The file of the text in the editor.
    pub fn path(&self) -> Option<&PathBuf> {
        self.draft.path.as_ref()
    }

    pub fn file(&self) -> Option<&Path> {
        self.draft.path.as_deref()
    }

    /// Write mode on a new note: no file yet, not saved.
    pub fn start_new(&mut self) {
        self.draft.editor = Editor::new("# ");
        self.draft.editor.move_to_end();
        self.draft.path = None;
    }

    /// Write mode on the text of a file.
    pub fn start_on(&mut self, text: &str, path: PathBuf) {
        self.draft.editor = Editor::new(text);
        self.draft.path = Some(path);
    }

    /// Write mode starts: the status bar shows no save yet.
    pub fn not_saved(&mut self) {
        self.draft.saved_at = None;
    }

    /// The text is on disk in `path`.
    pub fn saved(&mut self, path: PathBuf) {
        self.draft.editor.dirty = false;
        self.draft.saved_at = Some(Local::now());
        self.draft.path = Some(path);
    }

    pub fn saved_at(&self) -> Option<DateTime<Local>> {
        self.draft.saved_at
    }

    /// A key changed the text now.
    pub fn touch(&mut self) {
        self.draft.last_edit = Instant::now();
    }

    /// The last edit was at `at`.
    pub fn edited_at(&mut self, at: Instant) {
        self.draft.last_edit = at;
    }

    /// The time since the last edit.
    pub fn idle(&self) -> Duration {
        self.draft.last_edit.elapsed()
    }

    /// The float and the screen behind it swap their drafts.
    pub(crate) fn swap_draft(&mut self, other: &mut Draft) {
        std::mem::swap(&mut self.draft, other);
    }

    pub fn float(&self) -> Option<&Float> {
        self.float.as_ref()
    }

    pub fn take_float(&mut self) -> Option<Float> {
        self.float.take()
    }

    pub fn set_float(&mut self, float: Float) {
        self.float = Some(float);
    }

    pub fn say(&self) -> Option<&Say> {
        self.say.as_ref()
    }

    pub fn start_say(&mut self, say: Say) {
        self.say = Some(say);
    }

    pub fn end_say(&mut self) {
        self.say = None;
    }

    /// The file waits for a title or front matter from the agent. False when it waits already.
    pub fn ask_meta(&mut self, rel: String) -> bool {
        let new = !self.meta_queue.contains(&rel);
        if new {
            self.meta_queue.push_back(rel);
        }
        new
    }

    pub fn meta_waits(&self, rel: &str) -> bool {
        self.meta_queue.iter().any(|r| r == rel)
    }

    pub fn next_meta(&mut self) -> Option<String> {
        self.meta_queue.pop_front()
    }
}

#[cfg(test)]
#[path = "writer_tests.rs"]
mod tests;
