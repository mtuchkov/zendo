//! Keys of the chat input.
use super::*;

impl App {
    pub(crate) fn key_panel(&mut self, k: KeyEvent, act: Option<&str>) {
        let key = self.key();
        let busy = self.threads.get(&key).is_some_and(|t| t.busy.is_some());
        match act {
            Some("back") if busy => self.stop_answer(key),
            Some("back") if self.writer.say().is_some() => {
                self.writer.end_say();
                self.composer.input_mut().take();
                self.composer.drop_context();
                self.focus = Focus::Note;
            }
            Some("back") => {
                if self.composer.input_mut().is_empty() {
                    self.panel = false;
                    self.threads.unpin();
                    self.composer.clear_chips();
                }
                self.focus = Focus::Note;
            }
            Some("new_thread") => self.new_thread(key),
            Some("handoff") => self.open_cli(),
            Some("insert_output") => self.insert_output(),
            Some("scroll_up") => self.threads.scroll_by(true, self.frame.view_height() / 2),
            Some("scroll_down") => self.threads.scroll_by(false, self.frame.view_height() / 2),
            // `\` and Enter is a new line in every terminal, as in Claude Code: a terminal without the
            // kitty keyboard protocol sends Shift-Enter as a plain Enter.
            Some("send") if self.composer.ends_with_backslash() => {
                self.composer.input_mut().backspace();
                self.composer.input_mut().type_str("\n");
            }
            Some("send") if self.cli.running(self.state.session.min(SESSIONS - 1)) => self.send_to_cli(),
            Some("send") => self.send_input(),
            None => self.input_key(k),
            Some(a) => self.input_action(a),
        }
        self.update_at();
        self.composer.update_slash();
    }

    /// `esc` while the agent answers: stop it, and put the queued questions back into the input
    /// (CLAUDE-34).
    fn stop_answer(&mut self, key: String) {
        self.link.send(Cmd::Cancel { key: key.clone() });
        self.queue_to_input(&key);
    }

    /// The queued questions of a thread go back into the chat input, before what it holds.
    pub(crate) fn queue_to_input(&mut self, key: &str) {
        let Some(t) = self.threads.get_mut(key).filter(|t| !t.queue.is_empty()) else { return };
        let mut parts: Vec<String> = t.queue.drain(..).map(|o| o.question).collect();
        let now = self.composer.input_mut().take();
        if !now.is_empty() {
            parts.push(now);
        }
        self.composer.input_mut().type_str(&parts.join("\n"));
    }

    fn new_thread(&mut self, key: String) {
        self.link.send(Cmd::Forget { key: key.clone() });
        // A new thread is a new Claude session: the name of the old one goes (NAMES-4).
        if let Some(id) = self.state.take_session(&self.cfg.agent, &key) {
            self.state.names.remove(&id);
        }
        self.state.save(&self.cfg.root);
        self.threads.remove(&key);
        self.flash("new thread");
    }

    /// The edit actions of the chat input.
    fn input_action(&mut self, a: &str) {
        match a {
            "newline" => self.composer.input_mut().type_str("\n"),
            "delete_word" => self.composer.input_mut().delete_selection_or(Editor::delete_word_back),
            "clear" => {
                self.composer.clear();
            }
            "paste" => {
                // An image on the clipboard is an attachment; text may be dropped files.
                if let Some(p) = crate::text::select::paste_image() {
                    self.attach_pasted_image(p);
                } else if let Some(t) = crate::text::select::paste().or_else(|| self.pointer.last_copy()) {
                    self.paste_into_chat(&t);
                }
            }
            "templates" => self.pick_template(),
            "select_left" => self.composer.input_mut().select_with(|e| e.left()),
            "select_right" => self.composer.input_mut().select_with(|e| e.right()),
            "select_word_left" => self.composer.input_mut().select_with(|e| e.word_left()),
            "select_word_right" => self.composer.input_mut().select_with(|e| e.word_right()),
            "word_left" => self.composer.input_mut().word_left(),
            "word_right" => self.composer.input_mut().word_right(),
            "home" => self.composer.input_mut().home(),
            "end" => self.composer.input_mut().end(),
            _ => {}
        }
    }

    /// A key with no action: typing and the arrows in the chat input.
    fn input_key(&mut self, k: KeyEvent) {
        if self.composer.input_mut().selection().is_none() && self.composer.chip_key(k.code) {
            return;
        }
        let w = self.frame.input_width().max(10);
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        let e = self.composer.input_mut();
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
