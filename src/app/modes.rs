//! The delta, the notes list, search and the theme picker.
use super::*;

impl App {
    pub(crate) fn key_delta(&mut self, act: Option<&str>) {
        let page = self.view_height.saturating_sub(3).max(1);
        match act {
            Some("close") => {
                self.delta_open = false;
                self.focus = Focus::Note;
            }
            Some("base") => self.switch_base(),
            Some("ask") => self.ask_about_delta(),
            Some("recap") => self.recap_now(),
            Some("layout") => self.toggle_layout(),
            Some("write") if self.mode == Mode::Read => self.start_write(false),
            Some("down") => self.delta_scroll += 1,
            Some("up") => self.delta_scroll = self.delta_scroll.saturating_sub(1),
            Some("page_down") => self.delta_scroll += page,
            Some("page_up") => self.delta_scroll = self.delta_scroll.saturating_sub(page),
            Some("top") => self.delta_scroll = 0,
            Some("bottom") => self.delta_scroll = usize::MAX / 2,
            _ => return,
        }
        if matches!(act, Some("down" | "up" | "page_down" | "page_up")) {
            self.interact(1);
        }
    }

    pub fn open_notes(&mut self) {
        self.notes_sel = self.note_index().unwrap_or(0);
        self.mode = Mode::Notes;
    }

    pub(crate) fn key_notes(&mut self, act: Option<&str>) {
        let n = self.notes.len();
        match act {
            Some("close") => self.mode = Mode::Read,
            Some("down") if n > 0 => self.notes_sel = (self.notes_sel + 1).min(n - 1),
            Some("up") => self.notes_sel = self.notes_sel.saturating_sub(1),
            Some("top") => self.notes_sel = 0,
            Some("bottom") => self.notes_sel = n.saturating_sub(1),
            Some("open") if n > 0 => {
                self.current = Some(self.notes[self.notes_sel].path.clone());
                self.scroll = 0;
                self.read_cursor = 0;
                self.read_col = 0;
                self.read_anchor = None;
                self.pinned = None;
                self.mode = Mode::Read;
            }
            Some("new") => self.start_write(true),
            Some("search") => self.open_search(""),
            Some("tags") => self.open_search("#"),
            Some("rename") if n > 0 => self.rename(self.notes_sel),
            Some("rename_file") => self.ask_file(self.notes_file(), 'r'),
            Some("delete") => self.ask_file(self.notes_file(), 'd'),
            _ => {}
        }
    }

    /// Rename the file to match its title, keeping the date.
    pub(crate) fn rename(&mut self, i: usize) {
        let note = &self.notes[i];
        let date = note.created.unwrap_or_else(|| note.modified.date_naive());
        let target = self.store.dir.join(format!("{}-{}.md", date.format("%Y-%m-%d"), notes::slug(&note.title)));
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
                self.notes_sel = self.notes.iter().position(|n| n.path == target).unwrap_or(0);
                self.flash(format!("renamed to {}", self.rel(&target)));
            }
            Err(e) => self.flash(format!("rename failed: {e}")),
        }
    }

    pub fn open_search(&mut self, q: &str) {
        self.query = q.to_string();
        self.run_search();
        self.mode = Mode::Search;
    }

    pub(crate) fn run_search(&mut self) {
        self.results = if self.query.starts_with('?') { vec![] } else { search::search(&self.notes, &self.query) };
        self.result_sel = 0;
    }

    pub(crate) fn key_search(&mut self, k: KeyEvent) {
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        let n = self.results.len();
        match k.code {
            KeyCode::Esc => self.mode = Mode::Read,
            KeyCode::Down | KeyCode::Tab if n > 0 => self.result_sel = (self.result_sel + 1).min(n - 1),
            KeyCode::Char('j') if ctrl && n > 0 => self.result_sel = (self.result_sel + 1).min(n - 1),
            KeyCode::Up | KeyCode::BackTab => self.result_sel = self.result_sel.saturating_sub(1),
            KeyCode::Char('k') if ctrl => self.result_sel = self.result_sel.saturating_sub(1),
            KeyCode::Enter => {
                if let Some(q) = self.query.strip_prefix('?') {
                    let q = q.trim().to_string();
                    self.mode = Mode::Read;
                    self.pinned = Some(ALL_KEY.into());
                    self.open_panel();
                    self.ask(q, ALL_KEY.into());
                    return;
                }
                if let Some(g) = self.results.get(self.result_sel) {
                    let line = g.snippets.first().map(|s| s.line).unwrap_or(0);
                    self.current = Some(self.notes[g.note].path.clone());
                    self.pinned = None;
                    self.mode = Mode::Read;
                    let rows = crate::ui::source_rows(self, self.text_width);
                    self.read_cursor = rows.get(line).copied().unwrap_or(0);
                    self.read_anchor = None;
                    self.scroll = self.read_cursor.saturating_sub(3);
                }
            }
            KeyCode::Backspace => {
                self.query.pop();
                self.run_search();
            }
            KeyCode::Char('u') if ctrl => {
                self.query.clear();
                self.run_search();
            }
            KeyCode::Char(c) if !ctrl => {
                self.query.push(c);
                self.run_search();
            }
            _ => {}
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
