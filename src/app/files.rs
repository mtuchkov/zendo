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
            self.resume_log();
            self.cursor_to_editor();
            self.reader.keep_screen_row();
        } else {
            return self.start_write(true);
        }
        self.writer.not_saved();
        self.mode = Mode::Write;
        self.focus = Focus::Note;
    }

    /// Save the editor. A change of the file from outside comes first: the save does not write
    /// over it (UNDO-6).
    pub fn save(&mut self) {
        if let Some(body) = self.writer.path().and_then(|p| std::fs::read_to_string(p).ok()) {
            self.writer.editor_mut().external(body.trim_end_matches('\n'));
        }
        self.write_note();
    }

    /// Write the text of the editor to its file. A new note gets its file name from the title.
    pub(crate) fn write_note(&mut self) {
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
                // The loglet of the document takes what is on disk now (UNDO-16).
                self.writer.editor_mut().flush(text.trim_end());
                self.keep_log(&path);
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
    /// Changes made outside zendo: Claude Code, git pull, another editor. A worker looks at the
    /// disk each second; its answer reads the structure again or the notes (IO-1).
    pub(crate) fn check_disk(&mut self) {
        self.disk_answer();
        self.files.look_at_disk(&self.cfg.root);
    }

    /// The look at the disk answered: read the structure or the notes again.
    pub(crate) fn disk_answer(&mut self) {
        let Some((structure, changed)) = self.files.poll_look() else { return };
        if let Some((loaded, issues)) = structure {
            let said = self.files.apply_structure(loaded, issues, &self.state.tree_view, self.state.tree_all);
            self.apply(said);
            self.files.scan_again();
        }
        if changed {
            self.reload_after_agent();
            self.mark_tree();
        }
    }

    /// The work of each tick; each phase on the clock of the slow-tick log (IO-8).
    pub fn tick(&mut self) {
        let mut c = crate::worker::Clock::start("tick");
        c.time("poll_tasks", || self.poll_tasks());
        c.time("poll_find", || self.poll_find());
        c.time("music_tick", || self.music_tick());
        c.time("tips_tick", || self.tips_tick());
        c.time("hints_tick", || self.hints_tick());
        c.time("peek_tick", || self.peek_tick());
        // CMD-11: the message names no commit and no key; the top line has the button.
        if c.time("update", || self.update.poll().is_some()) {
            self.flash("a new version of zendo is available".to_string());
        }
        c.time("usage", || self.usage.tick(Instant::now()));
        c.time("background_tick", || self.background_tick());
        c.time("check_disk", || self.check_disk());
        c.time("tree", || self.tree_tick());
        c.time("sync_visit", || self.sync_visit());
        c.time("track_recent", || self.track_recent());
        c.time("worktree_recap_tick", || self.worktree_recap_tick());
        c.time("api_tick", || self.api_tick());
        c.time("recap_tick", || self.recap_tick());
        c.time("cli_tick", || self.cli_tick());
        c.time("board_tick", || self.board_tick());
        c.time("hold_tick", || self.hold_tick());
        c.time("attention_tick", || self.attention_tick());
        c.time("doc_tick", || self.doc_tick());
        c.time("saves", || self.save_tick());
        self.disk_failures();
        if self.flash.as_ref().is_some_and(|(_, t)| t.elapsed() > Duration::from_secs(4)) {
            self.flash = None;
        }
        self.note_slow(&c);
    }

    fn tree_tick(&mut self) {
        if self.state.tree && self.files.tree_stale() {
            self.scan_tree();
            self.mark_tree();
        }
        if self.files.poll_tree() {
            self.mark_tree();
        }
    }

    /// The drafts of the chat and the open document, when they are due.
    fn save_tick(&mut self) {
        if self.chat.due(Instant::now()) {
            self.save_drafts();
        }
        if self.mode == Mode::Write && self.writer.editor().dirty && self.writer.idle() > Duration::from_millis(1500) {
            self.save();
        }
    }

    /// The writes that failed, each file once (FAIL-1).
    fn disk_failures(&mut self) {
        for (path, e) in self.disk.failures() {
            if self.disk_failed.insert(path.clone()) {
                let rel = self.rel(&path);
                self.flash(format!("cannot write {rel}: {e}"));
            }
        }
    }

    /// In a debug build, a tick, a key or a frame over 100 ms goes to `.zendo/slow-ticks.log`, with
    /// its slowest phase (IO-8).
    pub fn note_slow(&self, c: &crate::worker::Clock) {
        if cfg!(debug_assertions)
            && let Some(line) = c.report()
        {
            self.disk.append(&self.cfg.root.join(".zendo").join("slow-ticks.log"), line.into_bytes());
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
