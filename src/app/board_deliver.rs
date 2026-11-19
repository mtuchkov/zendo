//! The delivery of the messages of the board to the sessions, and the choice of the session of a
//! task without a target (BOARD-1 to BOARD-3, BOARD-6, BOARD-10, BOARD-11, BOARD-37).
use super::board::BOARD_REQUEST;
use super::*;
use crate::agent::{Pick, Question};
use crate::board::{Event, FUSE, How, Msg, label, tagged};

impl App {
    /// Each session that waits for nothing gets its waiting messages, squashed into one question.
    pub(crate) fn board_deliver(&mut self) {
        // Offline, no agent runs: the messages wait for a zendo that can deliver them.
        if self.cfg.offline {
            return;
        }
        for r in 0..SESSIONS {
            let key = session_key(r);
            let busy = self.threads.get(&key).is_some_and(|t| t.busy.is_some() || t.asking().is_some());
            if busy && !self.cli.running(r) {
                continue;
            }
            let running = |n: usize| self.board_running(n);
            let ids: Vec<String> = self.board.state.pending(&self.board.repo, r, &running).iter().map(|m| m.id.clone()).collect();
            if ids.is_empty() || self.board_fused(r) {
                continue;
            }
            let Some(j) = self.board.journal.clone() else { return };
            let won: Vec<String> = ids.into_iter().filter(|id| j.claim(id, r, How::Delivered)).collect();
            self.board.append(&[]);
            let won: Vec<Msg> = won.iter().filter_map(|id| self.board.state.msg(id).cloned()).collect();
            if !won.is_empty() {
                self.board.turns += 1;
                self.board_send(r, &won);
            }
        }
    }

    /// The fuse holds the messages after FUSE turns in a row that the board started; it tells the
    /// user one time (BOARD-37).
    fn board_fused(&mut self, r: usize) -> bool {
        if self.board.turns < FUSE {
            return false;
        }
        if !self.board.held_shown {
            self.board.held_shown = true;
            let key = session_key(r);
            self.thread(&key).push(Item::Info(format!("the board started {FUSE} turns in a row: messages from other sessions wait until you ask a question")));
            self.turn_ended(r);
        }
        true
    }

    /// Send `msgs` to session `r` as one question: to Claude Code while it runs, else to the
    /// current thread of the chat, which opens if no agent runs (BOARD-2, BOARD-8, BOARD-9).
    fn board_send(&mut self, r: usize, msgs: &[Msg]) {
        let shown = msgs.iter().map(|m| label(m, &self.session_name(m.from).unwrap_or_default())).collect::<Vec<_>>().join("\n");
        let tags = msgs.iter().map(tagged).collect::<Vec<_>>().join("\n\n");
        let key = session_key(r);
        if self.cli.running(r) {
            return self.board_send_cli(r, &shown, &tags);
        }
        self.ensure_agent();
        let mut text = String::new();
        if !self.threads.get(&key).is_some_and(|t| t.primed) {
            text.push_str(&state::prompt(&self.cfg.root));
            text.push_str("\n\n");
        }
        text.push_str(&self.board_posts_context(r));
        text.push_str(&tags);
        let n = self.threads.number();
        self.dispatch(key, Outgoing { question: shown, text, attach: Vec::new(), sent: None, command: false, n });
        if !self.sees(r) {
            self.attention.start(r);
        }
    }

    /// The messages go to Claude Code, which queues them while it works (BOARD-8).
    fn board_send_cli(&mut self, r: usize, shown: &str, text: &str) {
        let bracketed = self.cli.with_screen(r, |s| s.bracketed_paste()).unwrap_or(false);
        let body = if bracketed { text.to_string() } else { text.replace('\n', "\x1b\r") };
        self.cli.write(r, &crate::session::keys::paste(&body, bracketed));
        self.cli.write(r, b"\r");
        let t = self.thread(&session_key(r));
        t.sent_to_cli.push(crate::session::plain(text));
        t.push(Item::User(shown.to_string()));
    }

    /// The messages that a watch printed show in the chat of their session, once (BOARD-21).
    pub(crate) fn board_watched(&mut self) {
        let repo = self.board.repo.clone();
        for r in 0..SESSIONS {
            let new: Vec<Msg> = self.board.state.taken_as(&repo, r, How::Watched).into_iter().filter(|m| !self.board.shown.contains(&m.id)).cloned().collect();
            for m in new {
                self.board.shown.insert(m.id.clone());
                let shown = format!("{} · watched", label(&m, &self.session_name(m.from).unwrap_or_default()));
                self.thread(&session_key(r)).push(Item::User(shown));
            }
        }
    }

    /// A task without a target from a session of this repository: its chat asks the user which
    /// session takes it (BOARD-10).
    pub(crate) fn board_choose(&mut self) {
        let repo = self.board.repo.clone();
        let open: Vec<Msg> = self.board.state.msgs.iter().map(|(_, m)| m).filter(|m| m.repo == repo && m.to.is_none() && self.board.state.choice(&m.id).is_none() && !self.board.asked.contains(&m.id)).cloned().collect();
        for m in open {
            self.board.asked.insert(m.id.clone());
            let request = BOARD_REQUEST + self.board.asked.len() as u64;
            self.board.choices.insert(request, m.id.clone());
            let choices = (0..SESSIONS).filter(|n| *n != m.from).map(|n| (n.to_string(), self.board_choice_title(n))).collect();
            let q = Question { field: "session".into(), text: format!("Session {} hands off a task. Which session takes it?\n\n{}", m.from, m.text), choices, many: false, other: None };
            self.thread(&session_key(m.from)).push(Item::Question(Box::new(Ask::new(request, vec![q]))));
            self.turn_ended(m.from);
        }
    }

    fn board_choice_title(&self, n: usize) -> String {
        let name = self.session_name(n).map(|s| format!(" · {s}")).unwrap_or_default();
        format!("session {n}{name} · {}", self.board_state_name(n))
    }

    /// The answer to a question of the board: the chosen session, or `esc`. True when `request`
    /// was a question of the board (BOARD-10, BOARD-12).
    pub(crate) fn board_reply(&mut self, request: u64, picks: Option<&[Pick]>) -> bool {
        let Some(msg) = self.board.choices.remove(&request) else { return request >= BOARD_REQUEST };
        let to = picks.and_then(|p| p.first()).and_then(|p| p.values.first()).and_then(|v| v.parse().ok());
        self.board.append(&[Event::Choice { msg, to }]);
        true
    }
}
