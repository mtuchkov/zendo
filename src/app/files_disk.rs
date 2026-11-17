//! The files and the disk: `project-structure.toml` and the tree view that follows it, the fit of
//! the tree to the screen, and the changes that come from outside zendo.
use super::{Effect, Files};
use crate::docs::structure::{self, Structure};
use crate::docs::tree::View;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

impl Files {
    /// The saved tree view, else the structure when the repo has one, else docs.
    pub fn apply_tree_view(&mut self, saved: &str, all: bool) {
        let has = self.tree.structure.as_ref().is_some_and(|s| s.main().next().is_some());
        self.tree.view = match View::from_name(saved) {
            Some(View::Structure) if !has => View::Docs,
            Some(v) => v,
            None if all => View::Text,
            None if has => View::Structure,
            None => View::Docs,
        };
    }

    /// Read `project-structure.toml` again; keep the reason when it does not parse. `saved` and
    /// `all` are the tree view that the state keeps.
    pub fn load_structure(&mut self, root: &Path, saved: &str, all: bool) -> Option<Effect> {
        let mut said = None;
        match Structure::load(root) {
            Some(Ok(s)) => {
                self.tree.structure = Some(s);
                self.structure_error = None;
                if let Some(crate::setup::doctor::Problem::Issues(list)) = crate::setup::doctor::structure_problem(root) {
                    said = Effect::flash(format!("{}: {} thing(s) zendo does not understand · zendo doctor --fix", structure::FILE, list.len()));
                }
            }
            Some(Err(e)) => {
                self.structure_error = Some(e.lines().next().unwrap_or("").to_string());
                said = Effect::flash(format!("{} does not parse · zendo doctor --fix", structure::FILE));
            }
            None => {
                self.tree.structure = None;
                self.structure_error = None;
            }
        }
        self.structure_seen = std::fs::metadata(root.join(structure::FILE)).and_then(|m| m.modified()).ok();
        self.apply_tree_view(saved, all);
        said
    }

    /// The next tick looks at the disk, without the wait of a second.
    pub fn look_again(&mut self) {
        self.disk_checked = None;
    }

    /// At most once a second: `project-structure.toml` changed on disk.
    pub(crate) fn disk_due(&mut self, root: &Path) -> Option<bool> {
        if self.disk_checked.is_some_and(|t| t.elapsed() < Duration::from_secs(1)) {
            return None;
        }
        self.disk_checked = Some(Instant::now());
        let st = std::fs::metadata(root.join(structure::FILE)).and_then(|m| m.modified()).ok();
        Some(st != self.structure_seen)
    }

    /// Changes made outside zendo: Claude Code, git pull, another editor. True when the notes
    /// folder or the open document changed since the last look, but not at the first look.
    pub(crate) fn disk_changed(&mut self) -> bool {
        // The open document came into being in the checkout of the session (FILES-44).
        if self.current.as_ref().is_some_and(|p| p.exists()) && self.peek.on.is_none() && self.note().is_none() {
            self.reload();
        }
        let Ok(dir) = std::fs::read_dir(&self.store.dir) else { return false };
        let md: Vec<Option<SystemTime>> = dir.flatten().filter(|e| e.path().extension().is_some_and(|x| x == "md")).map(|e| e.metadata().and_then(|m| m.modified()).ok()).collect();
        let open = self.current.as_ref().and_then(|p| std::fs::metadata(p).ok()).and_then(|m| m.modified().ok());
        let now = (md.len(), md.iter().copied().chain([open]).max().flatten());
        if now == self.disk {
            return false;
        }
        let first = self.disk.1.is_none() && self.disk.0 == 0;
        self.disk = now;
        !first
    }
}
