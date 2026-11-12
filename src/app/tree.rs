//! The file tree: views, the structure, opening files, the name filter.
use super::*;

impl App {
    pub fn toggle_tree(&mut self) {
        self.state.tree = !self.state.tree;
        self.state.save(&self.cfg.root);
        if self.state.tree {
            self.apply_tree_view();
            self.scan_tree();
            self.mark_tree();
            if let Some(p) = self.current.clone() {
                let rel = self.store.rel(&p).to_path_buf();
                self.tree.reveal(&rel);
            }
            self.focus = Focus::Tree;
        } else if self.focus == Focus::Tree {
            self.focus = Focus::Note;
        }
    }

    /// The saved tree view, else the structure when the repo has one, else docs.
    pub fn apply_tree_view(&mut self) {
        use crate::docs::tree::View;
        let has = self.tree.structure.as_ref().is_some_and(|s| s.main().next().is_some());
        self.tree.view = match View::from_name(&self.state.tree_view) {
            Some(View::Structure) if !has => View::Docs,
            Some(v) => v,
            None if self.state.tree_all => View::All,
            None if has => View::Structure,
            None => View::Docs,
        };
    }

    /// Read `project-structure.toml` again; keep the reason when it does not parse.
    pub fn load_structure(&mut self) {
        match crate::docs::structure::Structure::load(&self.cfg.root) {
            Some(Ok(s)) => {
                self.tree.structure = Some(s);
                self.structure_error = None;
                if let Some(crate::setup::doctor::Problem::Issues(list)) = crate::setup::doctor::structure_problem(&self.cfg.root) {
                    self.flash(format!("{}: {} thing(s) zendo does not understand · zendo doctor --fix", crate::docs::structure::FILE, list.len()));
                }
            }
            Some(Err(e)) => {
                self.structure_error = Some(e.lines().next().unwrap_or("").to_string());
                self.flash(format!("{} does not parse · zendo doctor --fix", crate::docs::structure::FILE));
            }
            None => {
                self.tree.structure = None;
                self.structure_error = None;
            }
        }
        self.structure_seen = std::fs::metadata(self.cfg.root.join(crate::docs::structure::FILE)).and_then(|m| m.modified()).ok();
        self.apply_tree_view();
    }

    pub fn toggle_tree_filter(&mut self) {
        self.tree.next_view();
        self.state.tree_view = self.tree.view.name().to_string();
        self.state.save(&self.cfg.root);
        self.scan_tree();
        self.tree.sel = 0;
        self.tree.scroll = 0;
        if let Some(p) = self.current.clone() {
            let rel = self.store.rel(&p).to_path_buf();
            self.tree.reveal(&rel);
        }
    }

    /// Enter on a tree row: open the file, or open / close the folder.
    pub fn tree_activate(&mut self, focus_note: bool) {
        let rows = self.tree.rows();
        let Some(row) = rows.get(self.tree.sel).cloned() else { return };
        match row.kind {
            crate::docs::tree::Kind::Dir { .. } => self.tree.toggle(&row.rel),
            crate::docs::tree::Kind::File => {
                self.open_file(self.cfg.root.join(&row.rel));
                if focus_note {
                    self.focus = Focus::Note;
                }
            }
        }
    }

