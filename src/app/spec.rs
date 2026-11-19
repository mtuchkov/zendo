//! Note to spec (docs/zendo/note-to-spec.md): `[ → spec ]` or `s` gives the agent in the chat the
//! task to turn a note into specifications; the agent chooses to update them or to write new ones.
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

impl App {
    /// The file is a note of the notes folder: only a note turns into a spec (SPEC-3).
    pub fn is_note(&self, path: &Path) -> bool {
        path.starts_with(&self.files.store().dir) && path.extension().is_some_and(|e| e == "md")
    }

    /// `s` or a click on `[ → spec ]`: the task goes to the chat of the active agent session, and
    /// waits there while the chat answers (SPEC-1, SPEC-5).
    pub(crate) fn note_to_spec(&mut self) {
        let Some(path) = self.files.current().cloned() else { return self.flash("open a note first") };
        if !self.is_note(&path) {
            return self.flash(format!("only a note of {} turns into a spec", self.rel(&self.cfg.notes_dir)));
        }
        let rel = self.rel(&path);
        self.open_panel();
        if !self.threads.shown() {
            return;
        }
        self.composer_mut().set_context(spec_task(&rel));
        let scope = self.scope();
        self.ask(spec_question(&rel), scope);
    }
}

#[cfg(test)]
#[path = "spec_tests.rs"]
mod tests;
