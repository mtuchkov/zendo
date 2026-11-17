//! The work on a document: the delta and its base, the visit that counts, and the jobs that the
//! agent does in the background: recaps and reviews (ADR 0008). The work does not see the app.
use super::{AUTO_PER_HOUR, Base, ENGAGED, IDLE, RecapJob, Visit};
use crate::agent::work::{Review, Reviews};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(smart_default::SmartDefault)]
pub struct Work {
    delta_open: bool,
    delta_base: Base,
    delta_scroll: usize,
    visit: Option<Visit>,
    /// The job of the agent in the background: one at a time.
    job: Option<RecapJob>,
    /// The files whose recap the user asked for.
    queue: VecDeque<String>,
    reviews: Reviews,
    /// The times of the automatic recaps of the last hour.
    auto: Vec<Instant>,
    /// The time of the last key or click: recaps wait until the user is idle.
    #[default(Instant::now())]
    last_input: Instant,
}

impl Work {
    pub fn delta_open(&self) -> bool {
        self.delta_open
    }

    /// Open or close the delta, at its top. True when it is open.
    pub fn toggle_delta(&mut self) -> bool {
        self.delta_open = !self.delta_open;
        self.delta_scroll = 0;
        self.delta_open
    }

    pub fn close_delta(&mut self) {
        self.delta_open = false;
    }

    /// What the delta compares the note with.
    pub fn base(&self) -> Base {
        self.delta_base
    }

    /// The other base, from the top of the delta.
    pub fn switch_base(&mut self) {
        self.delta_base = if self.delta_base == Base::LastVisit { Base::GitHead } else { Base::LastVisit };
        self.delta_scroll = 0;
    }

    pub fn delta_scroll(&self) -> usize {
        self.delta_scroll
    }

    pub fn delta_top(&mut self) {
        self.delta_scroll = 0;
    }

    /// The end of the delta: the draw fits it to the text.
    pub fn delta_bottom(&mut self) {
        self.delta_scroll = usize::MAX / 2;
    }

    pub fn scroll_delta(&mut self, up: bool, n: usize) {
        self.delta_scroll = if up { self.delta_scroll.saturating_sub(n) } else { self.delta_scroll + n };
    }

    /// The draw of the delta: the scroll stays in the text; `max` is its last first row.
    pub fn fit_delta(&mut self, max: usize) -> usize {
        self.delta_scroll = self.delta_scroll.min(max);
        self.delta_scroll
    }

    pub fn visit(&self) -> Option<&Visit> {
        self.visit.as_ref()
    }

    /// Scrolls and clicks on the note on screen.
    pub fn interact(&mut self, n: u32) {
        if let Some(v) = self.visit.as_mut() {
            v.interactions += n;
        }
    }

    pub fn mark_edited(&mut self) {
        if let Some(v) = self.visit.as_mut() {
            v.edited = true;
        }
    }

    /// The visit counts as work: an edit, or enough scrolls and clicks.
    pub fn visit_counts(&self) -> bool {
        self.visit.as_ref().is_some_and(|v| v.edited || v.interactions >= ENGAGED)
    }

    /// The visit that ends, for the app to keep when it counted.
    pub fn take_visit(&mut self) -> Option<Visit> {
        self.visit.take()
    }

    /// A new visit of the document on the screen, or none.
    pub fn start_visit(&mut self, path: Option<PathBuf>) {
        self.visit = path.map(|path| Visit { path, interactions: 0, edited: false });
    }

    pub fn job(&self) -> Option<&RecapJob> {
        self.job.as_ref()
    }

    pub fn job_mut(&mut self) -> Option<&mut RecapJob> {
        self.job.as_mut()
    }

    /// The agent has no job in the background.
    pub fn free(&self) -> bool {
        self.job.is_none()
    }

    pub fn start(&mut self, job: RecapJob) {
        self.job = Some(job);
    }

    pub fn take_job(&mut self) -> Option<RecapJob> {
        self.job.take()
    }

    /// A recap of the file is in work, or waits.
    pub fn recap_waits(&self, rel: &str) -> bool {
        self.job.as_ref().is_some_and(|j| j.rel == rel) || self.queue.iter().any(|r| r == rel)
    }

    /// The user asked for a recap of the file.
    pub fn ask_recap(&mut self, rel: String) {
        self.queue.push_back(rel);
    }

    pub fn next_recap(&mut self) -> Option<String> {
        self.queue.pop_front()
    }

    /// A key or a click came now.
    pub fn input_now(&mut self) {
        self.last_input = Instant::now();
    }

    /// The last key or click came at `at`.
    pub fn input_at(&mut self, at: Instant) {
        self.last_input = at;
    }

    /// An automatic recap can start: the user is idle long enough, and the hour has room.
    pub fn auto_allowed(&mut self) -> bool {
        self.auto.retain(|t| t.elapsed() < Duration::from_secs(3600));
        self.last_input.elapsed() >= IDLE && self.auto.len() < AUTO_PER_HOUR
    }

    /// An automatic recap starts now.
    pub fn count_auto(&mut self) {
        self.auto.push(Instant::now());
    }

    /// The review key: the first press waits for the second. True at a second press within `chord`.
    pub fn review_chord(&mut self, chord: Duration) -> bool {
        if self.reviews.armed.take().is_some_and(|t| t.elapsed() < chord) {
            return true;
        }
        self.reviews.armed = Some(Instant::now());
        false
    }

    /// The user asked for a review of the file. False when one waits already.
    pub fn ask_review(&mut self, rel: String) -> bool {
        let new = !self.reviews.queue.contains(&rel);
        if new {
            self.reviews.queue.push_back(rel);
        }
        new
    }

    pub fn next_review(&mut self) -> Option<String> {
        self.reviews.queue.pop_front()
    }

    /// A review of the file waits for the agent.
    pub fn review_waits(&self, rel: &str) -> bool {
        self.reviews.queue.iter().any(|r| r == rel)
    }

    pub fn show_review(&mut self, review: Review) {
        self.reviews.shown = Some(review);
    }

    /// The review on the screen.
    pub fn review(&self) -> Option<&Review> {
        self.reviews.shown.as_ref()
    }

    /// The review goes when `here`, the open file, is not its file.
    pub fn keep_review_of(&mut self, here: Option<&str>) {
        if self.reviews.shown.as_ref().is_some_and(|r| Some(r.rel.as_str()) != here) {
            self.reviews.shown = None;
        }
    }
}

#[cfg(test)]
#[path = "work_tests.rs"]
mod tests;
