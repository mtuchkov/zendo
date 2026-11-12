//! The review on request: `ctrl-r` twice. Claude finds the lines of the open file that a reader
//! finds confusing, and writes a short summary. The summary shows at the top of the file, and the
//! lines show marked, until the file changes.
use super::*;
use crate::setup::state::Review;
use std::collections::VecDeque;

/// The second `ctrl-r` must come within this time of the first.
pub const REVIEW_CHORD: Duration = Duration::from_millis(1500);
/// Claude marks at most this many lines.
pub const REVIEW_LINES: usize = 12;

/// Reviews on their way, and the time of the first `ctrl-r`.
#[derive(Clone, Debug, Default)]
pub struct Reviews {
    pub queue: VecDeque<String>,
    pub armed: Option<Instant>,
}

/// The review of a file, as the agent writes it: `SUMMARY: …`, then `LINE <n>: <why>`.
pub fn parse_review(s: &str) -> (String, Vec<(usize, String)>) {
    let mut summary = Vec::new();
    let mut lines = Vec::new();
    for l in s.lines().map(str::trim).filter(|l| !l.is_empty() && *l != "END") {
        match l.strip_prefix("LINE ").and_then(|rest| rest.split_once(':')) {
            Some((n, why)) => {
                let n: String = n.trim().chars().take_while(char::is_ascii_digit).collect();
                if let Ok(n) = n.parse::<usize>().map(|n| n.saturating_sub(1)) {
                    lines.push((n, why.trim().to_string()));
                }
            }
            None if lines.is_empty() => summary.push(l.strip_prefix("SUMMARY:").unwrap_or(l).trim()),
            None => {}
        }
    }
    lines.truncate(REVIEW_LINES);
    (summary.join(" "), lines)
}

impl App {
    /// `ctrl-r`: the first press waits for the second; the second asks for the review.
    pub fn review_key(&mut self) {
        if self.reviews.armed.take().is_some_and(|t| t.elapsed() < REVIEW_CHORD) {
            return self.review_now();
        }
        self.reviews.armed = Some(Instant::now());
        self.flash("ctrl-r again: Claude marks the confusing lines");
    }

    /// Ask Claude to review the open file now.
    pub fn review_now(&mut self) {
        let Some(path) = self.current.clone() else { return };
        if self.cfg.offline {
            return self.flash("offline: start zendo without --offline to use Claude");
        }
        if self.mode == Mode::Write && self.editor.dirty {
            self.save();
        }
        let rel = self.rel(&path);
        if !self.reviews.queue.contains(&rel) {
            self.ensure_agent();
            self.reviews.queue.push_back(rel);
        }
        self.flash("Claude reads the file for confusing lines");
    }

    /// Start a review that waits. True when one started.
    pub(super) fn review_tick(&mut self) -> bool {
        let Some(rel) = self.reviews.queue.pop_front() else { return false };
        let Ok(text) = std::fs::read_to_string(self.cfg.root.join(&rel)) else { return true };
        let numbered: String = text.lines().enumerate().map(|(i, l)| format!("{:>4}| {l}\n", i + 1)).collect();
        let prompt = format!("{REVIEW_PROMPT}File: {rel}\n--- the file, with line numbers ---\n{numbered}");
        let key = format!("{RECAP_PREFIX}review {rel}");
        self.recap_job = Some(RecapJob { rel, hash: state::hash(&text), text, buf: String::new(), auto: false, failed: None, meta: None, review: true });
        if let Some(a) = &self.agent {
            a.send(Cmd::Prompt { key, known: None, text: prompt, attach: vec![] });
        }
        true
    }

    /// The review turn ended: keep the summary and the lines.
    pub(super) fn review_done(&mut self, job: RecapJob) {
        let (summary, lines) = parse_review(&job.buf);
        if summary.is_empty() && lines.is_empty() {
            let why = job.failed.as_deref().and_then(|e| e.lines().next()).unwrap_or("no answer");
            return self.flash(format!("review failed: {why}"));
        }
        let n = lines.len();
        self.state.file(&job.rel).review = Some(Review { summary, lines, of_hash: job.hash, at: Local::now() });
        self.state.save(&self.cfg.root);
        self.flash(format!("review ready: {n} confusing line{}", if n == 1 { "" } else { "s" }));
    }

    /// The review of a file, when it is of the text that the file has now.
    pub fn fresh_review(&self, rel: &str, body: &str) -> Option<&Review> {
        self.state.files.get(rel)?.review.as_ref().filter(|r| r.of_hash == state::hash(body))
    }
}

const REVIEW_PROMPT: &str = "Find the lines of this file that a reader will find confusing: unclear, ambiguous, \
    contradictory, or hard to follow. Do not use tools and do not edit files.\n\
    Answer in this format only, with no other text:\n\
    SUMMARY: <at most 3 sentences: what the file says, and what in it is most confusing>\n\
    LINE <number>: <why the line is confusing, in at most 15 words>\n\
    Give at most 12 lines, the most confusing first. The numbers are the line numbers below.\n\n";

#[cfg(test)]
#[path = "review_tests.rs"]
mod tests;
