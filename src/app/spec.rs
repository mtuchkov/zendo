//! Note to spec (docs/zendo/note-to-spec.md): `[ → spec ]` or `s` gives an agent session the task to
//! turn a note into specifications; the agent chooses to update them or to write new ones. The task
//! goes to the free agent session with the lowest id, else into the chat input of the active one
//! with `/new` (SPEC-1, SPEC-9, SPEC-10). In the float, the button closes the float first (SPEC-6).
use super::*;

pub const SPEC_PROMPT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/note-to-spec.md"));

/// The label of the button in the head of a note (SPEC-1).
pub const SPEC_BUTTON: &str = "[ → spec ]";

/// The task for the agent: the note, and how to turn it into specifications (SPEC-2, SPEC-4).
pub fn spec_task(rel: &str) -> String {
    SPEC_PROMPT.replace("{{FILE}}", rel)
}

/// The question in the chat for the note `rel`.
pub fn spec_question(rel: &str) -> String {
    format!("Turn {rel} into specifications: update the ones that cover it, or write new ones.")
}

/// The label of the chip that carries the instructions of the task in the chat input (SPEC-10).
const SPEC_CHIP: &str = "note to spec";

impl App {
    /// The file is a note of the notes folder: only a note turns into a spec (SPEC-3).
    pub fn is_note(&self, path: &Path) -> bool {
        path.starts_with(&self.files.store().dir) && path.extension().is_some_and(|e| e == "md")
    }

    /// `s` or a click on `[ → spec ]`: the open note, or the note of the float, goes to spec.
    pub(crate) fn note_to_spec(&mut self) {
        if self.writer.float().is_some() {
            return self.float_to_spec();
        }
        let Some(path) = self.files.current().cloned() else { return self.flash("open a note first") };
        self.send_to_spec(&path);
    }

    /// The task for the note `path`: to the free agent session with the lowest id, which the chat
    /// shows; the task waits there while the chat answers (SPEC-1, SPEC-5, SPEC-9). With no free
    /// session, into the chat input of the active one (SPEC-10).
    fn send_to_spec(&mut self, path: &Path) {
        if !self.is_note(path) {
            return self.flash(format!("only a note of {} turns into a spec", self.rel(&self.cfg.notes_dir)));
        }
        let rel = self.rel(path);
        // The chat with the task shows, not a terminal in its place (TERM-37).
        self.cli.pane = None;
        let Some(n) = self.free_session() else { return self.spec_into_input(path, &rel) };
        if n == self.state.session {
            self.open_panel();
        } else {
            self.switch_session(n);
        }
        // The note pane shows the note where the user wrote it, also in a session with a checkout
        // of its own, and the task names that file (SPEC-11).
        if self.session_path(path) != path {
            self.files.keep_as_is(path.to_path_buf());
        }
        if self.files.current().map(PathBuf::as_path) != Some(path) {
            self.open_file(path.to_path_buf());
        }
        if !self.threads.shown() || self.cfg.offline {
            return;
        }
        // The task names this file: a title that comes later does not rename it (SPEC-7).
        self.writer.keep_name_of(path);
        self.composer_mut().set_context(spec_task(&rel));
        let scope = self.scope();
        self.ask_about(spec_question(&rel), scope, path.to_path_buf());
        // A task that did not go leaves no context for the next question.
        self.composer_mut().drop_context();
    }

    /// The agent session with the lowest id that never got a question, has no session ID, no
    /// Claude Code session, and an empty chat input (SPEC-9).
    fn free_session(&self) -> Option<usize> {
        let asked = |n: usize| self.threads.get(&session_key(n)).is_some_and(|t| t.busy.is_some() || t.items.iter().any(|i| matches!(i, Item::User(_))));
        let kept = |n: usize| self.state.session(&self.cfg.agent, &session_key(n)).is_some();
        (0..SESSIONS).find(|&n| !kept(n) && !asked(n) && !self.cli.running(n) && self.chat.composer(n).is_blank())
    }

    /// No free session: `/new` and the task at the start of the chat input of the active session,
    /// the draft under them; nothing goes. The instructions are a chip of the task, so they go only
    /// with it. The chat input of a session in Claude Code goes to Claude Code: no task goes into
    /// it (SPEC-10, SPEC-12, SESSIONS-4).
    fn spec_into_input(&mut self, path: &Path, rel: &str) {
        self.open_panel();
        if !self.threads.shown() || self.cli_blocks() {
            return;
        }
        // The task in the input names this file: a title does not rename it (SPEC-7).
        self.writer.keep_name_of(path);
        let (question, c) = (spec_question(rel), self.composer());
        let text = c.input().text();
        // The task is in the input already, with `/new` or without it, and with its instructions.
        let rules = c.chips().iter().any(|chip| chip.label.contains(SPEC_CHIP) && text.contains(chip.label.as_str()));
        if rules && text.contains(&question) {
            return self.flash("the task is in the chat input");
        }
        self.composer_mut().put_first(&format!("/new {question}"), SPEC_CHIP, &spec_task(rel));
        self.flash("no free agent session: enter starts a new thread with the task; delete /new to ask in this one");
    }

    /// `[ → spec ]` in the float: save the note, close the float, open the note in the note pane,
    /// and send the task at once, with the name that the file has. A title that comes later goes
    /// into the note and does not rename the file (SPEC-6, SPEC-7, SPEC-8).
    fn float_to_spec(&mut self) {
        self.save();
        let Some(path) = self.writer.path().cloned() else { return self.flash("the note has no text yet") };
        self.close_float();
        // A float over a session on the full screen: the note and the chat show (FLOAT-14).
        self.cli.view = None;
        self.open_file(path.clone());
        self.send_to_spec(&path);
    }
}

#[cfg(test)]
#[path = "spec_tests.rs"]
mod tests;
