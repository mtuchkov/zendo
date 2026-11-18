//! Files: open, reload, write mode, save, and changes on disk.
use super::*;

impl App {
    pub fn reload(&mut self) {
        self.files.reload();
    }

    pub fn start_write(&mut self, new: bool) {
        self.threads.unpin();
        if new {
            self.writer.start_new();
        } else if let Some(n) = self.note() {
            let body = n.body.clone();
            let path = n.path.clone();
            self.writer.start_on(body.trim_end_matches('\n'), path);
            self.cursor_to_editor();
            self.reader.keep_screen_row();
        } else {
            return self.start_write(true);
        }
        self.writer.not_saved();
        self.mode = Mode::Write;
        self.focus = Focus::Note;
    }

    /// Save the editor. A new note gets its file name from the title.
    pub fn save(&mut self) {
        let text = self.writer.editor_mut().text();
        let body = format!("{}\n", text.trim_end());
        let empty = text.trim().is_empty() || text.trim() == "#";
        let path = match self.writer.path() {
            Some(p) => p.clone(),
            None if empty => return,
            None => {
                // Without a title: the first line of the body (FLOAT-17).
                let first = crate::agent::work::body_of(&text).lines().next().unwrap_or("note").to_string();
                let title = notes::title_of(&body).unwrap_or(first);
                self.new_note_path(&title)
            }
        };
        match notes::save(&path, &body) {
            Ok(()) => {
                self.writer.saved(path.clone());
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
            self.files.scan_again();
        }
        if self.files.disk_changed() {
            self.reload_after_agent();
            self.mark_tree();
        }
    }

    pub fn tick(&mut self) {
        self.music_tick();
        self.hints_tick();
        self.peek_tick();
        // CMD-11: the message names no commit and no key; the top line has the button.
        if self.update.poll().is_some() {
            self.flash("a new version of zendo is available".to_string());
        }
        self.usage.tick(Instant::now());
        self.check_disk();
        if self.state.tree && self.files.tree_stale() {
            self.scan_tree();
            self.mark_tree();
        }
        if self.files.poll_tree() {
            self.mark_tree();
        }
        self.sync_visit();
        self.track_recent();
        self.worktree_recap_tick();
        self.api_tick();
        self.recap_tick();
        self.cli_tick();
        self.hold_tick();
        self.attention_tick();
        if self.mode == Mode::Write && self.writer.editor().dirty && self.writer.idle() > Duration::from_millis(1500) {
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
        self.update.upgrade();
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
        // FILES-58: the tree shows the file, also when it was open before.
        self.follow_open_file();
        self.reader.to_top();
        self.work.delta_top();
        self.threads.unpin();
        if !matches!(self.mode, Mode::Read | Mode::Write) {
            self.mode = Mode::Read;
        }
    }
}
