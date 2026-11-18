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
            self.follow_open_file();
            self.focus = Focus::Tree;
        } else if self.focus == Focus::Tree {
            self.focus = Focus::Note;
        }
    }

    /// While the tree shows, it shows the open document now, also when it did not change: the
    /// pane opens, the session changes, a file opens (FILES-58). Each draw follows other changes.
    pub fn follow_open_file(&mut self) {
        self.files.follow_again();
        if self.state.tree {
            self.files.follow_current();
        }
    }

    /// The saved tree view, else the structure when the repo has one, else docs.
    pub fn apply_tree_view(&mut self) {
        self.files.apply_tree_view(&self.state.tree_view, self.state.tree_all);
    }

    /// Read `project-structure.toml` again; keep the reason when it does not parse.
    pub fn load_structure(&mut self) {
        let said = self.files.load_structure(&self.cfg.root, &self.state.tree_view, self.state.tree_all);
        self.apply(said);
    }

    pub fn toggle_tree_filter(&mut self) {
        self.state.tree_view = self.files.next_view().to_string();
        self.state.save(&self.cfg.root);
        self.scan_tree();
        self.files.reveal_current();
    }

    /// Enter on a tree row: open the file, or open / close the folder.
    pub fn tree_activate(&mut self, focus_note: bool) {
        match self.files.activate() {
            Picked::Nothing => {}
            Picked::Deleted => self.flash("deleted in the worktree: it does not open"),
            Picked::File(rel) => {
                self.open_file(self.cfg.root.join(rel));
                if focus_note {
                    self.focus = Focus::Note;
                }
            }
        }
    }

    pub(crate) fn key_tree(&mut self, act: Option<&str>) {
        let Some(act) = act.filter(|a| !self.files.key_tree(a)) else { return };
        let file = |a: &Self, what| TreeAct::File(a.files.tree_file(), what);
        match act {
            "close" => self.toggle_tree(),
            "open" => self.tree_activate(true),
            "preview" => self.tree_activate(false),
            "view" => self.toggle_tree_filter(),
            "delete" => self.tree_act(file(self, 'd')),
            "rename" => self.tree_act(file(self, 'r')),
            "move" => self.tree_act(file(self, 'm')),
            "new_file" => self.tree_act(TreeAct::New { folder: false }),
            "new_folder" => self.tree_act(TreeAct::New { folder: true }),
            "add_to_chat" => self.attach_tree_file(),
            "layout" => self.toggle_layout(),
            "delta" => self.toggle_delta(),
            "chat" => self.toggle_chat(),
            _ => {}
        }
    }

    /// Typing in the file name filter of the tree.
    pub(crate) fn key_tree_find(&mut self, k: KeyEvent) {
        // The filter takes the keys of `[textbox]`; the files do the typing (TEXT-1).
        if let Some(changed) = self.field_action(&k, |a| a.files.filter_field()) {
            if changed {
                self.files.filter_changed();
            }
            return;
        }
        let act = self.keys.local(crate::keys::Pane::List, &k);
        match self.files.key_filter(k, act) {
            FilterKey::Done => {}
            FilterKey::Act(act) => self.tree_act(act),
            FilterKey::Open(rel) => {
                self.open_file(self.cfg.root.join(&rel));
                self.files.reveal(&rel);
                self.focus = Focus::Note;
            }
        }
    }

    /// `[+]` or `[-]` in the head of the tree: open or close every folder (FILES-57).
    pub fn fold_all(&mut self, open: bool) {
        self.focus = Focus::Tree;
        self.did_by_mouse(crate::keys::Pane::Tree, if open { "expand_all" } else { "collapse_all" });
        if open { self.files.expand_all() } else { self.files.collapse_all() }
    }

    /// `a` in the tree: the file goes to the chat as an attachment; the focus stays in the tree.
    fn attach_tree_file(&mut self) {
        let Some(rel) = self.files.tree_file() else { return self.flash("select a file first") };
        self.attach(Attachment::of(self.cfg.root.join(rel)));
        let back = self.focus;
        self.open_panel();
        if self.threads.shown() {
            self.focus = back;
        }
    }
}
