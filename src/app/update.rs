//! A newer zendo: the check that runs at the start, what it found, and the upgrade at the quit
//! (ADR 0008).
use tokio::sync::oneshot::{Receiver, error::TryRecvError};

#[derive(Default)]
pub struct Update {
    /// A newer commit on main (short sha), found on start.
    latest: Option<String>,
    rx: Option<Receiver<Option<String>>>,
    /// Quit, upgrade, and start again.
    now: bool,
}

impl Update {
    /// The check runs: its answer comes through `rx`.
    pub fn watch(&mut self, rx: Receiver<Option<String>>) {
        self.rx = Some(rx);
    }

    /// The answer of the check, once: the newer commit that it found.
    pub fn poll(&mut self) -> Option<String> {
        match self.rx.as_mut()?.try_recv() {
            Ok(found) => {
                self.rx = None;
                self.latest = found.clone().or(self.latest.take());
                found
            }
            Err(TryRecvError::Closed) => {
                self.rx = None;
                None
            }
            Err(TryRecvError::Empty) => None,
        }
    }

    pub fn latest(&self) -> Option<&String> {
        self.latest.as_ref()
    }

    /// Upgrade after the quit.
    pub fn upgrade(&mut self) {
        self.now = true;
    }

    pub fn upgrade_now(&self) -> bool {
        self.now
    }
}

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
