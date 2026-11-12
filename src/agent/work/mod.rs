//! Work on files: visits that count, delta bases, and recaps Claude writes when idle.

mod meta;
mod recap;
mod review;

pub use meta::*;
pub use review::*;

use crate::agent::Cmd;
use crate::app::{App, Base, Focus, IDLE, AUTO_PER_HOUR, ENGAGED, Mode, RecapJob, Visit};
use crate::docs::delta;
use crate::setup::state::{self, Recap, Snapshot};
use chrono::Local;
use std::time::{Duration, Instant};

pub const RECAP_PREFIX: &str = "(recap) ";

impl App {
    /// The text of the note on screen, with unsaved edits.
    pub fn note_text(&self) -> Option<String> {
        if self.mode == Mode::Write && self.editor_path == self.current {
            return Some(format!("{}\n", self.editor.text().trim_end()));
        }
        self.note().map(|n| n.body.clone())
    }

    // ----- visits ------------------------------------------------------------

    /// Count scrolls and clicks on the note on screen.
    pub fn interact(&mut self, n: u32) {
        self.sync_visit();
        if let Some(v) = self.visit.as_mut() {
            v.interactions += n;
        }
    }

    pub fn mark_edited(&mut self) {
        self.sync_visit();
        if let Some(v) = self.visit.as_mut() {
            v.edited = true;
        }
    }

    pub fn visit_counts(&self) -> bool {
        self.visit.as_ref().is_some_and(|v| v.edited || v.interactions >= ENGAGED)
    }

    /// Start a visit for the note on screen; end the visit of the note we left.
    pub fn sync_visit(&mut self) {
        let here = match self.mode {
            Mode::Read | Mode::Write | Mode::Theme | Mode::Help => self.current.clone(),
            _ => None,
        };
        if self.visit.as_ref().map(|v| &v.path) == here.as_ref() {
            return;
        }
        self.end_visit();
        self.visit = here.map(|path| Visit { path, interactions: 0, edited: false });
    }

    /// A visit that counted moves the delta base to the text you just worked on.
    pub fn end_visit(&mut self) {
        let Some(v) = self.visit.take() else { return };
        if !(v.edited || v.interactions >= ENGAGED) {
            return;
        }
        let rel = self.rel(&v.path);
        let text = std::fs::read_to_string(&v.path).unwrap_or_default();
        let f = self.state.file(&rel);
        f.visits += 1;
        f.interactions += v.interactions;
        if v.edited {
            f.edits += 1;
        }
        f.worked_at = Some(Local::now());
        f.seen = Some(Snapshot { hash: state::hash(&text), text, at: Local::now() });
        self.state.save(&self.cfg.root);
        self.mark_tree();
    }

    /// Δ marks for the tree: files whose text differs from the last visit that counted.
    pub fn mark_tree(&mut self) {
        let root = self.cfg.root.clone();
        self.tree.changed = self
            .state
            .files
            .iter()
            .filter_map(|(rel, f)| f.seen.as_ref().map(|s| (rel, s)))
            .filter(|(rel, s)| std::fs::read_to_string(root.join(rel)).is_ok_and(|t| state::hash(&t) != s.hash))
            .map(|(rel, _)| std::path::PathBuf::from(rel))
            .collect();
    }

    // ----- delta -------------------------------------------------------------

    pub fn git_head(&self, rel: &str) -> Option<String> {
        let out = std::process::Command::new("git")
            .args(["show", &format!("HEAD:{rel}")])
            .current_dir(&self.cfg.root)
            .stderr(std::process::Stdio::null())
            .output()
            .ok()?;
        out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    }

    /// The text the delta starts from, and a label for it.
    pub fn delta_base(&self) -> (String, String) {
        let Some(path) = &self.current else { return ("no note".into(), String::new()) };
        let rel = self.rel(path);
        if self.delta_base == Base::LastVisit
            && let Some(s) = self.state.files.get(&rel).and_then(|f| f.seen.as_ref()) {
                return (format!("since your last visit · {}", s.at.format("%a %-d %b %H:%M")), s.text.clone());
            }
        let label = if self.delta_base == Base::LastVisit { "vs git HEAD (no visit saved yet)" } else { "vs git HEAD" };
        match self.git_head(&rel) {
            Some(t) => (label.into(), t),
            None => (format!("{label} · new file"), String::new()),
        }
    }

    pub fn toggle_delta(&mut self) {
        self.delta_open = !self.delta_open;
        self.delta_scroll = 0;
        self.focus = if self.delta_open { Focus::Delta } else { Focus::Note };
        self.make_room_for_delta();
    }

    /// The delta is open but the panes do not all fit beside the file tree: the tree closes.
    pub fn make_room_for_delta(&mut self) {
        let width: u16 = self.pane_rects.iter().map(|(_, r)| r.width).sum();
        if !self.delta_open || !self.state.tree || width == 0 {
            return;
        }
        let wanted = crate::ui::panes(self, u16::MAX);
        let need: u16 = wanted.iter().map(|p| if *p == Focus::Note { 50 } else { crate::ui::pane_width(self, *p, width) }).sum();
        if need <= width {
            return;
        }
        self.state.tree = false;
        self.state.save(&self.cfg.root);
        if self.focus == Focus::Tree {
            self.focus = Focus::Note;
        }
    }

    /// Open the chat with the delta attached to the next question.
    pub fn ask_about_delta(&mut self) {
        let Some(new) = self.note_text() else { return };
        let (label, old) = self.delta_base();
        let rel = self.current.as_ref().map(|p| self.rel(p)).unwrap_or_default();
        let d = delta::unified(&old, &new, &rel);
        if d.trim().is_empty() {
            self.flash("no changes to ask about");
            return;
        }
        self.ask_context = Some(format!("Changes in this note ({label}):\n{d}"));
        self.ask_default = Some(crate::app::DELTA_QUESTION.into());
        self.ask_about = "the changes";
        self.open_panel();
    }

    // ----- recaps ------------------------------------------------------------

}


#[cfg(test)]
#[path = "meta_tests.rs"]
mod meta_tests;
