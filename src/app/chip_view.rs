//! The chip window: what a chip of the chat input brings, after a double-click (CLAUDE-48 to
//! CLAUDE-53); and the choice of the open document above the input (CLAUDE-54 to CLAUDE-56).
use super::Composer;
use super::super::*;
use super::chips;

/// Two clicks on one chip within this time are a double-click.
pub const DOUBLE_CLICK: Duration = Duration::from_millis(500);

/// The chip window: the label of the chip, its lines (the text, and true for a dim line), the
/// scroll, and the style of the chip: 0 a text or a file, 1 text from the chat, 2 an error.
#[derive(Clone, Debug, PartialEq)]
pub struct ChipView {
    pub label: String,
    pub lines: Vec<(String, bool)>,
    pub scroll: usize,
    pub style: usize,
}

/// The lines of a context: the quoted text plain, without `> `; the line of its source dim.
pub fn context_lines(ctx: &str) -> Vec<(String, bool)> {
    ctx.lines().map(|l| match l.strip_prefix("> ").or((l == ">").then_some("")) {
        Some(q) => (q.to_string(), false),
        None => (l.to_string(), true),
    }).collect()
}

/// What the worker found for the window of a file chip.
pub enum ChipFile {
    Text(String),
    /// The file is not text, or an image: it opened in the program of the system, or why not.
    Outside(Result<(), String>),
    Error(String),
}

impl Composer {
    pub fn view(&self) -> Option<&ChipView> {
        self.view.as_ref()
    }

    pub fn close_view(&mut self) {
        self.view = None;
    }

    /// Scroll the text of the chip window by `by` lines.
    pub fn scroll_view(&mut self, by: isize) {
        if let Some(v) = &mut self.view {
            v.scroll = v.scroll.saturating_add_signed(by).min(v.lines.len().saturating_sub(1));
        }
    }

    /// A click on the chip `label` at `now`: true when it is the second one of a double-click.
    pub fn double_click(&mut self, label: &str, now: Instant) -> bool {
        let double = self.last_click.as_ref().is_some_and(|(l, t)| l == label && now.duration_since(*t) < DOUBLE_CLICK);
        self.last_click = (!double).then(|| (label.to_string(), now));
        double
    }

    /// The open document `id` with the stamp of its text goes with the question (CLAUDE-55).
    pub fn doc_selected(&self, id: &str, stamp: u64) -> bool {
        self.doc_skip.as_ref().is_none_or(|(i, s)| i != id || *s != stamp)
    }

    pub fn toggle_doc(&mut self, id: &str, stamp: u64) {
        self.doc_skip = if self.doc_selected(id, stamp) { Some((id.to_string(), stamp)) } else { None };
    }

    /// Another document or another text: the choice goes, and the document is selected again
    /// (CLAUDE-56).
    pub fn sync_doc(&mut self, id: &str, stamp: u64) {
        if self.doc_selected(id, stamp) {
            self.doc_skip = None;
        }
    }

    /// Another document, also before its stamp is known: the choice goes (CLAUDE-56, IO-6).
    pub fn sync_doc_id(&mut self, id: &str) {
        if self.doc_skip.as_ref().is_some_and(|(i, _)| i != id) {
            self.doc_skip = None;
        }
    }
}

impl App {
    /// A click on a chip of the chat input; the second one of a double-click opens it.
    pub(crate) fn click_chip(&mut self, label: String) {
        self.focus = Focus::Claude;
        if self.composer_mut().double_click(&label, Instant::now()) {
            self.open_chip(&label);
        }
    }

    /// A click on a chip of a message in the transcript; the second one of a double-click opens it
    /// (CLAUDE-63).
    pub(crate) fn click_message_chip(&mut self, content: String) {
        if self.composer_mut().double_click(&content, Instant::now()) {
            self.open_message_chip(&content);
        }
    }

    /// What a chip of a message brings, from the text in its marks: `@` and a file, or its text;
    /// a chip with only a label says that zendo does not know (CLAUDE-65).
    pub(crate) fn open_message_chip(&mut self, content: &str) {
        let name = chips::label_of(content);
        if chips::unknown(content) {
            let lines = vec![("zendo does not know what this chip brought".to_string(), true)];
            return self.composer_mut().view = Some(ChipView { label: name, lines, scroll: 0, style: 0 });
        }
        let body = chips::body_of(content);
        match chips::unescape(body).strip_prefix('@').filter(|p| !p.contains('\n')) {
            Some(p) => self.open_source(name, Some(Attachment::of(PathBuf::from(p))), None),
            None => self.open_source(name, None, Some(body.to_string())),
        }
    }

