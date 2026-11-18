//! The background sync of the notes repo.
use super::*;

/// An index.lock of another git fails this many intervals in a row before the status shows it.
const QUIET_INDEX_FAILS: u32 = 2;
/// How long the exit waits for another zendo to finish its sync.
const EXIT_WAIT: Duration = Duration::from_secs(3);

/// The work of one interval for one location folder. The thread of a `Syncer` owns it.
pub struct Worker {
    repo: PathBuf,
    dir: PathBuf,
    message: String,
    lock: PathBuf,
    state: Arc<Mutex<State>>,
    index_fails: u32,
}

impl Worker {
    /// `lock` is the sync lock of the user: `lock_path()`, or a file of a test.
    pub fn new(repo: PathBuf, dir: PathBuf, machine: &str, lock: PathBuf) -> Self {
        let rel = dir.strip_prefix(repo.join(machine)).unwrap_or(&dir).display().to_string();
        let message = format!("notes: {machine} {rel}");
        Self { repo, dir, message, lock, state: Arc::new(Mutex::new(State::Starting)), index_fails: 0 }
    }

    pub fn state(&self) -> State {
        self.state.lock().unwrap().clone()
    }

    pub(super) fn set(&self, s: State) {
        *self.state.lock().unwrap() = s;
    }

    fn who(&self) -> String {
        self.dir.display().to_string()
    }

    /// One interval: commit, then pull and push when there is a reason to. A busy sync lock skips
    /// the interval (NOTESREP-15); an index.lock of another git shows only when it stays
    /// (NOTESREP-16); a commit that works clears an earlier failure (NOTESREP-17).
    pub fn step(&mut self) {
        let state = self.state();
        if matches!(state, State::Conflict(_)) {
            return;
        }
        let Some(_lock) = SyncLock::try_take(&self.lock, &self.who()) else {
            return;
        };
        match commit(&self.repo, &self.dir, &self.message) {
            Err(e) if e.contains("index.lock") && self.index_fails < QUIET_INDEX_FAILS => self.index_fails += 1,
            Err(e) => self.set(State::Failed(e)),
            Ok(made) => {
                self.index_fails = 0;
                let unsure = matches!(state, State::Starting | State::Failed(_));
                if made || unsure || ahead(&self.repo) && !matches!(state, State::Local) {
                    self.set(pull_push(&self.repo));
                }
            }
        }
    }

    /// At exit: commit what is left and push (NOTESREP-8). The wait for the lock and the push are
    /// bounded, so the exit is quick; what does not get out goes with the next start.
    pub fn last(self, wait: Duration) -> State {
        let state = self.state();
        if matches!(state, State::Conflict(_)) {
            return state;
        }
        let Some(_lock) = SyncLock::take_within(&self.lock, &self.who(), wait) else {
            return State::Busy(holder(&self.lock).unwrap_or_else(|| "pid unknown".into()));
        };
        match commit(&self.repo, &self.dir, &self.message) {
            Ok(_) if ahead(&self.repo) || matches!(state, State::Starting | State::Failed(_)) => pull_push(&self.repo),
            Ok(_) => state,
            Err(e) => State::Failed(e),
        }
    }
}

/// Background sync of one location folder.
pub struct Syncer {
    state: Arc<Mutex<State>>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<Worker>>,
}

impl Syncer {
    pub fn start(mut worker: Worker, every: Duration) -> Self {
        let state = worker.state.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let st = stop.clone();
        let thread = std::thread::spawn(move || {
            worker.step();
            let mut next = Instant::now() + every;
            while !st.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(200));
                if Instant::now() >= next {
                    next = Instant::now() + every;
                    worker.step();
                }
            }
            worker
        });
        Self { state, stop, thread: Some(thread) }
    }

    /// The sync of a location folder with the sync lock of the user (NOTESREP-15).
    pub fn of_user(repo: PathBuf, dir: PathBuf, machine: &str, every: Duration) -> Self {
        Self::start(Worker::new(repo, dir, machine, lock_path()), every)
    }

    pub fn state(&self) -> State {
        self.state.lock().unwrap().clone()
    }

    /// At exit: stop the loop, then the last commit and push of the worker.
    pub fn finish(mut self) -> State {
        self.stop.store(true, Ordering::Relaxed);
        match self.thread.take().map(|t| t.join()) {
            Some(Ok(worker)) => worker.last(EXIT_WAIT),
            _ => self.state(),
        }
    }
}
