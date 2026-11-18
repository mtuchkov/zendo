//! The text search of the find panel on a thread of its own: the keys never wait for it (FIND-21).
//! The thread reads the files of the repository and the text of the markdown files once, when the
//! panel opens, and then searches the newest query that the panel sends.
use super::FindItem;
use crate::docs::notes::{self, Note};
use crate::docs::search::{self, Group};
use crate::text::code;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Notify;

/// What the thread sends to the panel.
pub(crate) enum Found {
    /// The files of the repository, from the root, and the notes of the notes repo by their
    /// absolute path (FIND-18).
    Files(Vec<PathBuf>),
    /// The markdown files whose text has the query that had this number.
    Text(u64, Vec<FindItem>),
}

/// The thread of the text search. It stops when the panel drops it.
pub(crate) struct Worker {
    jobs: Sender<(u64, String)>,
    found: Receiver<Found>,
}

impl Worker {
    /// Start the thread; `wake` draws the screen when it sends a result.
    pub(crate) fn start(root: PathBuf, notes_dir: Option<PathBuf>, wake: Arc<Notify>) -> Self {
        let (jobs, job_rx) = mpsc::channel::<(u64, String)>();
        let (tx, found) = mpsc::channel();
        std::thread::spawn(move || {
            let files = repo_files(&root, notes_dir.as_deref());
            let docs: Vec<Note> = files.iter().filter(|f| !code::is_code(f)).filter_map(|f| notes::load(&root.join(f)).ok()).collect();
            let send = |f: Found| {
                let sent = tx.send(f).is_ok();
                wake.notify_one();
                sent
            };
            if !send(Found::Files(files)) {
                return;
            }
            while let Ok(mut job) = job_rx.recv() {
                // Only the newest query counts: the user typed on while the thread searched.
                job = job_rx.try_iter().last().unwrap_or(job);
                if !send(Found::Text(job.0, items_of(&docs, search::search(&docs, &job.1)))) {
                    return;
                }
            }
        });
        Self { jobs, found }
    }

    /// Search the text for this query; the result comes back with its number.
    pub(crate) fn ask(&self, n: u64, query: &str) {
        let _ = self.jobs.send((n, query.to_string()));
    }

    /// A result that is here, without a wait.
    pub(crate) fn try_take(&self) -> Option<Found> {
        self.found.try_recv().ok()
    }

    /// The next result; `None` when the thread stopped.
    pub(crate) fn take(&self) -> Option<Found> {
        self.found.recv().ok()
    }
}

/// The text files of the repository, then the notes of the notes repo (FIND-16, FIND-18).
fn repo_files(root: &Path, notes_dir: Option<&Path>) -> Vec<PathBuf> {
    let mut files = crate::app::at::repo_files(root);
    if let Some(dir) = notes_dir {
        let md = crate::app::at::repo_files(dir).into_iter().filter(|f| f.extension().is_some_and(|x| x == "md"));
        files.extend(md.map(|f| dir.join(f)));
    }
    files
}

/// The rows of the matches in the text.
pub(crate) fn items_of(docs: &[Note], groups: Vec<Group>) -> Vec<FindItem> {
    groups.into_iter().map(|g| FindItem { path: docs[g.note].path.clone(), title: docs[g.note].title.clone(), at: None, snippets: g.snippets }).collect()
}
