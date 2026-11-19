//! The files and the disk: `project-structure.toml` and the tree view that follows it, the fit of
//! the tree to the screen, and the changes that come from outside zendo.
use super::{Effect, Files};
use crate::docs::structure::{self, Structure};
use crate::docs::tree::View;
use std::path::Path;

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
    /// `all` are the tree view that the state keeps. Before the first frame only (IO-7).
    pub fn load_structure(&mut self, root: &Path, saved: &str, all: bool) -> Option<Effect> {
        let issues = match crate::setup::doctor::structure_problem(root) {
            Some(crate::setup::doctor::Problem::Issues(list)) => Some(list.len()),
            _ => None,
        };
        self.structure_seen = std::fs::metadata(root.join(structure::FILE)).and_then(|m| m.modified()).ok();
        self.apply_structure(Structure::load(root), issues, saved, all)
    }

    /// The structure that a worker read: keep the reason when it does not parse (IO-1).
    pub fn apply_structure(&mut self, loaded: Option<Result<Structure, String>>, issues: Option<usize>, saved: &str, all: bool) -> Option<Effect> {
        let mut said = None;
        match loaded {
            Some(Ok(s)) => {
                self.tree.structure = Some(s);
                self.structure_error = None;
                if let Some(n) = issues {
                    said = Effect::flash(format!("{}: {n} thing(s) zendo does not understand · zendo doctor --fix", structure::FILE));
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
        self.apply_tree_view(saved, all);
        said
    }

    /// The next tick looks at the disk, without the wait of a second.
    pub fn look_again(&mut self) {
        self.disk_checked = None;
    }
}