    /// What the chip brings: its text in the window, or its file in the program of the system.
    pub(crate) fn open_chip(&mut self, label: &str) {
        let Some(chip) = self.composer().chips().iter().find(|c| c.label == label).cloned() else { return };
        let name = label.trim_matches([chips::OPEN, chips::CLOSE]).to_string();
        self.open_source(name, chip.attach, chip.context);
    }

    /// The window of a chip with its file or its text; the text shows without its escapes
    /// (CLAUDE-61).
    fn open_source(&mut self, name: String, attach: Option<Attachment>, context: Option<String>) {
        let style = usize::from(name.starts_with(CHAT_MARK));
        let lines = match (&attach, &context) {
            (Some(a), _) if !self.facts.exists(a.path()) => return self.chip_error(name, format!("{} is gone", a.path().display())),
            // A worker reads the file or opens it outside; `poll_chip` shows it (IO-1).
            (Some(a), _) => {
                let (path, image, opener) = (a.path().to_path_buf(), matches!(a, Attachment::Image(_)), self.opener);
                return self.chip_reads.start(move || (name, crate::app::facts::chip_file(&path, image, opener)));
            }
            (None, Some(ctx)) => context_lines(&chips::unescape(ctx)),
            (None, None) => Vec::new(),
        };
        self.composer_mut().view = Some(ChipView { label: name, lines, scroll: 0, style });
    }

    /// The worker read the file of a chip: its text in the window, or the program of the system
    /// opened it, or its error in the window (CLAUDE-52, CLAUDE-53).
    pub(crate) fn poll_chip(&mut self) {
        let Some((name, file)) = self.chip_reads.poll() else { return };
        let style = usize::from(name.starts_with(CHAT_MARK));
        match file {
            ChipFile::Text(t) => self.composer_mut().view = Some(ChipView { label: name, lines: t.lines().map(|l| (l.to_string(), false)).collect(), scroll: 0, style }),
            ChipFile::Outside(Ok(())) => self.flash(format!("{name} opens in the program of the system")),
            ChipFile::Outside(Err(e)) | ChipFile::Error(e) => self.chip_error(name, e),
        }
    }

    fn chip_error(&mut self, label: String, text: String) {
        self.composer_mut().view = Some(ChipView { label, lines: vec![(text, false)], scroll: 0, style: 2 });
    }

    /// The keys while the chip window is open: the scroll keys and `esc`; the others do nothing
    /// (CLAUDE-50, CLAUDE-51).
    pub(crate) fn key_chip_view(&mut self, k: KeyEvent) {
        let page = 10;
        let by = match k.code {
            KeyCode::Esc => return self.composer_mut().close_view(),
            KeyCode::Up => -1,
            KeyCode::Down => 1,
            KeyCode::PageUp => -page,
            KeyCode::PageDown => page,
            KeyCode::Home => isize::MIN / 2,
            KeyCode::End => isize::MAX / 2,
            _ => return,
        };
        self.composer_mut().scroll_view(by);
    }

    /// The open document of the question and the stamp of its text; None when the line of the
    /// open document does not show: all notes, a slash command, no document (CLAUDE-54).
    fn doc_id(&self) -> Option<(String, u64)> {
        let scope = self.scope();
        if scope == ALL_KEY || self.composer().input().text().starts_with('/') {
            return None;
        }
        if scope == NEW_KEY {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            self.writer.editor().lines.hash(&mut h);
            return Some((NEW_KEY.to_string(), h.finish()));
        }
        // The stamp comes from a worker; None while it is not known or the file is not there (IO-1).
        let p = self.files.current().map(|p| self.session_path(p))?;
        let stamp = self.facts.stamp.get(&p).flatten()?;
        Some((p.display().to_string(), stamp))
    }

    /// The line of the open document: Some(true) for `[x]`, Some(false) for `[ ]`, None for no line.
    pub fn doc_line(&self) -> Option<bool> {
        let (id, stamp) = self.doc_id()?;
        Some(self.composer().doc_selected(&id, stamp))
    }

    /// A click on the line of the open document (CLAUDE-55).
    pub(crate) fn toggle_doc(&mut self) {
        if let Some((id, stamp)) = self.doc_id() {
            self.composer_mut().toggle_doc(&id, stamp);
        }
    }

    /// Each tick: another document or another text selects the open document again (CLAUDE-56).
    pub(crate) fn doc_tick(&mut self) {
        if let Some(p) = self.files.current().map(|p| self.session_path(p)).filter(|_| self.scope() != NEW_KEY) {
            self.composer_mut().sync_doc_id(&p.display().to_string());
        }
        if let Some((id, stamp)) = self.doc_id() {
            self.composer_mut().sync_doc(&id, stamp);
        }
    }
}

#[cfg(test)]
#[path = "chip_view_tests.rs"]
mod tests;
