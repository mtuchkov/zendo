//! Recaps: when they run, and what they keep.
use super::*;

const RECAP_PROMPT: &str = "Write a recap of this file, for me to read tomorrow morning.\n\
    Rules: at most 4 short markdown bullets, no preamble, no heading. Do not use tools and do not edit files.\n\
    1) What the file is about as a whole: what it decides, proposes or does.\n\
    2) What the changes below do and why they matter: the changes against git HEAD, or else the last commit of the file. Say which one.\n\
    No open questions.\n\n";
pub const GIT_CHANGES: &str = "--- changes against git HEAD, not committed ---";
pub const LAST_COMMIT: &str = "--- no changes against git HEAD; the last commit of the file ---";
/// The last commit goes to the agent up to this many chars.
const COMMIT_MAX: usize = 8000;

/// Git for a recap gets this long (IO-5).
const GIT_LIMIT: std::time::Duration = std::time::Duration::from_secs(5);
/// Without a file to recap, the next look for an automatic recap waits this long.
const AUTO_AGAIN: std::time::Duration = std::time::Duration::from_secs(30);

/// The checkout of `rel` and its path in it: a file of a worktree is in its worktree (FILES-34).
fn in_checkout(root: &std::path::Path, rel: &str) -> crate::app::InCheckout {
    match crate::docs::tree::worktree::split(rel) {
        Some((wt, rest)) => (root.join(wt), rest.to_string()),
        None => (root.to_path_buf(), rel.to_string()),
    }
}

/// The last commit that changed the file: its id, date, message and diff, shortened. On a worker.
pub fn last_commit(root: &std::path::Path, rel: &str) -> Option<String> {
    let (dir, rel) = in_checkout(root, rel);
    let text = crate::setup::sync::git_out(&dir, &["log", "-1", "--date=short", "--format=%h %ad %s%n%b", "-p", "--", &rel], GIT_LIMIT).ok()?;
    (!text.trim().is_empty()).then(|| text.chars().take(COMMIT_MAX).collect())
}

/// A recap that a worker prepared: the file, its text, and the prompt.
pub struct Prepared {
    pub rel: String,
    pub auto: bool,
    pub text: String,
    pub prompt: String,
}

/// A file that may get an automatic recap: when it was worked on, and the hash of its last recap.
type Candidate = (chrono::DateTime<Local>, String, Option<String>);

/// On a worker: the newest candidate whose text changed since its recap, and its prompt.
fn prepare(root: &std::path::Path, cands: Vec<Candidate>, auto: bool) -> Option<Prepared> {
    let mut fresh: Vec<(chrono::DateTime<Local>, String, String)> = cands
        .into_iter()
        .filter_map(|(when, rel, recap)| {
            let body = std::fs::read_to_string(root.join(&rel)).ok()?;
            let stale = recap.as_deref() != Some(state::hash(&body).as_str());
            (stale && !body.trim().is_empty()).then_some((when, rel, body))
        })
        .collect();
    fresh.sort();
    let (_, rel, text) = fresh.pop()?;
    let prompt = recap_prompt(root, &rel, &text);
    Some(Prepared { rel, auto, text, prompt })
}

/// The recap prompt: the file, and its changes against git HEAD, or else its last commit.
fn recap_prompt(root: &std::path::Path, rel: &str, text: &str) -> String {
    let path = root.join(rel);
    let mut prompt = String::from(RECAP_PROMPT);
    prompt.push_str(&format!("File: {}\n--- file ---\n{text}\n", path.display()));
    let head = crate::app::facts_head_of(&in_checkout(root, rel));
    let d = head.as_deref().map(|h| delta::unified(h, text, rel)).unwrap_or_default();
    match (head, last_commit(root, rel)) {
        (Some(_), _) if !d.trim().is_empty() => prompt.push_str(&format!("{GIT_CHANGES}\n{d}\n")),
        (Some(_), Some(c)) => prompt.push_str(&format!("{LAST_COMMIT}\n{c}\n")),
        _ => prompt.push_str("--- the file is not in git yet ---\n"),
    }
    prompt
}

impl App {
    /// Recap the note on screen now.
    pub fn recap_now(&mut self) {
        let Some(path) = self.files.current().cloned() else { return };
        let rel = self.rel(&path);
        if self.work.recap_waits(&rel) {
            self.flash("the recap is on its way");
            return;
        }
        if self.cfg.offline {
            self.flash("offline: start zendo without --offline to use Claude");
            return;
        }
        self.ensure_agent();
        self.work.ask_recap(rel);
        self.state.show_recap = true;
        self.flash("writing a recap");
    }

