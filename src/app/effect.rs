//! What a sub-state asks the app to do in the other sub-states (ADR 0008). A sub-state changes its
//! own fields directly; an effect is only for what it cannot reach.
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

type Job = Box<dyn FnOnce() -> Option<Effect> + Send>;

/// A change of files that runs on a thread of the app; its answer is the effect to apply (IO-1).
#[derive(Clone)]
pub struct IoJob(Arc<Mutex<Option<Job>>>);

impl IoJob {
    pub fn new(job: impl FnOnce() -> Option<Effect> + Send + 'static) -> IoJob {
        IoJob(Arc::new(Mutex::new(Some(Box::new(job)))))
    }

    /// The job, once.
    pub fn take(&self) -> Option<Job> {
        self.0.lock().ok()?.take()
    }

    /// Run the job here: for tests and for a caller off the UI thread.
    pub fn run(&self) -> Option<Effect> {
        self.take().and_then(|job| job())
    }
}

impl PartialEq for IoJob {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for IoJob {}

impl std::fmt::Debug for IoJob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("IoJob")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Show a message in the status bar.
    Flash(String),
    /// A file is gone: the thread and the session of its key go with it.
    Deleted(String),
    /// A file has a new path: its thread, its session and its place in the recent files follow.
    /// `done` is the word for the message: "renamed" or "moved".
    Moved { old: PathBuf, target: PathBuf, done: &'static str },
    /// A new file: it opens to write.
    MadeFile(PathBuf),
    /// A new folder: the tree shows it.
    MadeFolder(PathBuf),
    /// A change of files to run on a thread; its answer is the effect to apply (IO-1).
    Io(IoJob),
}

impl Effect {
    pub(crate) fn flash(text: impl Into<String>) -> Option<Effect> {
        Some(Effect::Flash(text.into()))
    }
}
