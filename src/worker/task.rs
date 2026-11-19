//! A task: one IO operation on a worker thread, with a time limit; the UI thread polls its answer
//! (IO-1, IO-5, IO-6).
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::time::{Duration, Instant};

/// The answer of the IO operation that runs now, if any. A new start drops the older answer.
pub struct Task<T> {
    answer: Option<Receiver<T>>,
    started: Option<Instant>,
    limit: Duration,
}

impl<T: Send + 'static> Task<T> {
    /// A task whose operation may take up to `limit`.
    pub fn new(limit: Duration) -> Task<T> {
        Task { answer: None, started: None, limit }
    }

    /// Run `job` on a worker thread. Returns at once; the answer of an older start is dropped. The
    /// answer wakes the main loop (`worker::wake`).
    pub fn start(&mut self, job: impl FnOnce() -> T + Send + 'static) {
        let (tx, rx) = mpsc::channel();
        let spawned = std::thread::Builder::new().name("zendo-task".into()).spawn(move || {
            if tx.send(job()).is_ok() {
                super::wake().notify_one();
            }
        });
        self.answer = spawned.is_ok().then_some(rx);
        self.started = Some(Instant::now());
    }

    /// The answer, once, when it came.
    pub fn poll(&mut self) -> Option<T> {
        match self.answer.as_ref()?.try_recv() {
            Ok(a) => {
                self.stop();
                Some(a)
            }
            Err(TryRecvError::Disconnected) => {
                self.stop();
                None
            }
            Err(TryRecvError::Empty) => None,
        }
    }

    /// An operation runs, and zendo waits for its answer.
    pub fn running(&self) -> bool {
        self.answer.is_some()
    }

    /// True once when the operation ran over its limit: zendo stops waiting for it, and its late
    /// answer is dropped (IO-5).
    pub fn overdue(&mut self) -> bool {
        let over = self.running() && self.started.is_some_and(|t| t.elapsed() > self.limit);
        if over {
            self.stop();
        }
        over
    }

    fn stop(&mut self) {
        self.answer = None;
        self.started = None;
    }
}

#[cfg(test)]
#[path = "task_tests.rs"]
mod tests;
