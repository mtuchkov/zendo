//! Key input: the dispatch to the pane in focus.
use super::*;

impl App {
    pub fn switch_base(&mut self) {
        self.work.switch_base();
    }

    /// The draw: the focus stays on a pane that is on the screen, left to right in `panes`.
    pub(crate) fn keep_focus_in(&mut self, panes: &[Focus]) {
        if !panes.contains(&self.focus) {
            self.focus = panes[0];
        }
    }

    pub fn toggle_layout(&mut self) {
        self.state.layout = if self.state.wide() { "standard".into() } else { "wide".into() };
        self.state.save(&self.cfg.root);
        self.flash(if self.state.wide() { "wide screen: panes side by side" } else { "standard layout" });
    }

    /// Tab: the next pane on screen.
    pub(crate) fn next_pane(&mut self) {
        if self.frame.visible().len() < 2 {
            return;
        }
        let i = self.frame.visible().iter().position(|f| *f == self.focus).unwrap_or(0);
        self.focus = self.frame.visible()[(i + 1) % self.frame.visible().len()];
    }

    /// What any key ends: `m` or `u` held in read mode, a run of `alt-o` or `tab`, a mouse
    /// selection. True when the key went up: then it is done (FILES-45, FILES-47).
    fn key_comes(&mut self, k: &KeyEvent) -> bool {
        if self.peek_key(k) {
            return true;
        }
        self.track_recent();
        self.work.input_now();
        self.tab_run.key();
        // Only `alt-o` right after `alt-o` takes the next terminal (TERM-28).
        if self.keys.local(crate::keys::Pane::Chat, k) != Some("insert_output") {
            self.composer.drop_pick();
        }
        // Any key ends a mouse selection; the text is already copied.
        self.pointer.clear();
        false
    }

    /// The views that take a key before the panes: the music question, the float, the waiting view, a session on the
    /// full screen, a terminal in the chat pane. True when one of them took it.
    fn key_to_view(&mut self, k: KeyEvent) -> bool {
        // The float and find open over every pane and view, also a session on the full screen
        // (FLOAT-14, FIND-15).
        let global = if self.mode == Mode::Help { None } else { self.keys.local(crate::keys::Pane::Global, &k) };
        match (self.cli.view, self.pane_terminal()) {
            // The question of the first start takes every key until it has an answer (LOFI-12).
            _ if self.music.asking => self.key_music_question(k),
            // The key float takes every key: the next key is the new key (REKEY-2, REKEY-14).
            _ if self.rekey.float().is_some() => self.key_key_float(k),
            _ if self.writer.float().is_some() => self.key_float(k),
            _ if global == Some("new_note") => drop(self.global_key("new_note", crate::keys::Pane::Global)),
            _ if global == Some("music") => drop(self.global_key("music", crate::keys::Pane::Global)),
            _ if global == Some("find") && self.mode != Mode::Find => drop(self.global_key("find", crate::keys::Pane::Global)),
            _ if self.mode == Mode::Find => return false,
            _ if self.hold.is_some() => self.key_hold(k),
            (Some(n), _) => self.key_cli(n, k),
            (None, Some(n)) => self.key_pane_terminal(n, k),
            (None, None) => return false,
        }
        true
    }

    pub fn on_key(&mut self, k: KeyEvent) {
        use crate::keys::Pane;
        if self.key_comes(&k) || self.key_to_view(k) {
            return;
        }
        if self.mode == Mode::Help {
            self.mode = self.back;
            return;
        }
        // f1 shows the keys of the pane or the mode, from everywhere.
        if self.keys.local(Pane::Global, &k) == Some("help") {
            self.did(Pane::Global, "help");
            return self.open_help(self.help_pane_now());
        }
        if self.files.worktree.confirm.is_some() {
            return self.key_confirm(k);
        }
        if self.files.prompt().is_some() {
            return self.key_file_prompt(k);
        }
        if !matches!(self.mode, Mode::Read | Mode::Write) {
            if self.keys.local(Pane::Global, &k) == Some("quit") {
                self.did(Pane::Global, "quit");
                self.quit = true;
                return;
            }
            return match self.mode {
                Mode::Notes => {
                    let act = self.keys.local(Pane::Notes, &k);
                    self.did_some(Pane::Notes, act);
                    self.key_notes(act)
                }
                Mode::Search => self.key_search(k),
                Mode::Find => self.key_find(k),
                Mode::Theme => self.key_theme(k),
                _ => {}
            };
        }
        if self.composer.at().is_some() && self.key_pane() == Pane::Chat && self.key_at(k) {
            return;
        }
        if self.composer.slash().open && self.key_pane() == Pane::Chat && self.composer.key_slash(k) {
            return;
        }
        if self.files.filtering() && self.key_pane() == Pane::Tree {
            return self.key_tree_find(k);
        }
        let pane = self.key_pane();
        if self.selection_key(pane, &k) {
            return;
        }
        let local = self.keys.local(pane, &k);
        if local.is_none()
            && let Some(act) = self.keys.local(Pane::Global, &k)
            && self.global_key(act, pane)
        {
            return;
        }
        self.did_some(pane, local);
        match pane {
            Pane::Chat => self.key_panel(k, local),
            Pane::Transcript => self.key_transcript(local),
            Pane::Delta => self.key_delta(local),
            Pane::Tree => self.key_tree(local),
            Pane::Write => self.key_write(k, local),
            _ => self.key_read(local),
        }
    }

    /// A key of no pane: quit, the tree, the text width, help, the next pane, the pane edges. True
    /// when it did something; then the action counts (USAGE-1).
    pub(super) fn global_key(&mut self, act: &str, pane: crate::keys::Pane) -> bool {
        let ran = self.run_global(act, pane);
        if ran {
            self.did(crate::keys::Pane::Global, act);
        }
        ran
    }

    fn run_global(&mut self, act: &str, pane: crate::keys::Pane) -> bool {
        match act {
            "quit" => {
                if self.mode == Mode::Write {
                    self.save();
                }
                self.quit = true;
            }
            "toggle_tree" => self.toggle_tree(),
            "new_note" => self.open_float(),
            "music" => self.toggle_music(),
            "find" => self.open_find(),
            "next_session" => self.step_in_chat(true),
            "prev_session" => self.step_in_chat(false),
            "terminal" => self.terminal_key(false),
            "terminal_run" => self.terminal_key(true),
            s if s.starts_with("session_") => self.switch_session(s[8..].parse::<usize>().map_or(0, |n| n - 1)),
            "wider" => self.change_width(4),
            "narrower" => self.change_width(-4),
            "help" => self.open_help(pane),
            "next_pane" => self.tab(pane),
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

    /// The pane whose keys the help shows: the pane in focus, the notes list, or only the global keys.
    fn help_pane_now(&self) -> crate::keys::Pane {
        match self.mode {
            Mode::Read | Mode::Write => self.key_pane(),
            Mode::Notes => crate::keys::Pane::Notes,
            _ => crate::keys::Pane::Global,
        }
    }

    pub fn open_help(&mut self, pane: crate::keys::Pane) {
        self.key_help.show(pane);
        self.back = self.mode;
        self.mode = Mode::Help;
    }

    /// The pane that gets the keys.
    pub fn key_pane(&self) -> crate::keys::Pane {
        use crate::keys::Pane;
        if self.threads.shown() && self.focus == Focus::Claude && self.threads.transcript().on() {
            Pane::Transcript
        } else if self.threads.shown() && self.focus == Focus::Claude {
            Pane::Chat
        } else if self.work.delta_open() && self.focus == Focus::Delta {
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
