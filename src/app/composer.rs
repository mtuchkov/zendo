//! What goes into the next question: the chat input, its chips and attachments, the suggestions
//! under it, and the context that the question brings (ADR 0008). The composer does not see the
//! app. The chips are in `chips.rs`, the suggestions in `slash.rs` and `at.rs`.
use super::{AtPick, Chip, Slash};
#[cfg(test)]
use super::AtKey;
use crate::agent::Attachment;
use crate::text::editor::Editor;
use std::path::PathBuf;

pub struct Composer {
    input: Editor,
    /// Selections added to the chat: the label in the input, and the text that goes as context.
    chips: Vec<Chip>,
    /// Files and images for the next prompt.
    attach: Vec<Attachment>,
    slash: Slash,
    /// `@` in the input: the file suggestions under it.
    at: Option<AtPick>,
    /// The files of the repository while the suggestions are open.
    at_files: Option<Vec<PathBuf>>,
    /// `ctrl-t`: the next suggestions show only templates.
    at_templates: bool,
    /// Text attached to the next question, e.g. the delta.
    context: Option<String>,
    /// The question sent when the input is empty and there is a context.
    default: Option<String>,
    /// What the context is, for the input placeholder: "the changes", "the selection".
    about: &'static str,
    /// The last `alt-o`: the index of its terminal, and the input before it (TERM-28).
    pick: Option<(usize, Vec<String>, usize, usize)>,
}

impl Default for Composer {
    fn default() -> Self {
        Self {
            input: Editor::new(""),
            chips: Vec::new(),
            attach: Vec::new(),
            slash: Slash::default(),
            at: None,
            at_files: None,
            at_templates: false,
            context: None,
            default: None,
            about: "",
            pick: None,
        }
    }
}

impl Composer {
    pub fn input(&self) -> &Editor {
        &self.input
    }

    /// The editor of the input, for the keys that edit it.
    pub fn input_mut(&mut self) -> &mut Editor {
        &mut self.input
    }

    /// The char before the cursor is `\\`: `enter` after it is a new line.
    pub fn ends_with_backslash(&self) -> bool {
        let e = &self.input;
        e.col > 0 && e.lines[e.row].chars().nth(e.col - 1) == Some('\\')
    }

    /// The cursor is in a code block of the input, or at the end of the fence that opens one:
    /// `enter` there is a new line (TEXT-10).
    pub fn in_code(&self) -> bool {
        self.input.in_fence() || self.input.opens_fence()
    }

    pub fn chips(&self) -> &[Chip] {
        &self.chips
    }

    pub fn clear_chips(&mut self) {
        self.chips.clear();
    }

    /// An empty input without chips.
    pub fn clear(&mut self) {
        self.input.take();
        self.chips.clear();
    }

    pub fn slash(&self) -> &Slash {
        &self.slash
    }

    pub fn at(&self) -> Option<&AtPick> {
        self.at.as_ref()
    }

    /// Close the suggestions under the input.
    pub fn close_picks(&mut self) {
        self.at = None;
        self.slash.open = false;
    }

    /// Another key came: the next `alt-o` starts with the first terminal again.
    pub fn drop_pick(&mut self) {
        self.pick = None;
    }

    /// `alt-o` with `n` terminals that have a result: the index of the terminal to take. A second
    /// `alt-o` puts the input back as it was before the first, and takes the next terminal.
    pub fn next_pick(&mut self, n: usize) -> usize {
        let next = match self.pick.take() {
            Some((i, lines, row, col)) => {
                (self.input.lines, self.input.row, self.input.col) = (lines, row, col);
                (i + 1) % n
            }
            None => 0,
        };
        self.pick = Some((next, self.input.lines.clone(), self.input.row, self.input.col));
        next
    }

    pub fn has_context(&self) -> bool {
        self.context.is_some()
    }

    pub fn about(&self) -> &'static str {
        self.about
    }

    /// Text that goes with the next question.
    pub fn set_context(&mut self, context: String) {
        self.context = Some(context);
    }

    pub fn drop_context(&mut self) {
        self.context = None;
    }

    /// The question that goes when the user writes none, and what the context is about.
    pub fn set_default(&mut self, question: String, about: &'static str) {
        (self.default, self.about) = (Some(question), about);
    }

    /// The context for the question that goes now; the default question goes away with it.
    pub fn take_context(&mut self) -> Option<String> {
        self.default = None;
        self.context.take()
    }

    /// The files and images for the question that goes now.
    pub fn take_attach(&mut self) -> Vec<Attachment> {
        std::mem::take(&mut self.attach)
    }

    /// An image waits for the next question.
    pub fn sends_image(&self) -> bool {
        self.attach.iter().any(|a| matches!(a, Attachment::Image(_)))
    }

    /// The text of the input and the chips that are still in it; the input is empty after it.
    pub fn take_message(&mut self) -> (String, Vec<Chip>) {
        let q = self.input.take();
        let chips = std::mem::take(&mut self.chips).into_iter().filter(|c| q.contains(c.label.as_str())).collect();
        (q, chips)
    }

    /// The question to send: the text of the input. The chips in it give their text to the
    /// context and their files to the attachments. Only chips, or nothing, with a context: the
    /// default question goes after the chips.
    pub fn take_question(&mut self) -> String {
        let (mut q, chips) = self.take_message();
        let ctx: Vec<String> = chips.iter().filter_map(|c| c.context.clone()).collect();
        self.attach = chips.into_iter().filter_map(|c| c.attach).collect();
        if !ctx.is_empty() {
            let all: Vec<String> = self.context.take().into_iter().chain(ctx).collect();
            self.context = Some(all.join("\n"));
        }
        let bare = chips::chip_ranges(&q).into_iter().rev().fold(q.clone(), |t, (a, b)| t.chars().take(a).chain(t.chars().skip(b)).collect());
        if bare.trim().is_empty() && self.context.is_some() {
            let default = self.default.take().unwrap_or_else(|| super::DELTA_QUESTION.into());
            q = format!("{} {default}", q.trim()).trim().to_string();
        }
        q
    }
}

#[path = "at.rs"]
pub(crate) mod at;
#[path = "chips.rs"]
pub mod chips;
#[path = "slash.rs"]
pub(crate) mod slash;

#[cfg(test)]
#[path = "composer_tests.rs"]
mod tests;
