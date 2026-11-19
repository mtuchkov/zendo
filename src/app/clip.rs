//! The clipboard of the app: the paste key reads the system clipboard on a worker, and the answer
//! lands where the user pasted (IO-1, IO-5, IO-6, ADR 0024).
use super::*;
use crate::text::clipboard::Clip;
use crate::worker::Task;
use std::sync::Arc;

/// How long a read of the clipboard may take (IO-5).
const LIMIT: Duration = Duration::from_secs(2);
const NO_ANSWER: &str = "the clipboard does not answer";

/// Where a paste goes: the place that had the keys when the user pressed the paste key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasteTo {
    Chat,
    Write,
    Field,
}

type Read = Arc<dyn Fn() -> Clip + Send + Sync>;
type Copy = Arc<dyn Fn(&str) + Send + Sync>;

/// The system clipboard; a test puts one here that reads and copies in memory.
pub struct Clipboard {
    read: Read,
    copy: Copy,
    task: Task<Clip>,
    to: Option<PasteTo>,
}

impl Default for Clipboard {
    fn default() -> Self {
        Clipboard::with(crate::text::clipboard::read, crate::text::clipboard::copy)
    }
}

impl Clipboard {
    pub fn with(read: impl Fn() -> Clip + Send + Sync + 'static, copy: impl Fn(&str) + Send + Sync + 'static) -> Clipboard {
        Clipboard { read: Arc::new(read), copy: Arc::new(copy), task: Task::new(LIMIT), to: None }
    }

    /// Copy `text`; the system clipboard gets it on a thread of its own.
    pub fn copy(&self, text: &str) {
        (self.copy)(text);
    }

    fn start(&mut self, to: PasteTo) {
        let read = self.read.clone();
        self.to = Some(to);
        self.task.start(move || read());
    }

    /// The answer of the read, once: what the clipboard holds, or that it did not answer in time.
    fn take(&mut self) -> Option<(PasteTo, Clip)> {
        if self.task.overdue() {
            return self.to.take().map(|to| (to, Clip::Failed(NO_ANSWER.into())));
        }
        let clip = self.task.poll()?;
        Some((self.to.take()?, clip))
    }
}

impl App {
    /// A worker has an answer: take the answers of the tasks of the app (ADR 0024).
    pub fn poll_tasks(&mut self) {
        self.poll_clipboard();
    }

    /// The paste key: read the system clipboard on a worker; the answer comes in `poll_clipboard`.
    pub(crate) fn paste_from_clipboard(&mut self, to: PasteTo) {
        self.clipboard.start(to);
    }

    /// Put the answer of the clipboard where the user pasted. Without one, the last copy of zendo
    /// goes, else the user learns why nothing came (FAIL-1).
    pub fn poll_clipboard(&mut self) {
        let Some((to, clip)) = self.clipboard.take() else { return };
        let failed = match clip {
            Clip::Image(p) if to == PasteTo::Chat => return self.attach_pasted_image(p),
            Clip::Text(t) => return self.paste_text(to, &t),
            Clip::Failed(e) => e,
            Clip::Image(_) | Clip::Empty => "the clipboard is empty, or zendo cannot read it here".into(),
        };
        match self.pointer.last_copy() {
            Some(t) => self.paste_text(to, &t),
            None => self.flash(failed),
        }
    }

    fn paste_text(&mut self, to: PasteTo, text: &str) {
        match to {
            PasteTo::Chat => self.paste_into_chat(text),
            PasteTo::Write if self.mode == Mode::Write => {
                self.writer.editor_mut().type_str(text);
                self.writer.touch();
            }
            PasteTo::Write => {}
            PasteTo::Field => drop(self.paste_into_field(text)),
        }
    }
}
