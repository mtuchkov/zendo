//! The save of the editor through the writer: a checked write, and what waits for it (UNDO-6,
//! IO-1, IO-6). The writer writes only over the text that zendo saw last on the disk; a change from
//! outside merges into the editor first, and the save goes again.
use super::*;
use crate::worker::Checked;

/// A save that waits for the answer of the writer.
pub struct Saving {
    id: u64,
    path: PathBuf,
    /// The text that went to the disk, without the new lines at its end.
    text: String,
}

impl App {
    /// Save the editor. A new note gets its file name from the title. Returns at once; a second
    /// save while one waits goes after it.
    pub fn save(&mut self) {
        if self.saving.is_some() {
            self.save_again = true;
            return;
        }
        let text = self.writer.editor_mut().text();
        let trimmed = text.trim_end().to_string();
        let empty = text.trim().is_empty() || text.trim() == "#";
        let body = format!("{trimmed}\n");
        let (path, expected) = match self.writer.path() {
            Some(p) => (p.clone(), Some(self.on_disk_text(p))),
            None if empty => return,
            None => {
                // Without a title: the first line of the body (FLOAT-17).
                let first = crate::agent::work::body_of(&text).lines().next().unwrap_or("note").to_string();
                let title = notes::title_of(&body).unwrap_or(first);
                (self.new_note_label(&title), None)
            }
        };
        let id = self.disk.write_checked(&path, body.into_bytes(), expected);
        self.saving = Some(Saving { id, path, text: trimmed });
    }

    /// The text of `path` on the disk as zendo last wrote or read it: an undo changes the loglet but
    /// not the disk (UNDO-6, UNDO-16).
    fn on_disk_text(&self, path: &Path) -> String {
        match &self.on_disk {
            Some((p, text)) if p == path => text.clone(),
            _ => self.writer.editor().logged().to_string(),
        }
    }

    /// Run `f` when the save that waits is on the disk, or at once when none waits (IO-6).
    pub(crate) fn when_saved(&mut self, f: impl FnOnce(&mut App) + 'static) {
        if self.saving.is_some() {
            self.after_save.push(Box::new(f));
        } else {
            f(self);
        }
    }

    /// The answers of the writer to the saves: keep the save, or merge the change from outside and
    /// save again, or take the next free name of a new note.
    pub(crate) fn poll_saves(&mut self) {
        for (id, answer) in self.disk.checked() {
            let Some(s) = self.saving.take_if(|s| s.id == id) else { continue };
            match answer {
                Checked::Written => self.saved(s),
                Checked::Changed(disk) => {
                    self.writer.editor_mut().external(&disk);
                    self.on_disk = Some((s.path, disk));
                    self.save();
                }
                Checked::Taken => {
                    self.facts.exists.put(s.path, true);
                    self.save();
                }
                Checked::Failed(e) => self.flash(format!("not saved: {e}")),
                Checked::Superseded => {}
            }
        }
        if self.saving.is_none() && std::mem::take(&mut self.save_again) && self.writer.editor().dirty {
            self.save();
        }
        if self.saving.is_none() {
            std::mem::take(&mut self.after_save).into_iter().for_each(|f| f(self));
        }
    }

    /// The text is on the disk: the loglet takes it (UNDO-16), and the notes read it again.
    fn saved(&mut self, s: Saving) {
        self.on_disk = Some((s.path.clone(), s.text.clone()));
        self.writer.editor_mut().flush(&s.text);
        self.keep_log(&s.path);
        self.writer.saved(s.path.clone());
        // A key that came while the save waited is not saved yet.
        if self.writer.editor_mut().text().trim_end() != s.text {
            self.writer.editor_mut().dirty = true;
        }
        self.files.saved(s.path);
        self.mark_edited();
    }

    /// For tests: wait at most 5 s for the saves, the jobs and the reload of the notes after them.
    pub fn settle_saves(&mut self) {
        let end = Instant::now() + Duration::from_secs(5);
        while (self.saving.is_some() || self.files.reloading() || self.serial.busy() || self.files.completes()) && Instant::now() < end {
            std::thread::sleep(Duration::from_millis(2));
            self.files.complete_waiting(&self.cfg.root);
            self.poll_jobs();
            self.poll_saves();
            self.poll_reload();
        }
    }

    /// After the last frame (IO-7): the saves, the jobs and the writes that wait get a moment to
    /// end.
    pub fn finish_io(&mut self) {
        self.finish_saves(Duration::from_secs(5));
        let done = self.serial.flush(Duration::from_secs(5));
        drop(done);
        self.disk.flush(Duration::from_secs(5));
    }

    /// After the last frame: wait at most `limit` for the saves, and take their answers (IO-7).
    pub fn finish_saves(&mut self, limit: Duration) {
        let end = Instant::now() + limit;
        while self.saving.is_some() && Instant::now() < end {
            std::thread::sleep(Duration::from_millis(2));
            self.poll_saves();
        }
    }
}
