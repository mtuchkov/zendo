//! Posts and their leases in the state of the board (BOARD-13 to BOARD-19, BOARD-29 to
//! BOARD-35).

use super::{Event, HEARTBEAT_MS, Post, State};

/// Why a post went to the history.
#[derive(Clone, Debug, PartialEq)]
pub enum End {
    Released,
    ThreadEnded,
    /// The session that set the tombstone, with its repository.
    Tombstone { repo: String, session: usize },
}

#[derive(Clone, Debug)]
pub struct PostState {
    pub post: Post,
    pub at: u64,
    pub end: Option<End>,
    /// The time and the line of the last expiry and the last renewal.
    expired: Option<(u64, usize)>,
    renewed: Option<(u64, usize)>,
}

/// The lease of a post: live, and the time of its last renewal or of its expiry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lease {
    pub live: bool,
    pub since: u64,
}

impl PostState {
    /// An expiry came after the last renewal.
    pub fn expired(&self) -> bool {
        self.expired.is_some_and(|(_, e)| self.renewed.is_none_or(|(_, r)| r < e))
    }

    /// The lease now. `beat` is the last heartbeat of a zendo process of the repository of the
    /// author: while none beats, no lease renews (BOARD-29).
    pub fn lease(&self, now: u64, beat: Option<u64>) -> Lease {
        if let Some((at, _)) = self.expired.filter(|_| self.expired()) {
            return Lease { live: false, since: at };
        }
        let renewed = self.renewed.map_or(self.at, |(at, _)| at.max(self.at));
        match beat {
            Some(b) if now.saturating_sub(b) <= HEARTBEAT_MS => Lease { live: true, since: now },
            Some(b) => Lease { live: false, since: b.max(renewed) },
            None => Lease { live: false, since: renewed },
        }
    }
}

impl State {
    pub(super) fn apply_post(&mut self, at: u64, ev: Event) {
        let seq = self.seq;
        match ev {
            Event::Post(post) => {
                self.post_ix.insert(post.id.clone(), self.posts.len());
                self.posts.push(PostState { post, at, end: None, expired: None, renewed: None });
            }
            Event::Release { post } => self.end_post(&post, End::Released),
            Event::Expired { post } => self.live_post(&post).into_iter().for_each(|p| p.expired = Some((at, seq))),
            Event::Renewed { post } => self.live_post(&post).into_iter().for_each(|p| p.renewed = Some((at, seq))),
            Event::Tombstone { post, repo, session } if self.live_post(&post).is_some_and(|p| p.expired()) => {
                self.end_post(&post, End::Tombstone { repo, session });
            }
            _ => {}
        }
    }

    fn live_post(&mut self, id: &str) -> Option<&mut PostState> {
        let i = *self.post_ix.get(id)?;
        self.posts.get_mut(i).filter(|p| p.end.is_none())
    }

    fn end_post(&mut self, id: &str, end: End) {
        if let Some(p) = self.live_post(id) {
            p.end = Some(end);
        }
    }

    pub(super) fn end_posts_of(&mut self, repo: &str, session: usize) {
        for p in self.posts.iter_mut().filter(|p| p.end.is_none() && p.post.repo == repo && p.post.session == session) {
            p.end = Some(End::ThreadEnded);
        }
    }

    pub fn post(&self, id: &str) -> Option<&PostState> {
        self.post_ix.get(id).map(|i| &self.posts[*i])
    }

    /// The posts on the board, oldest first.
    pub fn live_posts(&self) -> impl Iterator<Item = &PostState> {
        self.posts.iter().filter(|p| p.end.is_none())
    }

    /// The posts in the history, oldest first.
    pub fn history(&self) -> impl Iterator<Item = &PostState> {
        self.posts.iter().filter(|p| p.end.is_some())
    }

    /// The live post of session `s` about `resource`.
    pub fn post_of(&self, repo: &str, s: usize, resource: &str) -> Option<&PostState> {
        self.live_posts().find(|p| p.post.repo == repo && p.post.session == s && p.post.resource == resource)
    }
}
