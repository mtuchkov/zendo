//! Files: open, reload, write mode, save, and changes on disk.
use super::*;

impl App {
    pub fn reload(&mut self) {
        self.files.reload();
    }

    pub fn start_write(&mut self, new: bool) {
        self.threads.unpin();
        if new {
            self.editor = Editor::new("# ");
            self.editor.move_to_end();
            self.editor_path = None;
        } else if let Some(n) = self.note() {
            let body = n.body.clone();
            let path = n.path.clone();
            self.editor = Editor::new(body.trim_end_matches('\n'));
            self.editor_path = Some(path);
            self.cursor_to_editor();
            self.reader.keep_screen_row();
        } else {
            return self.start_write(true);
        }
        self.saved_at = None;
        self.mode = Mode::Write;
        self.focus = Focus::Note;
    }

    /// Save the editor. A new note gets its file name from the title.
    pub fn save(&mut self) {
        let text = self.editor.text();
        let body = format!("{}\n", text.trim_end());
        let empty = text.trim().is_empty() || text.trim() == "#";
        let path = match &self.editor_path {
            Some(p) => p.clone(),
            None if empty => return,
            None => {
                let title = notes::title_of(&body).unwrap_or_else(|| text.lines().next().unwrap_or("note").to_string());
                self.new_note_path(&title)
            }
        };
        match notes::save(&path, &body) {
            Ok(()) => {
                self.editor.dirty = false;
                self.saved_at = Some(Local::now());
                self.editor_path = Some(path.clone());
                self.files.saved(path);
                self.mark_edited();
            }
            Err(e) => self.flash(format!("not saved: {e}")),
        }
    }

    pub fn finish_write(&mut self) {
        self.save();
        self.ask_for_meta();
        self.mode = Mode::Read;
        // The read cursor on the char of the write cursor, at the same height on the screen.
        self.cursor_to_read();
        self.reader.keep_screen_row();
    }

    /// Pick up changes made outside zendo: Claude Code, git pull, another editor.
    pub(crate) fn check_disk(&mut self) {
        let Some(structure) = self.files.disk_due(&self.cfg.root) else { return };
        if structure {
            self.load_structure();
            self.files.tree.scanned = None;
        }
        if self.files.disk_changed() {
            self.reload_after_agent();
            self.mark_tree();
        }
    }

    pub fn tick(&mut self) {
        self.music_tick();
        self.peek_tick();
        if let Some(rx) = self.update_rx.as_mut() {
            match rx.try_recv() {
                Ok(found) => {
                    self.update_rx = None;
                    if let Some(sha) = found {
                        let how = self.keys.key(crate::keys::Pane::Read, "upgrade").map(|k| format!(" · {k} upgrades now")).unwrap_or_default();
                        self.flash(format!("a newer zendo is on GitHub ({sha}){how}"));
                        self.update = Some(sha);
                    }
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Closed) => self.update_rx = None,
                Err(_) => {}
            }
        }
        self.usage.tick(Instant::now());
        self.check_disk();
        if self.state.tree && self.files.tree.stale() {
            self.scan_tree();
            self.mark_tree();
        }
        self.sync_visit();
        self.track_recent();
        self.worktree_recap_tick();
        self.recap_tick();
        self.cli_tick();
        self.hold_tick();
        self.attention_tick();
        if self.mode == Mode::Write && self.editor.dirty && self.last_edit.elapsed() > Duration::from_millis(1500) {
            self.save();
        }
        if self.flash.as_ref().is_some_and(|(_, t)| t.elapsed() > Duration::from_secs(4)) {
            self.flash = None;
        }
    }

    /// Leave, build the newest zendo, and start again (see main.rs).
    pub fn upgrade(&mut self) {
        if self.mode == Mode::Write {
            self.save();
        }
        self.upgrade_now = true;
        self.quit = true;
    }

    /// Open any file of the repo, in the version of the active agent session (FILES-42).
    pub fn open_file(&mut self, path: PathBuf) {
        if self.mode == Mode::Write {
            self.save();
            self.mode = Mode::Read;
        }
        let path = self.session_path(&path);
        self.files.open(path);
        self.reader.to_top();
        self.work.delta_top();
        self.threads.unpin();
        if !matches!(self.mode, Mode::Read | Mode::Write) {
            self.mode = Mode::Read;
        }
    }
}
