//! Files: open, reload, write mode, save, and changes on disk.
use super::*;

impl App {
    pub fn reload(&mut self) {
        if let Ok(mut n) = self.store.load_all() {
            self.extra.retain(|p| p.exists());
            for p in &self.extra {
                if !n.iter().any(|x| &x.path == p)
                    && let Ok(note) = notes::load(p) {
                        n.push(note);
                    }
            }
            n.sort_by_key(|b| std::cmp::Reverse(b.modified));
            self.notes = n;
        }
        if self.current.as_ref().is_some_and(|p| !p.exists()) {
            self.current = self.notes.first().map(|n| n.path.clone());
        }
    }

    pub fn start_write(&mut self, new: bool) {
        self.pinned = None;
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
            self.keep_y = Some(self.cursor_y);
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
                let p = self.store.path_for(&title, Local::now().date_naive());
                // The thread of the unsaved note moves to the new file.
                if let Some(t) = self.threads.remove("(new note)") {
                    self.threads.insert(self.rel(&p), t);
                }
                p
            }
        };
        match notes::save(&path, &body) {
            Ok(()) => {
                self.editor.dirty = false;
                self.saved_at = Some(Local::now());
                self.editor_path = Some(path.clone());
                self.current = Some(path);
                self.reload();
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
        self.keep_y = Some(self.cursor_y);
    }

    /// Pick up changes made outside zendo: Claude Code, git pull, another editor.
    pub(crate) fn check_disk(&mut self) {
        if self.disk_checked.elapsed() < Duration::from_secs(1) {
            return;
        }
        self.disk_checked = Instant::now();
        let st = std::fs::metadata(self.cfg.root.join(crate::docs::structure::FILE)).and_then(|m| m.modified()).ok();
        if st != self.structure_seen {
            self.load_structure();
            self.tree.scanned = None;
        }
        let Ok(dir) = std::fs::read_dir(&self.store.dir) else { return };
        let mut n = 0;
        let mut newest = None;
        for e in dir.flatten() {
            if e.path().extension().is_some_and(|x| x == "md") {
                n += 1;
                let m = e.metadata().and_then(|m| m.modified()).ok();
                if m > newest {
                    newest = m;
                }
            }
        }
        if let Some(m) = self.current.as_ref().and_then(|p| std::fs::metadata(p).ok()).and_then(|m| m.modified().ok())
            && Some(m) > newest {
                newest = Some(m);
            }
        let now = (n, newest);
        if now != self.disk {
            let first = self.disk.1.is_none() && self.disk.0 == 0;
            self.disk = now;
            if !first {
                self.reload_after_agent();
                self.mark_tree();
            }
        }
    }

    pub fn tick(&mut self) {
        if let Some(rx) = self.update_rx.as_mut() {
            match rx.try_recv() {
                Ok(found) => {
                    self.update_rx = None;
                    if let Some(sha) = found {
                        self.flash(format!("a newer zendo is on GitHub ({sha}) · U upgrades now"));
                        self.update = Some(sha);
                    }
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Closed) => self.update_rx = None,
                Err(_) => {}
            }
        }
        self.check_disk();
        if self.state.tree && self.tree.stale() {
            self.scan_tree();
            self.mark_tree();
        }
        self.sync_visit();
        self.recap_tick();
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

    /// Open any markdown file of the repo.
    pub fn open_file(&mut self, path: PathBuf) {
        if self.mode == Mode::Write {
            self.save();
            self.mode = Mode::Read;
        }
        if !self.notes.iter().any(|n| n.path == path) && !self.extra.contains(&path) {
            self.extra.push(path.clone());
            self.reload();
        }
        self.current = Some(path);
        self.scroll = 0;
        self.read_cursor = 0;
        self.read_anchor = None;
        self.delta_scroll = 0;
        self.pinned = None;
        if !matches!(self.mode, Mode::Read | Mode::Write) {
            self.mode = Mode::Read;
        }
    }
}
