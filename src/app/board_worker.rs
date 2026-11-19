//! The journal of the board on a thread of its own (ADR 0021, ADR 0024): the lock of the journal
//! waits for other processes, a read takes up to 1 MB, and a compaction syncs. The app sends
//! operations, the thread does them in order, and the tick takes the answers (IO-1).
use crate::board::{Event, How, Journal, Line, proc_alive};
use std::sync::mpsc::{Receiver, Sender};

/// An operation on the journal.
pub(crate) enum Op {
    /// Read the lines after `offset` of the journal file `file`; from the start after a compaction.
    Poll { file: u64, offset: u64, repo: String },
    Append(Vec<Event>),
    /// Claim the delivery of `ids` to session `to` (BOARD-1).
    Claim { ids: Vec<String>, to: usize },
    /// The turn of session `to` that took `ids` ended (BOARD-9).
    Land { ids: Vec<String>, to: usize },
    /// The posts of this repository: id, pid, start of the process, and whether the lease expired
    /// (BOARD-30, BOARD-31).
    Leases(Vec<(String, u32, Option<u64>, bool)>),
}

/// An answer of the thread.
pub(crate) enum Answer {
    /// The new lines; `reset` when the state must fold the file from its start.
    Polled { file: u64, offset: u64, reset: bool, lines: Vec<Line>, beat: Option<u64> },
    /// The messages that session `to` won.
    Claimed { to: usize, won: Vec<String> },
}

pub(crate) struct BoardWorker {
    ops: Sender<Op>,
    answers: Receiver<Answer>,
}

impl BoardWorker {
    pub(crate) fn start(journal: Journal) -> BoardWorker {
        let (ops, inbox) = std::sync::mpsc::channel();
        let (outbox, answers) = std::sync::mpsc::channel();
        let _ = std::thread::Builder::new().name("zendo-board".into()).spawn(move || run(&journal, &inbox, &outbox));
        BoardWorker { ops, answers }
    }

    /// Hand `op` to the thread; returns at once.
    pub(crate) fn send(&self, op: Op) {
        let _ = self.ops.send(op);
    }

    /// The answers that came.
    pub(crate) fn answers(&self) -> Vec<Answer> {
        self.answers.try_iter().collect()
    }
}

fn run(j: &Journal, inbox: &Receiver<Op>, outbox: &Sender<Answer>) {
    for op in inbox {
        let answer = match op {
            Op::Poll { file, offset, repo } => Some(poll(j, file, offset, &repo)),
            Op::Append(ev) => {
                let _ = j.append(&ev);
                None
            }
            Op::Claim { ids, to } => Some(Answer::Claimed { to, won: ids.into_iter().filter(|id| j.claim(id, to, How::Delivered)).collect() }),
            Op::Land { ids, to } => {
                j.land(&ids, to);
                None
            }
            Op::Leases(posts) => {
                let ev: Vec<Event> = posts.into_iter().filter_map(|(id, pid, start, expired)| lease(id, proc_alive(pid, start), expired)).collect();
                if !ev.is_empty() {
                    let _ = j.append(&ev);
                }
                None
            }
        };
        if let Some(a) = answer {
            if outbox.send(a).is_err() {
                return;
            }
            crate::worker::wake().notify_one();
        }
    }
}

/// The lease of a post ends with its process, and goes on when the process runs again.
fn lease(post: String, alive: bool, expired: bool) -> Option<Event> {
    match (alive, expired) {
        (false, false) => Some(Event::Expired { post }),
        (true, true) => Some(Event::Renewed { post }),
        _ => None,
    }
}

fn poll(j: &Journal, file: u64, offset: u64, repo: &str) -> Answer {
    let (now, len) = j.stamp();
    let reset = now != file || len < offset;
    let (lines, offset) = j.read_from(if reset { 0 } else { offset });
    Answer::Polled { file: now, offset, reset, lines, beat: j.last_beat(repo) }
}
