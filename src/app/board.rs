//! The message board in the zendo process (BOARD, ADR 0021): it reads the journal, delivers the
//! messages to the sessions, lands them at the end of a turn, beats, and keeps the leases of the
//! posts.
use super::board_worker::{Answer, BoardWorker, Op};
use super::*;
use crate::board::{Beat, Event, Journal, SessionInfo, State, now_ms};
use std::collections::{HashMap, HashSet};

/// The request ids of the questions of the board in a chat: above those of the agent.
pub(crate) const BOARD_REQUEST: u64 = 1 << 62;

#[derive(Default)]
pub struct BoardLink {
    pub journal: Option<Journal>,
    pub repo: String,
    offset: u64,
    /// The id of the journal file that `offset` is in: a compaction makes a new one.
    file: u64,
    pub state: State,
    polled: Option<Instant>,
    beat: Option<Instant>,
    /// The turns in a row that the board started without a question of the user (BOARD-37).
    pub turns: usize,
    pub(crate) held_shown: bool,
    /// The tasks without a target whose question is in a chat, by request id (BOARD-10).
    pub(crate) choices: HashMap<u64, String>,
    pub(crate) asked: HashSet<String>,
    /// The watched messages that the chat shows already (BOARD-21).
    pub(crate) shown: HashSet<String>,
    /// The agent session of the thread of each session, to see a new thread (BOARD-7, BOARD-15).
    threads: Vec<Option<String>>,
    /// For each session, the posts that it saw, and whether it saw them expired (BOARD-17).
    pub(crate) seen: HashMap<usize, HashMap<String, bool>>,
    /// The thread that reads and writes the journal (IO-1).
    worker: Option<BoardWorker>,
    /// A poll waits for its answer.
    polling: bool,
    /// The sessions whose claim waits for its answer.
    claiming: HashSet<usize>,
    /// The newest heartbeat of this repository, from the last poll (BOARD-17).
    last_beat: Option<u64>,
}

impl BoardLink {
    /// The board in the folder `dir`, for the repository at `root`.
    pub fn open(dir: impl Into<std::path::PathBuf>, root: &Path) -> BoardLink {
        let journal = Journal::at(dir);
        let worker = Some(BoardWorker::start(journal.clone()));
        BoardLink { journal: Some(journal), worker, repo: crate::board::repo_of(root), threads: vec![None; SESSIONS], ..Default::default() }
    }

    /// Fold the new lines of the journal into the state. After a compaction, the state folds the
    /// new file from its start (ADR 0021).
    /// The thread reads them; `take` folds the answer (IO-1).
    pub(crate) fn poll(&mut self) {
        let Some(w) = self.worker.as_ref().filter(|_| !self.polling) else { return };
        w.send(Op::Poll { file: self.file, offset: self.offset, repo: self.repo.clone() });
        self.polling = true;
    }

    /// Append `ev` on the thread, then read the journal again.
    pub(crate) fn append(&mut self, ev: &[Event]) {
        if let Some(w) = self.worker.as_ref().filter(|_| !ev.is_empty()) {
            w.send(Op::Append(ev.to_vec()));
        }
        self.poll();
    }

    /// Hand `op` to the thread.
    pub(crate) fn send(&self, op: Op) {
        if let Some(w) = &self.worker {
            w.send(op);
        }
    }

    /// A claim of session `r` waits for its answer.
    pub(crate) fn claiming(&self, r: usize) -> bool {
        self.claiming.contains(&r)
    }

    /// Claim `ids` for session `r` on the thread (BOARD-1).
    pub(crate) fn claim(&mut self, ids: Vec<String>, r: usize) {
        self.send(Op::Claim { ids, to: r });
        self.claiming.insert(r);
    }

    /// Fold the polls that came; the claims that came, for the app to deliver.
    fn take(&mut self) -> Vec<(usize, Vec<String>)> {
        let answers = self.worker.as_ref().map(BoardWorker::answers).unwrap_or_default();
        let mut won = Vec::new();
        for a in answers {
            match a {
                Answer::Polled { file, offset, reset, lines, beat } => {
                    self.polling = false;
                    if reset {
                        self.state = State::default();
                    }
                    (self.file, self.offset, self.last_beat) = (file, offset, beat);
                    lines.into_iter().for_each(|l| self.state.apply(l));
                }
                Answer::Claimed { to, won: ids } => {
                    self.claiming.remove(&to);
                    won.push((to, ids));
                }
            }
        }
        won
    }
}

impl App {
    /// The agent of session `n` runs: a thread with an agent session, or Claude Code (BOARD-3).
    pub(crate) fn board_running(&self, n: usize) -> bool {
        self.cli.running(n) || self.threads.get(&session_key(n)).is_some_and(|t| t.session.is_some())
    }

    /// Each tick: read the journal, deliver, ask the user to choose; each second: beat, see new
    /// threads, keep the leases.
    pub fn board_tick(&mut self) {
        if self.board.journal.is_none() || self.board.polled.is_some_and(|t| t.elapsed() < Duration::from_millis(250)) {
            return;
        }
        self.board.polled = Some(Instant::now());
        self.board_answers();
        self.board.poll();
        self.board_deliver();
        self.board_watched();
        self.board_choose();
        if self.board.beat.is_some_and(|t| t.elapsed() < Duration::from_secs(1)) {
            return;
        }
        self.board.beat = Some(Instant::now());
        self.board_beat();
        self.board_threads();
        self.board_leases();
    }

