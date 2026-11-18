//! The keys of the file tree that change only the files: the moves in the tree, its name filter,
//! and the popup before a file action in a worktree view (FILES-37).
use super::{Effect, Files};
use crate::app::TreeAct;
use crate::docs::tree::Kind;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

/// What `enter` on a row of the tree gives.
#[derive(Debug, PartialEq, Eq)]
pub enum Picked {
    /// A folder opened or closed, or there is no row.
    Nothing,
    /// A file that the worktree deleted: it does not open (FILES-35).
    Deleted,
    /// A file to open, from the root of the repository.
    File(PathBuf),
}

/// What a key did in the name filter of the tree.
#[derive(Debug)]
pub enum FilterKey {
    Done,
    /// A file action on the selected match.
    Act(TreeAct),
    /// `enter`: open this file, from the root of the repository.
    Open(PathBuf),
}

impl Files {
    /// Enter on a tree row: the file to open, or open / close the folder.
    pub fn activate(&mut self) -> Picked {
        let rows = self.tree.rows();
        let Some(row) = rows.get(self.tree.sel).cloned() else { return Picked::Nothing };
        match row.kind {
            Kind::Dir { .. } => {
                self.tree.toggle(&row.rel);
                Picked::Nothing
            }
            Kind::File if self.tree.deleted.contains(&row.rel) => Picked::Deleted,
            Kind::File => Picked::File(row.rel),
            Kind::Head => Picked::Nothing,
        }
    }

    /// A key of the tree that only moves in it. False: the action is not one of them.
    pub fn key_tree(&mut self, act: &str) -> bool {
        let rows = self.tree.rows();
        let sel = self.tree.sel;
        let row = rows.get(sel);
        let notes = self.tree.view == crate::docs::tree::View::Notes;
        match act {
            "down" | "up" | "top" | "bottom" => self.move_in_tree(act, &rows),
            // FILES-69: the Notes view has no folders.
            "unfold" | "fold" | "expand_all" | "collapse_all" if notes => {}
            // Unfold or fold the folder: [+] / [-].
            "unfold" | "fold" => {
                let want_open = act == "unfold";
                if let Some(r) = row.filter(|r| matches!(r.kind, Kind::Dir { open } if open != want_open)) {
                    self.tree.toggle(&r.rel);
                }
            }
            "expand_all" => self.tree.expand_all(),
            "collapse_all" => self.tree.collapse_all(),
            // Close the folder, or jump to the parent folder.
            "close_folder" => match row {
                Some(r) if matches!(r.kind, Kind::Dir { open: true }) => self.tree.toggle(&r.rel),
                Some(r) => self.tree.sel = rows[..sel].iter().rposition(|p| p.depth + 1 == r.depth).unwrap_or(sel),
                None => {}
            },
            "search" => {
                self.tree.find = Some(String::new());
                self.tree.sel = 0;
            }
            _ => return false,
        }
        true
    }

    /// `j`, `k`, `g`, `G`: the selection moves, over the heads of the Notes view.
    fn move_in_tree(&mut self, act: &str, rows: &[crate::docs::tree::Row]) {
        let n = rows.len();
        if n == 0 {
            return;
        }
        let ok = |i: &usize| rows[*i].kind != Kind::Head;
        let sel = self.tree.sel;
        let to = match act {
            "down" => (sel + 1..n).find(ok),
            "up" => (0..sel).rev().find(ok),
            "top" => (0..n).find(ok),
            _ => (0..n).rev().find(ok),
        };
        self.tree.sel = to.unwrap_or(sel.min(n - 1));
    }

    /// Typing in the file name filter of the tree.
    pub fn key_filter(&mut self, k: KeyEvent) -> FilterKey {
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        let last = self.tree.rows().len().saturating_sub(1);
        let Some(empty) = self.tree.find.as_ref().map(String::is_empty) else { return FilterKey::Done };
        let alt = k.modifiers.contains(KeyModifiers::ALT);
        let typed = !ctrl && !alt;
        match k.code {
            KeyCode::Char('r') if alt => self.toggle_filter_regex(),
            KeyCode::Esc => {
                // The tree comes back with the match selected.
                let picked = self.tree_file();
                self.tree.find = None;
                if let Some(rel) = picked {
                    self.tree.reveal(&rel);
                }
            }
            // The keys of the files work on the selected match too.
            KeyCode::F(2) => return FilterKey::Act(TreeAct::File(self.tree_file(), 'r')),
            KeyCode::Delete => return FilterKey::Act(TreeAct::File(self.tree_file(), 'd')),
            KeyCode::Enter => {
                let picked = self.tree.rows().get(self.tree.sel).map(|r| r.rel.clone());
                self.tree.find = None;
                if let Some(rel) = picked.filter(|r| !self.tree.deleted.contains(r)) {
                    return FilterKey::Open(rel);
                }
            }
            KeyCode::Backspace if empty => self.tree.find = None,
            KeyCode::Backspace => self.edit_filter(|q| {
                q.pop();
            }),
            KeyCode::Down => self.tree.sel = (self.tree.sel + 1).min(last),
            KeyCode::Char('n' | 'j') if ctrl => self.tree.sel = (self.tree.sel + 1).min(last),
            KeyCode::Up => self.tree.sel = self.tree.sel.saturating_sub(1),
            KeyCode::Char('p' | 'k') if ctrl => self.tree.sel = self.tree.sel.saturating_sub(1),
            KeyCode::Char('u') if ctrl => self.edit_filter(String::clear),
            KeyCode::Char(c) if typed => self.edit_filter(|q| q.push(c)),
            _ => {}
        }
        FilterKey::Done
    }

    /// `[.*]` or `alt-r` in the filter: the text is a regular expression, or words again (FILES-59).
    pub fn toggle_filter_regex(&mut self) {
        self.tree.find_regex = !self.tree.find_regex;
        self.tree.sel = 0;
    }

    /// Change the text of the filter: the first match is selected.
    fn edit_filter(&mut self, edit: impl FnOnce(&mut String)) {
        if let Some(q) = self.tree.find.as_mut() {
            edit(q);
        }
        self.tree.sel = 0;
    }

    /// A file action from the tree. In a worktree view, while its agent answers (`works`), a popup
    /// asks first.
    pub fn tree_act(&mut self, act: TreeAct, works: bool) -> Option<Effect> {
        if self.tree.view.is_worktree() && works {
            self.worktree.confirm = Some(act);
            return None;
        }
        self.run_tree_act(act)
    }

    fn run_tree_act(&mut self, act: TreeAct) -> Option<Effect> {
        match act {
            TreeAct::File(rel, what) => self.ask_file(rel, what),
            TreeAct::New { folder } => {
                self.ask_new(folder);
                None
            }
        }
    }

    /// The popup is open: `y` goes on with the action; any other key drops it.
    pub fn key_confirm(&mut self, k: KeyEvent) -> Option<Effect> {
        let act = self.worktree.confirm.take()?;
        if matches!(k.code, KeyCode::Char('y' | 'Y')) { self.run_tree_act(act) } else { None }
    }

    /// The text of the popup, for agent session `session`, from 0.
    pub fn confirm_text(&self, session: usize) -> Option<String> {
        let act = self.worktree.confirm.as_ref()?;
        let n = session + 1;
        Some(format!("The agent of session {n} works in this worktree and can change the same files. Go on with the {}? y yes · any other key no", act.name()))
    }
}
