//! Read mode: the cursor line and its keys.
use super::*;

impl App {
    /// Move the read cursor by rows; the view follows it, the column stays where it can.
    pub fn move_cursor(&mut self, by: isize) {
        self.reader.move_rows(self.frame.read_text(), by);
        self.interact(1);
    }

    /// Move the read cursor by chars, across rows.
    pub(crate) fn move_col(&mut self, by: isize) {
        self.reader.move_col(self.frame.read_text(), by > 0);
        self.interact(1);
    }

    pub(crate) fn key_read(&mut self, act: Option<&str>) {
        let page = self.frame.view_height().saturating_sub(3).max(1) as isize;
        let Some(act) = act else { return };
        if act.starts_with("select_") {
            self.reader.select();
        }
        let before = self.reader.cursor();
        self.read_action(act, page);
        self.reader.soft_stop(self.frame.read_text(), act, before);
    }

    fn read_action(&mut self, act: &str, page: isize) {
        match act {
            "back" if self.writer.say().is_some() => self.writer.end_say(),
            "back" if self.reader.anchor().is_some() => self.reader.unselect(),
            "back" if self.threads.shown() => self.threads.hide(),
            "back" if self.work.delta_open() => self.work.close_delta(),
            "quit" => self.quit = true,
            "write" => self.start_write(false),
            "new" => self.open_float(),
            "notes" => self.open_notes(),
            "search" => self.open_find(),
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
            "peek_main" => self.peek_at(version::Other::Main),
            "peek_disk" => self.peek_at(version::Other::Disk),
            "review" => self.review_key(),
            "recap_toggle" => {
                self.state.show_recap = !self.state.show_recap;
                self.state.save(&self.cfg.root);
            }
            "layout" => self.toggle_layout(),
            "down" | "select_down" => self.move_cursor(1),
            "up" | "select_up" => self.move_cursor(-1),
            "left" | "select_left" => self.move_col(-1),
            "right" | "select_right" => self.move_col(1),
            "select_word_left" => self.move_word(false),
            "select_word_right" => self.move_word(true),
            "page_down" => self.move_cursor(page),
            "page_up" => self.move_cursor(-page),
            "top" => self.move_cursor(-(isize::MAX / 4)),
            "bottom" => self.move_cursor(isize::MAX / 4),
            "select" => self.reader.toggle_select(),
            "ask" => self.ask_about_selection(),
            "add_to_chat" => self.add_to_chat(),
            "say" => self.start_say(),
            "copy" => self.copy_selection(),
            "split" => self.split_document(),
            "to_spec" => self.note_to_spec(),
            "next_note" => self.step_note(1),
            "prev_note" => self.step_note(-1),
            _ => {}
        }
    }

    pub(crate) fn step_note(&mut self, d: isize) {
        if self.files.notes().is_empty() {
            return;
        }
        let i = self.note_index().unwrap_or(0) as isize + d;
        let i = i.clamp(0, self.files.notes().len() as isize - 1) as usize;
        self.files.show_note(i);
        self.reader.to_start();
        self.threads.unpin();
    }

    /// The cursor to the start of the next word, or of this or the previous word.
    pub(crate) fn move_word(&mut self, right: bool) {
        let steps = self.reader.move_word(self.frame.read_text(), right);
        self.interact(steps);
    }
}
