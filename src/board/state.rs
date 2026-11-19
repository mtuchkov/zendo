//! The state that the journal folds into: messages, claims, channels and choices (BOARD-1 to
//! BOARD-11). Posts are in `state_posts.rs`.

use super::{Event, How, Line, Msg, PostState, inbox, inbox_of, owner_of};
use crate::app::SESSIONS;
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Default, Clone, Debug)]
pub struct State {
    pub msgs: Vec<(u64, Msg)>,
    by_id: HashMap<String, usize>,
    claims: HashMap<(String, usize), How>,
    landed: HashSet<(String, usize)>,
    /// The open named channels of each repository, with their members; the owner is a member.
    channels: HashMap<(String, String), BTreeSet<usize>>,
    choices: HashMap<String, Option<usize>>,
    pub(super) posts: Vec<PostState>,
    pub(super) post_ix: HashMap<String, usize>,
    /// The number of the line that comes next.
    pub(super) seq: usize,
    /// The time of the last line.
    pub(super) last: u64,
}

impl State {
    pub fn fold(lines: Vec<Line>) -> State {
        let mut s = State::default();
        lines.into_iter().for_each(|l| s.apply(l));
        s
    }

    /// Fold one more line into the state.
    pub fn apply(&mut self, line: Line) {
        self.seq += 1;
        self.last = self.last.max(line.at);
        match line.ev {
            Event::Msg(m) => self.add_msg(line.at, m),
            Event::Claim { msg, to, how } => {
                self.claims.entry((msg, to)).or_insert(how);
            }
            Event::Landed { msg, to } => {
                self.landed.insert((msg, to));
            }
            Event::Invite { repo, channel, session } => {
                if let Some(m) = self.channels.get_mut(&(repo, channel)) {
                    m.insert(session);
                }
            }
            Event::Leave { repo, channel, session } => self.leave(repo, channel, session),
            Event::ThreadEnd { repo, session } => self.thread_end(repo, session, line.at),
            Event::Channel { repo, channel, members } => drop(self.channels.insert((repo, channel), members.into_iter().collect())),
            Event::Close { repo, channel } => drop(self.channels.remove(&(repo, channel))),
            Event::Choice { msg, to } => {
                self.choices.entry(msg).or_insert(to);
            }
            other => self.apply_post(line.at, other),
        }
    }

    fn add_msg(&mut self, at: u64, m: Msg) {
        for ch in [m.to.clone(), m.reply_to.clone()].into_iter().flatten() {
            if owner_of(&ch) == Some(m.from) {
                self.channels.entry((m.repo.clone(), ch)).or_insert_with(|| BTreeSet::from([m.from]));
            }
        }
        self.by_id.insert(m.id.clone(), self.msgs.len());
        self.msgs.push((at, m));
    }

    fn leave(&mut self, repo: String, channel: String, session: usize) {
        if owner_of(&channel) == Some(session) {
            return;
        }
        if let Some(m) = self.channels.get_mut(&(repo, channel)) {
            m.remove(&session);
        }
    }

    fn thread_end(&mut self, repo: String, session: usize, at: u64) {
        self.channels.retain(|(r, ch), _| !(*r == repo && owner_of(ch) == Some(session)));
        self.end_posts_of(&repo, session, at);
    }

    /// The open named channels with their members, for a compaction.
    pub(super) fn channels(&self) -> impl Iterator<Item = (&(String, String), &BTreeSet<usize>)> {
        self.channels.iter()
    }

    pub fn msg(&self, id: &str) -> Option<&Msg> {
        self.by_id.get(id).map(|i| &self.msgs[*i].1)
    }

    pub fn claimed(&self, id: &str, to: usize) -> Option<How> {
        self.claims.get(&(id.to_string(), to)).copied()
    }

    pub fn landed(&self, id: &str, to: usize) -> bool {
        self.landed.contains(&(id.to_string(), to))
    }

    pub fn choice(&self, id: &str) -> Option<Option<usize>> {
        self.choices.get(id).copied()
    }

    /// The members of an open named channel.
    pub fn members(&self, repo: &str, channel: &str) -> Option<&BTreeSet<usize>> {
        self.channels.get(&(repo.to_string(), channel.to_string()))
    }

    /// The full name of the channel `name` for session `s`, or None: not found (BOARD-4).
    pub fn resolve(&self, repo: &str, s: usize, name: &str) -> Option<String> {
        if name == "all" {
            return Some(name.into());
        }
        if let Some(n) = inbox_of(name) {
            return (n < SESSIONS).then(|| inbox(n));
        }
        let member = |ch: &str| self.members(repo, ch).is_some_and(|m| m.contains(&s));
        if name.contains('/') {
            return member(name).then(|| name.to_string());
        }
        let own = format!("{s}/{name}");
        if member(&own) {
            return Some(own);
        }
        let mut theirs = self.channels.iter().filter(|((r, ch), m)| r == repo && ch.split_once('/').is_some_and(|(_, n)| n == name) && m.contains(&s));
        match (theirs.next(), theirs.next()) {
            (Some(((_, ch), _)), None) => Some(ch.clone()),
            _ => None,
        }
    }