    /// The agent runs and has no job in the background.
    fn agent_free(&self) -> bool {
        self.link.running() && self.link.ready() && self.work.free()
    }

    pub(super) fn agent_idle(&self) -> bool {
        self.agent_free() && self.threads.values().all(|t| t.busy.is_none())
    }

    /// The files for an automatic recap: worked on in the last week, with the hash of their last
    /// recap; any markdown file counts, also files opened from the tree. A worker reads them.
    pub(super) fn auto_candidates(&self) -> Vec<Candidate> {
        let week = chrono::Duration::days(7);
        let here = self.work.visit().filter(|_| self.visit_counts()).map(|v| self.rel(&v.path));
        let mut rels: Vec<String> = self.state.files.keys().cloned().collect();
        if let Some(h) = &here
            && !rels.contains(h)
        {
            rels.push(h.clone());
        }
        rels.into_iter()
            .filter_map(|rel| {
                let f = self.state.files.get(&rel);
                let worked = f.and_then(|f| f.worked_at).filter(|t| Local::now() - *t < week);
                let when = if here.as_deref() == Some(rel.as_str()) { Some(Local::now()) } else { worked }?;
                Some((when, rel, f.and_then(|f| f.recap.as_ref()).map(|r| r.of_hash.clone())))
            })
            .collect()
    }

    /// Start the preparation of the next recap on a worker: the one that the user asked for, or an
    /// automatic one at most each 30 s (IO-1).
    fn prepare_recap(&mut self) {
        let root = self.cfg.root.clone();
        if let Some(rel) = self.work.next_recap() {
            let cands = vec![(Local::now(), rel, None)];
            return self.recaps.start(move || prepare(&root, cands, false));
        }
        if !self.work.auto_allowed() || self.recap_looked.is_some_and(|t| t.elapsed() < AUTO_AGAIN) {
            return;
        }
        self.recap_looked = Some(std::time::Instant::now());
        let cands = self.auto_candidates();
        self.recaps.start(move || prepare(&root, cands, true));
    }

    /// Called on every tick: start a requested recap, or an automatic one when all is idle. A
    /// question for a name does not wait for the turns of the chat (NAMES-1).
    pub fn recap_tick(&mut self) {
        // A title does not wait for the user: the agent starts for it, where no API answers
        // (FLOAT-18, FLOAT-19, NAMES-16).
        let no_api = self.link.api().is_none() && !self.link.api_pending();
        if self.wants_title() && no_api && self.link.down().is_none() {
            self.ensure_agent();
        }
        if !self.agent_free() {
            return;
        }
        let idle = self.agent_idle();
        if idle && (self.review_tick() || self.meta_tick()) {
            return;
        }
        if self.title_tick() || self.name_tick() || !idle {
            return;
        }
        // The worker prepares the recap; its answer starts the job (IO-1).
        let Some(prepared) = self.recaps.poll() else {
            if !self.recaps.running() {
                self.prepare_recap();
            }
            return;
        };
        let Some(Prepared { rel, auto, text, prompt }) = prepared else { return };
        if auto {
            self.work.count_auto();
        }
        let key = format!("{RECAP_PREFIX}{rel}");
        self.work.start(RecapJob { rel, hash: state::hash(&text), text, buf: String::new(), auto, failed: None, meta: None, review: false, name: None, title: None });
        self.link.send(Cmd::Prompt { key, known: None, text: prompt, attach: vec![] });
    }

    /// The recap turn ended: keep the text, drop the throwaway session.
    pub fn recap_done(&mut self, key: String) {
        let Some(job) = self.work.take_job() else { return };
        self.link.send(Cmd::Forget { key });
        if job.meta.is_some() {
            return self.meta_done(job);
        }
        if job.title.is_some() {
            return self.title_done(job);
        }
        if job.name.is_some() {
            return self.name_done(job);
        }
        if job.review {
            return self.review_done(job);
        }
        let text = job.buf.trim().to_string();
        if let Some(e) = job.failed.filter(|_| text.is_empty()) {
            self.flash(format!("recap failed: {}", e.lines().next().unwrap_or("")));
            return;
        }
        if text.is_empty() {
            return;
        }
        let rel = job.rel.clone();
        self.state.file(&rel).recap = Some(Recap { text, of_hash: job.hash, of_text: job.text, at: Local::now(), auto: job.auto });
        self.save_state();
        let title = self.files.notes().iter().find(|n| self.rel(&n.path) == rel).map(|n| n.title.clone()).unwrap_or(rel);
        self.flash(format!("recap ready: {title}"));
    }
}
