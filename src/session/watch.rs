//! The files that the hooks of a Claude Code session write, read on a thread of their own: its
//! state and its messages (SESSIONS-17, CLIMSG-1, IO-1). The thread sends only what changed.
use super::messages::{Inbox, Message};
use super::{State, hooks};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::time::Duration;

/// How often the thread reads the files: often, so that the start of a session comes before its
/// first prompt (CLIMSG-10).
const EVERY: Duration = Duration::from_millis(20);

/// What the hooks reported since the last look.
pub(super) struct Seen {
    pub state: Option<State>,
    pub messages: Vec<Message>,
}

pub(super) struct Watch {
    seen: Receiver<Seen>,
    stop: Arc<AtomicBool>,
}

impl Watch {
    /// Watch the state file and the messages file of a session; the thread removes the state file
    /// when the session goes.
    pub(super) fn start(state_file: PathBuf, messages: PathBuf) -> Watch {
        let (tx, seen) = std::sync::mpsc::channel();
        let stop = Arc::new(AtomicBool::new(false));
        let stopped = stop.clone();
        let _ = std::thread::Builder::new().name("zendo-hooks".into()).spawn(move || {
            let mut inbox = Inbox::new(messages);
            let mut last = None;
            while !stopped.load(Ordering::Relaxed) {
                let state = hooks::read(&state_file).filter(|s| last != Some(*s));
                let messages = inbox.take();
                if state.is_some() || !messages.is_empty() {
                    last = state.or(last);
                    if tx.send(Seen { state, messages }).is_err() {
                        break;
                    }
                    crate::worker::wake().notify_one();
                }
                std::thread::sleep(EVERY);
            }
            let _ = std::fs::remove_file(&state_file);
        });
        Watch { seen, stop }
    }

    /// The newest state and the messages that came since the last call.
    pub(super) fn take(&self) -> Seen {
        let mut all = Seen { state: None, messages: Vec::new() };
        for s in self.seen.try_iter() {
            all.state = s.state.or(all.state);
            all.messages.extend(s.messages);
        }
        all
    }
}

impl Drop for Watch {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
