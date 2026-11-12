//! Key input: the dispatch to the pane in focus.
use super::*;

impl App {
    pub fn switch_base(&mut self) {
        self.delta_base = if self.delta_base == Base::LastVisit { Base::GitHead } else { Base::LastVisit };
        self.delta_scroll = 0;
    }

    pub fn toggle_layout(&mut self) {
        self.state.layout = if self.state.wide() { "standard".into() } else { "wide".into() };
        self.state.save(&self.cfg.root);
        self.flash(if self.state.wide() { "wide screen: panes side by side" } else { "standard layout" });
    }

    /// Tab: the next pane on screen.
    pub(crate) fn next_pane(&mut self) {
        if self.visible.len() < 2 {
            return;
        }
        let i = self.visible.iter().position(|f| *f == self.focus).unwrap_or(0);
        self.focus = self.visible[(i + 1) % self.visible.len()];
    }

    pub fn on_key(&mut self, k: KeyEvent) {
        use crate::keys::Pane;
        self.last_input = Instant::now();
        // Any key ends a mouse selection; the text is already copied.
        self.sel = None;
        if self.mode == Mode::Help {
            self.mode = self.back;
            return;
        }
        if self.file_prompt.is_some() {
            return self.key_file_prompt(k);
        }
        if !matches!(self.mode, Mode::Read | Mode::Write) {
            if self.keys.local(Pane::Global, &k) == Some("quit") {
                self.quit = true;
                return;
            }
            return match self.mode {
                Mode::Notes => {
                    let act = self.keys.local(Pane::Notes, &k);
                    self.key_notes(act)
                }
                Mode::Search => self.key_search(k),
                Mode::Theme => self.key_theme(k),
                _ => {}
            };
        }
        if self.at.is_some() && self.key_pane() == Pane::Chat && self.key_at(k) {
            return;
        }
        if self.slash.open && self.key_pane() == Pane::Chat && self.key_slash(k) {
            return;
        }
        if self.tree.find.is_some() && self.key_pane() == Pane::Tree {
            return self.key_tree_find(k);
        }
        let pane = self.key_pane();
        let local = self.keys.local(pane, &k);
        if local.is_none()
            && let Some(act) = self.keys.local(Pane::Global, &k)
            && self.global_key(act, pane)
        {
            return;
        }
        match pane {
            Pane::Chat => self.key_panel(k, local),
            Pane::Delta => self.key_delta(local),
            Pane::Tree => self.key_tree(local),
            Pane::Write => self.key_write(k, local),
            _ => self.key_read(local),
        }
    }

    /// A key of no pane: quit, the tree, the text width, help, the next pane, the pane edges. True
    /// when it did something.
    fn global_key(&mut self, act: &str, pane: crate::keys::Pane) -> bool {
        match act {
            "quit" => {
                if self.mode == Mode::Write {
                    self.save();
                }
                self.quit = true;
            }
            "toggle_tree" => self.toggle_tree(),
            "wider" => self.change_width(4),
            "narrower" => self.change_width(-4),
            "help" => self.open_help(pane),
            "next_pane" => {
                self.next_pane();
                if pane == crate::keys::Pane::Chat && self.focus == Focus::Claude {
                    self.focus = Focus::Note;
                }
            }
            e @ ("edge_left" | "edge_right") if matches!(self.focus, Focus::Tree | Focus::Claude | Focus::Delta) => {
                let left = e == "edge_left";
                // The edge between this pane and the note moves; the tree's edge is on its right.
                let wider = if self.focus == Focus::Tree { !left } else { left };
                self.resize(self.focus, if wider { 4 } else { -4 });
            }
            _ => return false,
        }
        true
    }

    pub fn open_help(&mut self, pane: crate::keys::Pane) {
        self.help_pane = pane;
        self.back = self.mode;
        self.mode = Mode::Help;
    }

    /// The pane that gets the keys.
    pub fn key_pane(&self) -> crate::keys::Pane {
        use crate::keys::Pane;
        if self.panel && self.focus == Focus::Claude {
            Pane::Chat
        } else if self.delta_open && self.focus == Focus::Delta {
            Pane::Delta
        } else if self.state.tree && self.focus == Focus::Tree {
            Pane::Tree
        } else if self.mode == Mode::Write {
            Pane::Write
        } else {
            Pane::Read
        }
    }
}
