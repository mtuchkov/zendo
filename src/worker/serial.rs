//! Jobs that change files of the user, one at a time and in order, on a thread of their own: the
//! config, the keymap, the title of a note, a new file, a rename (IO-1, ADR 0024). Two changes of
//! one file never run at once.
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

type Job = Box<dyn FnOnce() -> Result<(), String> + Send>;

pub struct Serial {
    jobs: Sender<(u64, Job)>,
    answers: Receiver<(u64, Result<(), String>)>,
    next: u64,
    /// The jobs that did not answer yet.
    open: usize,
}

impl Default for Serial {
    fn default() -> Self {
        let (jobs, inbox) = std::sync::mpsc::channel::<(u64, Job)>();
        let (outbox, answers) = std::sync::mpsc::channel();
        let _ = std::thread::Builder::new().name("zendo-serial".into()).spawn(move || {
            for (id, job) in inbox {
                if outbox.send((id, job())).is_err() {
                    return;
                }
                super::wake().notify_one();
            }
        });
        Serial { jobs, answers, next: 0, open: 0 }
    }
}

impl Serial {
    /// Queue `job`; returns its id at once.
    pub fn run(&mut self, job: impl FnOnce() -> Result<(), String> + Send + 'static) -> u64 {
        self.next += 1;
        self.open += 1;
        let _ = self.jobs.send((self.next, Box::new(job)));
        self.next
    }

    /// The answers since the last call, each once.
    pub fn answers(&mut self) -> Vec<(u64, Result<(), String>)> {
        let got: Vec<_> = self.answers.try_iter().collect();
        self.open = self.open.saturating_sub(got.len());
        got
    }

    /// Jobs wait for their answer.
    pub fn busy(&self) -> bool {
        self.open > 0
    }

    /// After the last frame: wait at most `limit` for the jobs (IO-7); their answers stay.
    pub fn flush(&mut self, limit: Duration) -> Vec<(u64, Result<(), String>)> {
        let end = Instant::now() + limit;
        let mut got = Vec::new();
        while self.open > 0 && Instant::now() < end {
            got.extend(self.answers());
            std::thread::sleep(Duration::from_millis(2));
        }
        got
    }
}

#[cfg(test)]
#[path = "serial_tests.rs"]
mod tests;
