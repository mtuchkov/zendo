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
            self.writer.touch();
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
            "review" => self.review_key(),
            "chat" => {
                self.save();
                self.toggle_chat();
            }
            "add_to_chat" => self.add_line_to_chat(),
            _ => return self.write_edit(a),
        }
        None
    }

    /// The actions that change the text, then the moves.
    fn write_edit(&mut self, a: &str) -> Option<bool> {
        match a {
            // The clipboard is read on a worker; the text comes in `poll_clipboard` (IO-1).
            "paste" => {
                self.paste_from_clipboard(PasteTo::Write);
                return None;
            }
            "indent" => self.writer.editor_mut().type_str("  "),
            "undo" => return self.write_undo(),
            "redo" => return self.write_redo(),
            "select_up" | "select_down" => {
                let e = self.writer.editor();
                let anchor = e.anchor.unwrap_or((e.row, e.col));
                self.write_vertical(if a == "select_up" { -1 } else { 1 });
                self.writer.editor_mut().anchor = Some(anchor);
                return Some(false);
            }
            _ => return text_edit::rich_action(self.writer.editor_mut(), a, &mut self.pointer),
        }
        Some(true)
    }

    /// A key with no action: typing, deleting and the arrows. True when the text changed.
    fn write_key(&mut self, k: KeyEvent) -> bool {
        let w = self.frame.text_width();
        let page = self.frame.view_height() as isize - 2;
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        let rows = [(KeyCode::Up, -1), (KeyCode::Down, 1), (KeyCode::PageUp, -page), (KeyCode::PageDown, page)];
        if let Some((_, delta)) = rows.iter().find(|(c, _)| *c == k.code) {
            self.writer.editor_mut().clear_selection();
            self.write_vertical(*delta);
            return false;
        }
        let prose = !self.writer.file().is_some_and(crate::text::code::is_code);
        let e = self.writer.editor_mut();
        match k.code {
            KeyCode::Char(c) if !ctrl => {
                e.type_over(c);
                if prose && c == '/' {
                    Self::slashes_to_date(e);
                }
                // A line of prose longer than the text column breaks at a word; code only wraps.
                if prose {
                    e.hard_wrap(w);
                }
            }
            // Prose goes on with the list and closes a fence; code keeps the indent of the line.
            KeyCode::Enter => e.enter(prose),
            KeyCode::Backspace => e.delete_selection_or(Editor::backspace),
            KeyCode::Delete => e.delete_selection_or(Editor::delete),
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
