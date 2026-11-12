//! Read mode: the cursor line and its keys.
use super::*;

impl App {
    pub(crate) fn row_len(&self, r: usize) -> usize {
        self.read_text.get(r).map_or(0, |l| l.trim_end().chars().count())
    }

    /// Move the read cursor by rows; the view follows it, the column stays where it can.
    pub fn move_cursor(&mut self, by: isize) {
        let last = self.read_text.len().saturating_sub(1) as isize;
        self.read_cursor = (self.read_cursor as isize + by).clamp(0, last.max(0)) as usize;
        self.read_col = self.read_want.min(self.row_len(self.read_cursor).saturating_sub(1));
        self.interact(1);
    }

    /// Move the read cursor by chars, across rows.
    pub(crate) fn move_col(&mut self, by: isize) {
        let len = self.row_len(self.read_cursor);
        if by < 0 {
            if self.read_col > 0 {
                self.read_col = self.read_col.min(len).saturating_sub(1);
            } else if self.read_cursor > 0 {
                self.read_cursor -= 1;
                self.read_col = self.row_len(self.read_cursor).saturating_sub(1);
            }
        } else if self.read_col + 1 < len {
            self.read_col += 1;
        } else if self.read_cursor + 1 < self.read_text.len() {
            self.read_cursor += 1;
            self.read_col = 0;
        }
        self.read_want = self.read_col;
        self.interact(1);
    }

    pub(crate) fn key_read(&mut self, act: Option<&str>) {
        let page = self.view_height.saturating_sub(3).max(1) as isize;
        let Some(act) = act else { return };
        if act.starts_with("select_") && self.read_anchor.is_none() {
            self.read_anchor = Some((self.read_cursor, self.read_col));
        }
        match act {
            "back" if self.say.is_some() => self.say = None,
            "back" if self.read_anchor.is_some() => self.read_anchor = None,
            "back" if self.panel => self.panel = false,
            "back" if self.delta_open => self.delta_open = false,
            "quit" => self.quit = true,
            "write" => self.start_write(false),
            "new" => self.start_write(true),
            "notes" => self.open_notes(),
            "search" => self.open_search(""),
            "chat" => self.toggle_chat(),
            "theme" => {
                self.theme_before = self.theme;
                self.mode = Mode::Theme;
            }
            "help" => self.open_help(crate::keys::Pane::Read),
            "delta" => self.toggle_delta(),
            "tree" => self.toggle_tree(),
            "upgrade" => self.upgrade(),
            "recap" => self.recap_now(),
            "recap_toggle" => {
                self.state.show_recap = !self.state.show_recap;
                self.state.save(&self.cfg.root);
            }
            "layout" => self.toggle_layout(),
            "down" | "select_down" => self.move_cursor(1),
            "up" | "select_up" => self.move_cursor(-1),
            "left" | "select_left" => self.move_col(-1),
            "right" | "select_right" => self.move_col(1),
            "page_down" => self.move_cursor(page),
            "page_up" => self.move_cursor(-page),
            "top" => self.move_cursor(-(isize::MAX / 4)),
            "bottom" => self.move_cursor(isize::MAX / 4),
            "select" => {
                self.read_anchor = if self.read_anchor.is_some() { None } else { Some((self.read_cursor, self.read_col)) };
            }
            "ask" => self.ask_about_selection(),
            "add_to_chat" => self.add_to_chat(),
            "say" => self.start_say(),
            "copy" => self.copy_selection(),
            "split" => self.split_document(),
            "next_note" => self.step_note(1),
            "prev_note" => self.step_note(-1),
            _ => {}
        }
    }

    pub(crate) fn step_note(&mut self, d: isize) {
        if self.notes.is_empty() {
            return;
        }
        let i = self.note_index().unwrap_or(0) as isize + d;
        let i = i.clamp(0, self.notes.len() as isize - 1) as usize;
        self.current = Some(self.notes[i].path.clone());
        self.scroll = 0;
        self.read_cursor = 0;
        self.read_col = 0;
        self.read_anchor = None;
        self.pinned = None;
    }
}