    pub(crate) fn key_tree(&mut self, act: Option<&str>) {
        let rows = self.tree.rows();
        let n = rows.len();
        match act {
            Some("close") => self.toggle_tree(),
            Some("down") if n > 0 => self.tree.sel = (self.tree.sel + 1).min(n - 1),
            Some("up") => self.tree.sel = self.tree.sel.saturating_sub(1),
            Some("top") => self.tree.sel = 0,
            Some("bottom") => self.tree.sel = n.saturating_sub(1),
            Some("open") => self.tree_activate(true),
            Some("preview") => self.tree_activate(false),
            Some(a @ ("unfold" | "fold")) => {
                // Unfold or fold the folder: [+] / [-].
                if let Some(row) = rows.get(self.tree.sel) {
                    let want_open = a == "unfold";
                    if matches!(row.kind, crate::docs::tree::Kind::Dir { open } if open != want_open) {
                        self.tree.toggle(&row.rel.clone());
                    }
                }
            }
            Some("close_folder") => {
                // Close the folder, or jump to the parent folder.
                if let Some(row) = rows.get(self.tree.sel) {
                    if matches!(row.kind, crate::docs::tree::Kind::Dir { open: true }) {
                        self.tree.toggle(&row.rel.clone());
                    } else if let Some(i) = rows[..self.tree.sel].iter().rposition(|r| r.depth + 1 == row.depth) {
                        self.tree.sel = i;
                    }
                }
            }
            Some("view") => self.toggle_tree_filter(),
            Some("search") => {
                self.tree.find = Some(String::new());
                self.tree.sel = 0;
            }
            Some("delete") => self.ask_file(self.tree_file(), 'd'),
            Some("rename") => self.ask_file(self.tree_file(), 'r'),
            Some("move") => self.ask_file(self.tree_file(), 'm'),
            Some("new_file") => self.ask_new(false),
            Some("new_folder") => self.ask_new(true),
            Some("add_to_chat") => self.attach_tree_file(),
            Some("layout") => self.toggle_layout(),
            Some("delta") => self.toggle_delta(),
            Some("chat") => self.toggle_chat(),
            _ => {}
        }
    }

    /// Typing in the file name filter of the tree.
    pub(crate) fn key_tree_find(&mut self, k: KeyEvent) {
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        let n = self.tree.rows().len();
        let Some(q) = self.tree.find.as_mut() else { return };
        match k.code {
            KeyCode::Esc => {
                // The tree comes back with the match selected.
                let picked = self.tree_file();
                self.tree.find = None;
                if let Some(rel) = picked {
                    self.tree.reveal(&rel);
                }
            }
            // The keys of the files work on the selected match too.
            KeyCode::F(2) => self.ask_file(self.tree_file(), 'r'),
            KeyCode::Delete => self.ask_file(self.tree_file(), 'd'),
            KeyCode::Enter => {
                let picked = self.tree.rows().get(self.tree.sel).map(|r| r.rel.clone());
                self.tree.find = None;
                if let Some(rel) = picked {
                    self.open_file(self.cfg.root.join(&rel));
                    self.tree.reveal(&rel);
                    self.focus = Focus::Note;
                }
            }
            KeyCode::Backspace if q.is_empty() => self.tree.find = None,
            KeyCode::Backspace => {
                q.pop();
                self.tree.sel = 0;
            }
            KeyCode::Down => self.tree.sel = (self.tree.sel + 1).min(n.saturating_sub(1)),
            KeyCode::Char('n' | 'j') if ctrl => self.tree.sel = (self.tree.sel + 1).min(n.saturating_sub(1)),
            KeyCode::Up => self.tree.sel = self.tree.sel.saturating_sub(1),
            KeyCode::Char('p' | 'k') if ctrl => self.tree.sel = self.tree.sel.saturating_sub(1),
            KeyCode::Char('u') if ctrl => {
                q.clear();
                self.tree.sel = 0;
            }
            KeyCode::Char(c) if !ctrl && !k.modifiers.contains(KeyModifiers::ALT) => {
                q.push(c);
                self.tree.sel = 0;
            }
            _ => {}
        }
    }

    /// `a` in the tree: the file goes to the chat as an attachment; the focus stays in the tree.
    fn attach_tree_file(&mut self) {
        let Some(rel) = self.tree_file() else { return self.flash("select a file first") };
        self.attach(Attachment::of(self.cfg.root.join(rel)));
        let back = self.focus;
        self.open_panel();
        if self.panel {
            self.focus = back;
        }
    }
}
