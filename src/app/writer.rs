//! Write mode: the draft in the editor, the new note in a float, the mark of "say it here", and the
//! files that wait for a title or front matter (ADR 0008). The writer does not see the app.
use super::{Float, Say};
use crate::agent::work::TitleJob;
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
    /// The question before the undo of a change of another writer is open (UNDO-12).
    asks_external: bool,
}

impl Draft {
    /// An empty editor, as before write mode.
    fn empty() -> Self {
        Self { editor: document(""), path: None, last_edit: Instant::now(), saved_at: None, asks_external: false }
    }

    /// A new note: an empty title line, the cursor on the first line of the body (FLOAT-17).
    pub fn new_note() -> Self {
        Self { editor: new_note_editor(), ..Self::empty() }
    }
}

/// The text of a new note: an empty title line, an empty line, and the cursor on the first line of
/// the body (FLOAT-17).
pub const NEW_NOTE: &str = "# \n\n";

fn new_note_editor() -> Editor {
    let mut editor = Editor::new(NEW_NOTE);
    // No file yet: the first save is the first operation of the loglet (UNDO-16).
    editor.as_document("");
    editor.move_to_end();
    editor
}

/// An editor of a document with the text of its file: its loglet takes only the saved text
/// (UNDO-16).
fn document(text: &str) -> Editor {
    let mut editor = Editor::new(text);
    editor.as_document(text);
    editor
}

pub struct Writer {
    draft: Draft,
    /// `alt-n`: the new note in a float; the screen behind waits in it (FLOAT-1).
    float: Option<Float>,
    /// `ctrl-shift-s`: the arrow in the note, until the answer ends or `esc`.
    say: Option<Say>,
    /// Files to get a title or front matter from the agent, after write mode.
    meta_queue: VecDeque<String>,
    /// The notes of closed floats that wait for a title from the name model (FLOAT-19).
    titles: VecDeque<TitleJob>,
    /// The closed floats, for titles that come late.
    closed: VecDeque<Closed>,
}

/// A closed float: the time it opened, and the file of its note.
struct Closed {
    opened: DateTime<Local>,
    path: PathBuf,
    /// The note went to spec: the task names this file, so a title does not rename it (SPEC-7).
    keeps_name: bool,
}

/// The closed floats that the writer remembers, for titles that come late.
const CLOSED: usize = 8;

impl Default for Writer {
    fn default() -> Self {
        Self { draft: Draft::empty(), float: None, say: None, meta_queue: VecDeque::new(), titles: VecDeque::new(), closed: VecDeque::new() }
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

    /// Write mode on a new note: an empty title line, no file yet, not saved (FLOAT-17).
    pub fn start_new(&mut self) {
        self.draft.editor = new_note_editor();
        self.draft.path = None;
    }

    /// Write mode on the text of a file.
    pub fn start_on(&mut self, text: &str, path: PathBuf) {
        self.draft.editor = document(text);
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

    /// The question before the undo of an external change is open (UNDO-12).
    pub fn asks_external(&self) -> bool {
        self.draft.asks_external
    }

    pub fn set_asks_external(&mut self, on: bool) {
        self.draft.asks_external = on;
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

    pub fn float_mut(&mut self) -> Option<&mut Float> {
        self.float.as_mut()
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

    /// The float that opened at `opened` closed, and its note is in `path`.
    pub fn float_closed(&mut self, opened: DateTime<Local>, path: PathBuf) {
        self.closed.retain(|c| c.opened != opened);
        self.closed.push_back(Closed { opened, path, keeps_name: false });
        while self.closed.len() > CLOSED {
            self.closed.pop_front();
        }
    }

    /// The file of the note of the closed float that opened at `opened`.
    pub fn closed_note(&self, opened: DateTime<Local>) -> Option<&PathBuf> {
        self.closed.iter().find(|c| c.opened == opened).map(|c| &c.path)
    }

    /// The note in `path` went to spec: when it is the note of a closed float, it keeps the name
    /// of its file (SPEC-7).
    pub fn keep_name_of(&mut self, path: &Path) {
        self.closed.iter_mut().filter(|c| c.path == path).for_each(|c| c.keeps_name = true);
    }

    /// A title does not rename the note of the closed float that opened at `opened` (SPEC-7).
    pub fn keeps_name(&self, opened: DateTime<Local>) -> bool {
        self.closed.iter().any(|c| c.opened == opened && c.keeps_name)
    }

    /// The note of a closed float moved to `path`.
    pub fn closed_moved(&mut self, opened: DateTime<Local>, path: PathBuf) {
        if let Some(c) = self.closed.iter_mut().find(|c| c.opened == opened) {
            c.path = path;
        }
    }

    /// The note of a closed float waits for a title (FLOAT-19).
    pub fn ask_title(&mut self, job: TitleJob) {
        self.titles.push_back(job);
    }

    pub fn next_title(&mut self) -> Option<TitleJob> {
        self.titles.pop_front()
    }

    pub fn titles_wait(&self) -> bool {
        !self.titles.is_empty()
    }
}

#[cfg(test)]
#[path = "writer_tests.rs"]
mod tests;
