//! The background sync of the notes repo.
use super::*;

/// Background sync of one location folder.
pub struct Syncer {
    pub repo: PathBuf,
    pub dir: PathBuf,
    message: String,
    state: Arc<Mutex<State>>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Syncer {
    pub fn start(repo: PathBuf, dir: PathBuf, machine: &str, every: Duration) -> Self {
        let rel = dir.strip_prefix(repo.join(machine)).unwrap_or(&dir).display().to_string();
        let message = format!("notes: {machine} {rel}");
        let state = Arc::new(Mutex::new(State::Starting));
        let stop = Arc::new(AtomicBool::new(false));
        let (r, d, m, s, st) = (repo.clone(), dir.clone(), message.clone(), state.clone(), stop.clone());
        let thread = std::thread::spawn(move || {
            let set = |v: State| *s.lock().unwrap() = v;
            set(pull_push(&r));
            let mut next = Instant::now() + every;
            while !st.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(200));
                if Instant::now() < next {
                    continue;
                }
                next = Instant::now() + every;
                if matches!(*s.lock().unwrap(), State::Conflict(_)) {
                    continue;
                }
                match commit(&r, &d, &m) {
                    Err(e) => set(State::Failed(e)),
                    Ok(made) => {
                        if made || ahead(&r) && !matches!(*s.lock().unwrap(), State::Local) {
                            set(pull_push(&r));
                        }
                    }
                }
            }
        });
        Self { repo, dir, message, state, stop, thread: Some(thread) }
    }

    pub fn state(&self) -> State {
        self.state.lock().unwrap().clone()
    }

    /// At exit: stop the loop, commit what is left, and push. The push is bounded, so the exit is quick;
    /// what does not get out goes with the next start.
    pub fn finish(mut self) -> State {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
        if matches!(self.state(), State::Conflict(_)) {
            return self.state();
        }
        match commit(&self.repo, &self.dir, &self.message) {
            Ok(_) if ahead(&self.repo) => pull_push(&self.repo),
            Ok(_) => self.state(),
            Err(e) => State::Failed(e),
        }
    }
}
