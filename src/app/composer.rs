//! What goes into the next question: the chat input, its chips and attachments, the suggestions
//! under it, and the context that the question brings (ADR 0008). The composer does not see the
//! app. The chips are in `chips.rs`, the suggestions in `slash.rs` and `at.rs`.
use super::{AtPick, Chip, Slash};
#[cfg(test)]
use super::AtKey;
use crate::agent::Attachment;
use crate::setup::history::Sent;
use crate::text::editor::Editor;
use crate::setup::state::{DraftChip, InputDraft};
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
    /// `up` in the history: the index of the message on show, and the input before the first `up`
    /// (HISTORY-5, HISTORY-6).
    recall: Option<(usize, Editor, Vec<Chip>)>,
    /// The chip window, the last click on a chip, and the open document that the user unselected
    /// with the stamp of its text (CLAUDE-48, CLAUDE-55).
    view: Option<ChipView>,
    last_click: Option<(String, std::time::Instant)>,
    doc_skip: Option<(String, u64)>,
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
            recall: None,
            view: None,
            last_click: None,
            doc_skip: None,
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
        self.recall = None;
    }

    /// `up` on the first row: the message before in `history`, the oldest first, with its text
    /// chips and the cursor on the first row. False when there is none (HISTORY-5, HISTORY-7).
    pub fn recall_back(&mut self, history: &[Sent]) -> bool {
        let at = self.recall.as_ref().map_or(history.len(), |(i, ..)| *i);
        let Some(i) = at.checked_sub(1).filter(|i| *i < history.len()) else { return false };
        let (_, input, chips) = self.recall.take().unwrap_or_else(|| (0, self.input.clone(), self.chips.clone()));
        self.recall = Some((i, input, chips));
        self.show_sent(&history[i]);
        true
    }

    /// `down` on the last row: the next message, with the cursor at its end; after the newest, the
    /// input and the chips from before the first `up` (HISTORY-6). False when no recall is on.
    pub fn recall_forward(&mut self, history: &[Sent]) -> bool {
        let Some((i, input, chips)) = self.recall.take() else { return false };
        let Some(next) = history.get(i + 1) else {
            (self.input, self.chips) = (input, chips);
            return true;
        };
        self.show_sent(next);
        let row = self.input.lines.len() - 1;
        (self.input.row, self.input.col) = (row, self.input.lines[row].chars().count());
        self.recall = Some((i + 1, input, chips));
        true
    }

    /// A message of the history in the input: its text, and its text chips as chips again.
    fn show_sent(&mut self, m: &Sent) {
        self.input = Editor::new(&m.text);
        self.chips = m.chips.iter().map(|c| Chip { label: c.label.clone(), context: Some(c.text.clone()), attach: None }).collect();
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
        self.recall = None;
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
        // The file of a red chip is gone: the label goes without it (CLAUDE-57).
        self.attach = chips.into_iter().filter_map(|c| c.attach).filter(|a| a.path().exists()).collect();
        if !ctx.is_empty() {
            let all: Vec<String> = self.context.take().into_iter().chain(ctx).collect();
            self.context = Some(all.join("\n"));
        }
        let bare = chips::without_chips(&q);
        if bare.trim().is_empty() && self.context.is_some() {
            let default = self.default.take().unwrap_or_else(|| super::DELTA_QUESTION.into());
            q = format!("{} {default}", q.trim()).trim().to_string();
        }
        q
    }
}

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

    /// Nothing in the input and no context.
    pub fn is_blank(&self) -> bool {
        self.input.is_empty() && self.chips.is_empty() && self.context.is_none()
    }
}

/// What a context can be about, for the placeholder of the input (`set_default`).
const ABOUTS: [&str; 3] = ["the changes", "the selection", "the mark"];

#[path = "at.rs"]
pub(crate) mod at;
#[path = "chips.rs"]
pub mod chips;
#[path = "chip_view.rs"]
pub mod chip_view;
pub use chip_view::ChipView;
#[path = "slash.rs"]
pub(crate) mod slash;

#[cfg(test)]
#[path = "composer_tests.rs"]
mod tests;
