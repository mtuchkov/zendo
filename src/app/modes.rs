//! The modes of the screen; the delta, the notes list, search and the theme picker.
use super::*;

/// What the screen shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Read,
    Write,
    Notes,
    Find,
    Theme,
    Help,
}

impl App {
    pub(crate) fn key_delta(&mut self, act: Option<&str>) {
        let page = self.frame.view_height().saturating_sub(3).max(1);
        match act {
            Some("close") => {
                self.work.close_delta();
                self.focus = Focus::Note;
            }
            Some("base") => self.switch_base(),
            Some("ask") => self.ask_about_delta(),
            Some("recap") => self.recap_now(),
            Some("layout") => self.toggle_layout(),
            Some("write") if self.mode == Mode::Read => self.start_write(false),
            Some("down") => self.work.scroll_delta(false, 1),
            Some("up") => self.work.scroll_delta(true, 1),
            Some("page_down") => self.work.scroll_delta(false, page),
            Some("page_up") => self.work.scroll_delta(true, page),
            Some("top") => self.work.delta_top(),
            Some("bottom") => self.work.delta_bottom(),
            _ => return,
        }
        if matches!(act, Some("down" | "up" | "page_down" | "page_up")) {
            self.interact(1);
        }
    }

    pub fn open_notes(&mut self) {
        self.lists.select_note(self.note_index().unwrap_or(0));
        self.mode = Mode::Notes;
    }

    pub(crate) fn key_notes(&mut self, act: Option<&str>) {
        let n = self.files.notes().len();
        match act {
            Some("close") => self.mode = Mode::Read,
            Some("down") => self.lists.note_down(n),
            Some("up") => self.lists.note_up(),
            Some("top") => self.lists.select_note(0),
            Some("bottom") => self.lists.note_bottom(n),
            Some("open") if n > 0 => {
                self.files.show_note(self.lists.note());
                self.reader.to_start();
                self.threads.unpin();
                self.mode = Mode::Read;
            }
            Some("new") => self.open_float(),
            Some("rename") if n > 0 => self.rename(self.lists.note()),
            Some("rename_file") => self.ask_file(self.notes_file(), 'r'),
            Some("delete") => self.ask_file(self.notes_file(), 'd'),
            _ => {}
        }
    }

    /// Rename the file to match its title, keeping the date.
    pub(crate) fn rename(&mut self, i: usize) {
        let note = &self.files.notes()[i];
        let date = note.created.unwrap_or_else(|| note.modified.date_naive());
        let target = self.files.store().dir.join(format!("{}-{}.md", date.format("%Y-%m-%d"), notes::slug(&note.title)));
        if target == note.path {
            self.flash("the file name already matches the title");
            return;
        }
        if target.exists() {
            self.flash("a file with that name exists");
            return;
        }
        let old = note.path.clone();
        match self.move_file(&old, &target) {
            Ok(()) => {
                self.lists.select_note(self.files.notes().iter().position(|n| n.path == target).unwrap_or(0));
                self.flash(format!("renamed to {}", self.rel(&target)));
            }
            Err(e) => self.flash(format!("rename failed: {e}")),
        }
    }

    pub(crate) fn key_theme(&mut self, k: KeyEvent) {
        let n = THEMES.len();
        match k.code {
            KeyCode::Char('j') | KeyCode::Down => self.theme = (self.theme + 1) % n,
            KeyCode::Char('k') | KeyCode::Up => self.theme = (self.theme + n - 1) % n,
            KeyCode::Enter => {
                self.state.theme = THEMES[self.theme].name.to_string();
                self.state.save(&self.cfg.root);
                self.mode = Mode::Read;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.theme = self.theme_before;
                self.mode = Mode::Read;
            }
            _ => {}
        }
    }
}
