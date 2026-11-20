//! The chat input of a session as a draft in the state, and back (CLAUDE-45, ADR 0011).
use super::{Chip, Composer};
use crate::agent::Attachment;
use crate::setup::state::{DraftChip, InputDraft};
use crate::text::editor::Editor;

impl Composer {
    /// The input as a draft for the state (CLAUDE-45).
    pub fn draft(&self) -> InputDraft {
        let chips = self.chips.iter().map(|c| DraftChip {
            label: c.label.clone(),
            context: c.context.clone(),
            file: c.attach.as_ref().map(|a| a.path().to_path_buf()),
            image: matches!(c.attach, Some(Attachment::Image(_))),
        });
        InputDraft { text: self.input.text(), chips: chips.collect(), context: self.context.clone(), default: self.default.clone(), about: self.about.to_string() }
    }

    /// The input of a draft, with the cursor at its end.
    pub fn from_draft(d: &InputDraft) -> Self {
        let attach = |c: &DraftChip| c.file.clone().map(|f| if c.image { Attachment::Image(f) } else { Attachment::File(f) });
        let chips = d.chips.iter().map(|c| Chip { label: c.label.clone(), context: c.context.clone(), attach: attach(c) }).collect();
        let mut input = Editor::new(&d.text);
        input.move_to_end();
        let about = ABOUTS.iter().find(|a| **a == d.about).copied().unwrap_or("");
        Self { input, chips, context: d.context.clone(), default: d.default.clone(), about, ..Self::default() }
    }

    /// The labels of the chips whose file or image is gone: they show in red (CLAUDE-46).
    pub fn gone(&self) -> Vec<String> {
        self.chips.iter().filter(|c| c.attach.as_ref().is_some_and(|a| !a.path().exists())).map(|c| c.label.clone()).collect()
    }

    /// Nothing in the input and no context. A chip that the input does not show any more goes
    /// with no question, so it does not count.
    pub fn is_blank(&self) -> bool {
        self.input.is_empty() && self.context.is_none()
    }
}

/// What a context can be about, for the placeholder of the input (`set_default`).
const ABOUTS: [&str; 3] = ["the changes", "the selection", "the mark"];
