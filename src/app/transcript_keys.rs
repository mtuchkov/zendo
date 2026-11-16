//! The keys of the chat transcript that need more than the transcript: `y` copies, and `a` adds
//! the text to the chat input as a chip of its own color.
use super::*;

impl App {
    /// The keys of the transcript.
    pub(crate) fn key_transcript(&mut self, act: Option<&str>) {
        let page = self.frame.view_height().saturating_sub(3).max(1) as isize;
        let Some(act) = act else { return };
        if self.threads.transcript_mut().key(act, page, Instant::now()) {
            return;
        }
        match act {
            "copy" => self.copy_transcript(),
            "add_to_chat" => self.transcript_to_chat(),
            _ => {}
        }
    }

    fn copy_transcript(&mut self) {
        let text = self.threads.transcript().text();
        if !text.is_empty() {
            self.pointer.copy(text);
            self.threads.transcript_mut().unselect();
        }
    }

    /// `a`: the selected text of the chat goes to the input as a chip of its own color.
    fn transcript_to_chat(&mut self) {
        let text = self.threads.transcript().text();
        if text.is_empty() {
            return self.flash("nothing to add here");
        }
        let quoted: String = text.lines().map(|l| format!("> {l}\n")).collect();
        let ctx = format!("Text from this chat:\n{quoted}");
        let label = self.composer.add_chip(format!("{CHAT_MARK} {}", chips::selection_label(&text)), Some(ctx), None, true);
        self.threads.transcript_mut().unselect();
        self.flash(format!("{} is in the chat", label.trim_matches([chips::OPEN, chips::CLOSE])));
    }
}
