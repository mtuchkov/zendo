//! Write mode keys.
use super::*;

impl App {
    pub(crate) fn key_write(&mut self, k: KeyEvent, act: Option<&str>) {
        let edited = match act {
            Some(a) => match self.write_action(a) {
                Some(edited) => edited,
                None => return,
            },
            None => self.write_key(k),
        };
        if edited {
            self.last_edit = Instant::now();
        }
    }

    /// An action of the keymap. `Some(edited)`, or `None` when the action needs no more work.
    fn write_action(&mut self, a: &str) -> Option<bool> {
        match a {
            "done" => self.finish_write(),
            "delta" => {
                self.toggle_delta();
                self.focus = Focus::Note;
                self.make_room_for_delta();
            }
            "toggle_tree" => self.toggle_tree(),
            "say" => self.start_say(),
            "chat" => {
                self.save();
                self.toggle_chat();
            }
            "add_to_chat" => self.add_line_to_chat(),
            "copy" => {
                if let Some(t) = self.editor.selected_text() {
                    self.clip = Some(t);
                }
            }
            _ => return self.write_edit(a),
        }
        None
    }

    /// The actions that change the text, then the moves.
    fn write_edit(&mut self, a: &str) -> Option<bool> {
        match a {
            "bold" => self.editor.wrap_marks("**", "**"),
            "italic" => self.editor.wrap_marks("*", "*"),
            "underline" => self.editor.wrap_marks("<u>", "</u>"),
            "cut" => {
                let t = self.editor.selected_text()?;
                self.clip = Some(t);
                self.editor.delete_selection();
            }
            "paste" => match crate::text::select::paste().or_else(|| self.last_copy.clone()) {
                Some(t) => self.editor.type_str(&t),
                None => {
                    self.flash("the clipboard is empty, or zendo cannot read it here");
                    return None;
                }
            },
            "delete_word" => {
                if !self.editor.delete_selection() {
                    self.editor.delete_word_back();
                }
            }
            "indent" => self.editor.type_str("  "),
            _ => {
                self.write_move(a);
                return Some(false);
            }
        }
        Some(true)
    }

    fn write_move(&mut self, a: &str) {
        if let Some(delta) = [("select_up", -1), ("select_down", 1)].iter().find(|(n, _)| *n == a).map(|(_, d)| *d) {
            let anchor = self.editor.anchor.unwrap_or((self.editor.row, self.editor.col));
            self.write_vertical(delta);
            self.editor.anchor = Some(anchor);
            return;
        }
        let e = &mut self.editor;
        if a.starts_with("select_") {
            match a {
                "select_left" => e.select_with(|e| e.left()),
                "select_right" => e.select_with(|e| e.right()),
                "select_word_left" => e.select_with(|e| e.word_left()),
                "select_word_right" => e.select_with(|e| e.word_right()),
                "select_home" => e.select_with(|e| e.home()),
                "select_end" => e.select_with(|e| e.end()),
                _ => {}
            }
            return;
        }
        e.clear_selection();
        match a {
            "word_left" => e.word_left(),
            "word_right" => e.word_right(),
            "home" => e.home(),
            "end" => e.end(),
            _ => {}
        }
    }

    /// A key with no action: typing, deleting and the arrows. True when the text changed.
    fn write_key(&mut self, k: KeyEvent) -> bool {
        let w = self.text_width;
        let page = self.view_height as isize - 2;
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        let rows = [(KeyCode::Up, -1), (KeyCode::Down, 1), (KeyCode::PageUp, -page), (KeyCode::PageDown, page)];
        if let Some((_, delta)) = rows.iter().find(|(c, _)| *c == k.code) {
            self.editor.clear_selection();
            self.write_vertical(*delta);
            return false;
        }
        let e = &mut self.editor;
        match k.code {
            KeyCode::Char(c) if !ctrl => {
                e.type_char(c);
                // A line longer than the text column breaks at a word.
                e.hard_wrap(w);
            }
            KeyCode::Enter => {
                e.delete_selection();
                e.split_line(true)
            }
            KeyCode::Backspace => {
                if !e.delete_selection() {
                    e.backspace()
                }
            }
            KeyCode::Delete => {
                if !e.delete_selection() {
                    e.delete()
                }
            }
            _ => {
                e.clear_selection();
                match k.code {
                    KeyCode::Left => e.left(),
                    KeyCode::Right => e.right(),
                    _ => {}
                }
                return false;
            }
        }
        true
    }
}
