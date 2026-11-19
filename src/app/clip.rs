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

    /// How long a paste waits for the clipboard.
    pub fn waited(&self) -> Option<Duration> {
        self.task.waited()
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
        self.files.complete_waiting(&self.cfg.root);
        self.poll_jobs();
        self.poll_saves();
        self.disk_answer();
        self.poll_reload();
        self.poll_peek();
        self.poll_marks();
        if self.files.poll_tree() {
            self.mark_tree();
        }
    }

    /// The longest wait of a read that the user waits for: the notes, a paste, a version (IO-6).
    pub fn reading(&self) -> Option<Duration> {
        [self.files.reload_waited(), self.clipboard.waited(), self.peeks.waited()].into_iter().flatten().max()
    }

    /// Run `job` on `serial`; `then` gets its answer on the UI thread (IO-1).
    pub(crate) fn in_background(&mut self, job: impl FnOnce() -> Result<(), String> + Send + 'static, then: impl FnOnce(&mut App, Result<(), String>) + 'static) {
        let id = self.serial.run(job);
        self.after_job.insert(id, Box::new(then));
    }

    /// The answers of the jobs: what follows each runs.
    pub(crate) fn poll_jobs(&mut self) {
        for (id, answer) in self.serial.answers() {
            if let Some(then) = self.after_job.remove(&id) {
                then(self, answer);
            }
        }
    }

    /// For tests: wait at most 5 s until the tasks of the app answered, and take the answers.
    pub fn settle_tasks(&mut self) {
        let end = Instant::now() + Duration::from_secs(5);
        while (self.peeks.running() || self.marks.running() || self.recaps.running() || self.worktree_diff.running() || self.files.disk_busy() || self.saving.is_some() || self.serial.busy()) && Instant::now() < end {
            std::thread::sleep(Duration::from_millis(2));
            self.poll_tasks();
        }
        self.poll_tasks();
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