    /// Fold the answers of the journal; deliver what the claims won (BOARD-1, BOARD-2).
    pub(crate) fn board_answers(&mut self) {
        for (r, ids) in self.board.take() {
            let won: Vec<crate::board::Msg> = ids.iter().filter_map(|id| self.board.state.msg(id).cloned()).collect();
            if !won.is_empty() {
                self.board.turns += 1;
                self.board_send(r, &won);
            }
        }
    }

    /// The heartbeat with the state of each session (BOARD-10, BOARD-22).
    fn board_beat(&mut self) {
        let sessions = (0..SESSIONS).map(|n| SessionInfo { id: n, name: self.session_name(n).unwrap_or_default(), state: self.board_state(n).into() }).collect();
        let beat = Beat { pid: std::process::id(), repo: self.board.repo.clone(), at: now_ms(), sessions };
        // On the writer, without the sync: a heartbeat lives for seconds (IO-1, IO-3).
        if let Some(j) = &self.board.journal {
            self.disk.write(&j.beat_path(beat.pid), serde_json::to_vec(&beat).unwrap_or_default(), crate::worker::Sync::Own);
        }
    }

    pub(crate) fn board_state_name(&self, n: usize) -> &'static str {
        self.board_state(n)
    }

    fn board_state(&self, n: usize) -> &'static str {
        let t = self.threads.get(&session_key(n));
        match () {
            _ if self.cli.running(n) => "claude code",
            _ if t.is_some_and(|t| t.busy.is_some()) => "works",
            _ if t.is_some_and(|t| t.asking().is_some()) => "waits",
            _ if t.is_some_and(|t| t.session.is_some()) => "idle",
            _ => "no thread",
        }
    }

    /// zendo quits: no heartbeat stays behind, so the watches end (BOARD-22).
    pub fn board_quit(&mut self) {
        if let Some(j) = &self.board.journal {
            j.unbeat(std::process::id());
        }
    }

    /// A new thread in a session ends its channels and its posts (BOARD-7, BOARD-15).
    fn board_threads(&mut self) {
        for n in 0..SESSIONS {
            let now = self.threads.get(&session_key(n)).and_then(|t| t.session.clone());
            let before = self.board.threads.get(n).cloned().flatten();
            if before.is_some() && now.is_some() && now != before {
                let repo = self.board.repo.clone();
                self.board.append(&[Event::ThreadEnd { repo, session: n }]);
            }
            if now.is_some() && n < self.board.threads.len() {
                self.board.threads[n] = now;
            }
        }
    }

    /// The lease of a post ends with its process, and goes on when the process runs again after a
    /// restart (BOARD-30, BOARD-31).
    /// The thread looks at the processes in `/proc` and appends the changes (IO-1).
    fn board_leases(&mut self) {
        let repo = self.board.repo.clone();
        let posts: Vec<(String, u32, Option<u64>, bool)> = self.board.state.live_posts().filter(|p| p.post.repo == repo).filter_map(|p| Some((p.post.id.clone(), p.post.pid?, p.post.start, p.expired()))).collect();
        if !posts.is_empty() {
            self.board.send(Op::Leases(posts));
            self.board.poll();
        }
    }

    /// The turn of session `n` ended: the initiations that it took land (BOARD-9).
    pub(crate) fn board_turn_ended(&mut self, n: usize) {
        let ids = self.board.state.taken(&self.board.repo, n);
        self.board.send(Op::Land { ids, to: n });
        self.board.poll();
    }

    /// The user asked a question: the fuse starts again (BOARD-37).
    pub(crate) fn board_user_asked(&mut self) {
        self.board.turns = 0;
        self.board.held_shown = false;
    }

    /// The posts that session `n` did not see yet, and the posts whose lease expired since it saw
    /// them, as context of its next question (BOARD-17).
    pub fn board_posts_context(&mut self, n: usize) -> String {
        let now = now_ms();
        let beat = self.board.last_beat.or(Some(now));
        let seen = self.board.seen.entry(n).or_default();
        let mut lines = Vec::new();
        for p in self.board.state.live_posts() {
            let lease = p.lease(now, if p.post.repo == self.board.repo { beat } else { Some(now) });
            let new = match seen.get(&p.post.id) {
                None => true,
                Some(expired) => !*expired && !lease.live,
            };
            if new {
                seen.insert(p.post.id.clone(), !lease.live);
                let state = if lease.live { "lease live".to_string() } else { format!("lease expired {} s ago", now.saturating_sub(lease.since) / 1000) };
                lines.push(format!("- {} · post {} · session {} of {}: {} ({state})", p.post.resource, p.post.id, p.post.session, p.post.repo, p.post.text));
            }
        }
        lines.extend(self.board_tombstones(n));
        if lines.is_empty() { String::new() } else { format!("Board posts about shared resources of the machine:\n{}\n", lines.join("\n")) }
    }

    /// The posts of session `n` that another session tombstoned, once (BOARD-35).
    fn board_tombstones(&mut self, n: usize) -> Vec<String> {
        let seen = self.board.seen.entry(n).or_default();
        let mut lines = Vec::new();
        for p in self.board.state.history().filter(|p| p.post.repo == self.board.repo && p.post.session == n) {
            let Some(crate::board::End::Tombstone { repo, session }) = &p.end else { continue };
            if seen.insert(format!("tombstone {}", p.post.id), true).is_none() {
                lines.push(format!("- your post {} about {} was tombstoned by session {session} of {repo}: post again if you still use it", p.post.id, p.post.resource));
            }
        }
        lines
    }
}
