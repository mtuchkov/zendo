//! Keys of the chat input.
use super::*;

impl App {
    pub(crate) fn key_panel(&mut self, k: KeyEvent, act: Option<&str>) {
        let key = self.key();
        let pending = self.threads.get(&key).and_then(|t| t.pending()).is_some();
        let busy = self.threads.get(&key).is_some_and(|t| t.busy.is_some());
        match act {
            Some("back") if busy && !pending => {
                if let Some(a) = &self.agent {
                    a.send(Cmd::Cancel { key });
                }
            }
            Some("back") if self.say.is_some() => {
                self.say = None;
                self.input.take();
                self.ask_context = None;
                self.focus = Focus::Note;
            }
            Some("back") => {
                if self.input.is_empty() {
                    self.panel = false;
                    self.pinned = None;
                    self.chips.clear();
                }
                self.focus = Focus::Note;
            }
            Some("new_thread") => self.new_thread(key),
            Some("handoff") => self.open_cli(),
            Some("scroll_up") => self.panel_scroll += self.view_height / 2,
            Some("scroll_down") => self.panel_scroll = self.panel_scroll.saturating_sub(self.view_height / 2),
            Some(a @ ("allow" | "always" | "edit_first" | "discard")) if pending => self.answer(match a {
                "allow" => 'a',
                "always" => 'A',
                "edit_first" => 'e',
                _ => 'x',
            }),
            _ if pending => {}
            // `\` and Enter is a new line in every terminal, as in Claude Code: a terminal without the
            // kitty keyboard protocol sends Shift-Enter as a plain Enter.
            Some("send") if self.input.col > 0 && self.input.lines[self.input.row].chars().nth(self.input.col - 1) == Some('\\') => {
                self.input.backspace();
                self.input.type_str("\n");
            }
            Some("send") if self.cli_blocks() => {}
            Some("send") if !busy => self.send_input(),
            Some("send") => {}
            // A proposal key is text when there is no proposal.
            Some("allow" | "always" | "edit_first" | "discard") | None => self.input_key(k),
            Some(a) => self.input_action(a),
        }
        self.update_at();
        self.update_slash();
    }

    fn new_thread(&mut self, key: String) {
        if let Some(a) = &self.agent {
            a.send(Cmd::Forget { key: key.clone() });
        }
        self.state.take_session(&self.cfg.agent, &key);
        self.state.save(&self.cfg.root);
        self.threads.remove(&key);
        self.flash("new thread");
    }

    /// The edit actions of the chat input.
    fn input_action(&mut self, a: &str) {
        match a {
            "newline" => self.input.type_str("\n"),
            "delete_word" => self.input.delete_selection_or(Editor::delete_word_back),
            "clear" => {
                self.input.take();
                self.chips.clear();
            }
            "paste" => {
                // An image on the clipboard is an attachment; text may be dropped files.
                if let Some(p) = crate::text::select::paste_image() {
                    self.attach(Attachment::Image(p));
                } else if let Some(t) = crate::text::select::paste().or_else(|| self.last_copy.clone()) {
                    self.paste_into_chat(&t);
                }
            }
            "templates" => self.pick_template(),
            "select_left" => self.input.select_with(|e| e.left()),
            "select_right" => self.input.select_with(|e| e.right()),
            "select_word_left" => self.input.select_with(|e| e.word_left()),
            "select_word_right" => self.input.select_with(|e| e.word_right()),
            "word_left" => self.input.word_left(),
            "word_right" => self.input.word_right(),
            "home" => self.input.home(),
            "end" => self.input.end(),
            _ => {}
        }
    }

    /// A key with no action: typing and the arrows in the chat input.
    fn input_key(&mut self, k: KeyEvent) {
        if self.input.selection().is_none() && self.chip_key(k.code) {
            return;
        }
        let w = self.input_width.max(10);
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        let e = &mut self.input;
        match k.code {
            KeyCode::Char(c) if !ctrl => e.type_char(c),
            KeyCode::Backspace => e.delete_selection_or(Editor::backspace),
            KeyCode::Delete => e.delete(),
            KeyCode::Left => e.left(),
            KeyCode::Right => e.right(),
            KeyCode::Up => e.vertical(-1, w),
            KeyCode::Down => e.vertical(1, w),
            _ => {}
        }
    }
}
