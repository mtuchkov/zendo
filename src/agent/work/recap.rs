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

impl App {
    /// The last commit that changed the file: its id, date, message and diff, shortened.
    pub fn last_commit(&self, rel: &str) -> Option<String> {
        let out = std::process::Command::new("git")
            .args(["log", "-1", "--date=short", "--format=%h %ad %s%n%b", "-p", "--", rel])
            .current_dir(&self.cfg.root)
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        (out.status.success() && !text.trim().is_empty()).then(|| text.chars().take(COMMIT_MAX).collect())
    }
}

impl App {
    /// Recap the note on screen now.
    pub fn recap_now(&mut self) {
        let Some(path) = self.current.clone() else { return };
        let rel = self.rel(&path);
        if self.recap_job.as_ref().is_some_and(|j| j.rel == rel) || self.recap_queue.contains(&rel) {
            self.flash("the recap is on its way");
            return;
        }
        if self.cfg.offline {
            self.flash("offline: start zendo without --offline to use Claude");
            return;
        }
        self.ensure_agent();
        self.recap_queue.push_back(rel);
        self.state.show_recap = true;
        self.flash("writing a recap");
    }

    pub(super) fn agent_idle(&self) -> bool {
        self.agent.is_some()
            && self.agent_name.is_some()
            && self.recap_job.is_none()
            && self.threads.iter().all(|(k, t)| t.busy.is_none() && (k.starts_with(RECAP_PREFIX) || t.pending().is_none()))
    }

    /// The next file for an automatic recap: worked on in the last week, changed since its recap.
    /// Any markdown file counts, also files opened from the tree.
    pub(super) fn next_auto(&self) -> Option<String> {
        let week = chrono::Duration::days(7);
        let mut cands: Vec<(chrono::DateTime<Local>, String)> = Vec::new();
        let here = self.visit.as_ref().filter(|_| self.visit_counts()).map(|v| self.rel(&v.path));
        let mut rels: Vec<String> = self.state.files.keys().cloned().collect();
        if let Some(h) = &here
            && !rels.contains(h) {
                rels.push(h.clone());
            }
        for rel in rels {
            let f = self.state.files.get(&rel);
            let worked = f.and_then(|f| f.worked_at).filter(|t| Local::now() - *t < week);
            let when = if here.as_deref() == Some(rel.as_str()) { Some(Local::now()) } else { worked };
            let Some(when) = when else { continue };
            let Ok(body) = std::fs::read_to_string(self.cfg.root.join(&rel)) else { continue };
            let fresh = f.and_then(|f| f.recap.as_ref()).is_some_and(|r| r.of_hash == state::hash(&body));
            if !fresh && !body.trim().is_empty() {
                cands.push((when, rel));
            }
        }
        cands.sort();
        cands.pop().map(|(_, r)| r)
    }

    /// Called on every tick: start a requested recap, or an automatic one when all is idle.
    pub fn recap_tick(&mut self) {
        if !self.agent_idle() {
            return;
        }
        if self.review_tick() || self.meta_tick() {
            return;
        }
        let (rel, auto) = if let Some(r) = self.recap_queue.pop_front() {
            (r, false)
        } else {
            self.auto_recaps.retain(|t| t.elapsed() < Duration::from_secs(3600));
            if self.last_input.elapsed() < IDLE || self.auto_recaps.len() >= AUTO_PER_HOUR {
                return;
            }
            match self.next_auto() {
                Some(r) => {
                    self.auto_recaps.push(Instant::now());
                    (r, true)
                }
                None => return,
            }
        };
        let path = self.cfg.root.join(&rel);
        let Ok(text) = std::fs::read_to_string(&path) else { return };
        let prompt = self.recap_prompt(&rel, &path, &text);
        let key = format!("{RECAP_PREFIX}{rel}");
        self.recap_job = Some(RecapJob { rel, hash: state::hash(&text), text, buf: String::new(), auto, failed: None, meta: None, review: false });
        if let Some(a) = &self.agent {
            a.send(Cmd::Prompt { key, known: None, text: prompt, attach: vec![] });
        }
    }

    /// The recap prompt: the file, and its changes against git HEAD, or else its last commit.
    fn recap_prompt(&self, rel: &str, path: &std::path::Path, text: &str) -> String {
        let mut prompt = String::from(RECAP_PROMPT);
        prompt.push_str(&format!("File: {}\n--- file ---\n{text}\n", path.display()));
        let head = self.git_head(rel);
        let d = head.as_deref().map(|h| delta::unified(h, text, rel)).unwrap_or_default();
        match (head, self.last_commit(rel)) {
            (Some(_), _) if !d.trim().is_empty() => prompt.push_str(&format!("{GIT_CHANGES}\n{d}\n")),
            (Some(_), Some(c)) => prompt.push_str(&format!("{LAST_COMMIT}\n{c}\n")),
            _ => prompt.push_str("--- the file is not in git yet ---\n"),
        }
        prompt
    }

    /// The recap turn ended: keep the text, drop the throwaway session.
    pub fn recap_done(&mut self, key: String) {
        let Some(job) = self.recap_job.take() else { return };
        if let Some(a) = &self.agent {
            a.send(Cmd::Forget { key });
        }
        if job.meta.is_some() {
            return self.meta_done(job);
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
        self.state.save(&self.cfg.root);
        let title = self.notes.iter().find(|n| self.rel(&n.path) == rel).map(|n| n.title.clone()).unwrap_or(rel);
        self.flash(format!("recap ready: {title}"));
    }
}