    /// Like `resolve`, but a short name that names no channel of the session is a new channel of
    /// its own (BOARD-7).
    pub fn resolve_or_create(&self, repo: &str, s: usize, name: &str) -> Option<String> {
        let new = !name.is_empty() && !name.contains('/') && !name.contains(':') && name != "all";
        self.resolve(repo, s, name).or_else(|| new.then(|| format!("{s}/{name}")))
    }

    /// The channel of a message: its target, or the session that the user chose.
    pub fn target(&self, m: &Msg) -> Option<String> {
        m.to.clone().or_else(|| self.choice(&m.id).flatten().map(inbox))
    }

    /// The recipients of `m` now. A message to `all` or to a named channel goes only to the
    /// sessions whose agent runs (BOARD-3).
    pub fn recipients(&self, m: &Msg, running: &dyn Fn(usize) -> bool) -> Vec<usize> {
        let Some(to) = self.target(m) else { return Vec::new() };
        if let Some(n) = inbox_of(&to) {
            return if n == m.from { Vec::new() } else { vec![n] };
        }
        let all: BTreeSet<usize> = (0..SESSIONS).collect();
        let pool = if to == "all" { Some(&all) } else { self.members(&m.repo, &to) };
        pool.into_iter().flatten().copied().filter(|n| *n != m.from && running(*n)).collect()
    }

    /// The messages of `repo` that wait for session `r`, in their order: not taken, and not held
    /// behind an initiation in flight (BOARD-6).
    pub fn pending(&self, repo: &str, r: usize, running: &dyn Fn(usize) -> bool) -> Vec<&Msg> {
        self.msgs
            .iter()
            .enumerate()
            .filter(|(_, (_, m))| m.repo == repo && self.claimed(&m.id, r).is_none())
            .filter(|(_, (_, m))| self.recipients(m, running).contains(&r))
            .filter(|(i, (_, m))| !self.held(*i, m))
            .map(|(_, (_, m))| m)
            .collect()
    }

    /// An initiation is held while an earlier initiation of the same sender to the same channel is
    /// in flight with any of its recipients.
    fn held(&self, i: usize, m: &Msg) -> bool {
        if !m.kind.initiation() {
            return false;
        }
        let ch = self.target(m);
        self.msgs[..i].iter().any(|(_, e)| e.repo == m.repo && e.from == m.from && e.kind.initiation() && self.target(e) == ch && self.in_flight(e))
    }

    /// Some recipient took the initiation `e`, and neither answered it nor ended the turn that
    /// took it.
    pub(super) fn in_flight(&self, e: &Msg) -> bool {
        self.claims.keys().filter(|(id, _)| *id == e.id).any(|(_, r)| !self.landed(&e.id, *r) && !self.responded(e, *r))
    }

    /// Session `r` sent a response to `e`.
    pub fn responded(&self, e: &Msg, r: usize) -> bool {
        self.responses(e).any(|m| m.from == r)
    }

    /// The responses to `e`.
    pub fn responses<'a>(&'a self, e: &'a Msg) -> impl Iterator<Item = &'a Msg> + 'a {
        self.msgs.iter().map(|(_, m)| m).filter(move |m| m.repo == e.repo && m.in_reply_to.as_deref() == Some(&e.id))
    }

    /// The initiations that session `r` took and that are still in flight with it: the end of its
    /// turn lands them (BOARD-9).
    pub fn taken(&self, repo: &str, r: usize) -> Vec<String> {
        let mut ids: Vec<String> = self.claims.keys().filter(|(_, to)| *to == r).map(|(id, _)| id.clone()).collect();
        ids.retain(|id| self.msg(id).is_some_and(|m| m.repo == repo && m.kind.initiation() && !self.landed(id, r) && !self.responded(m, r)));
        ids.sort_by_key(|id| self.by_id.get(id).copied());
        ids
    }

    /// The messages of `repo` that session `r` took in the way `how`, in their order.
    pub fn taken_as(&self, repo: &str, r: usize, how: How) -> Vec<&Msg> {
        self.msgs.iter().map(|(_, m)| m).filter(|m| m.repo == repo && self.claimed(&m.id, r) == Some(how)).collect()
    }

    /// The open named channels of `repo` that session `s` belongs to.
    pub fn channels_of(&self, repo: &str, s: usize) -> Vec<String> {
        let mut v: Vec<String> = self.channels.iter().filter(|((r, _), m)| r == repo && m.contains(&s)).map(|((_, ch), _)| ch.clone()).collect();
        v.sort();
        v
    }
}

#[cfg(test)]
#[path = "state_tests.rs"]
mod tests;
