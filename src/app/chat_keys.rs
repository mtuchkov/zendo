//! Keys of the chat input.
use super::*;

impl App {
    pub(crate) fn key_panel(&mut self, k: KeyEvent, act: Option<&str>) {
        if act != Some("back") && self.question_key(&k, act) {
            return;
        }
        let key = self.key();
        let busy = self.threads.get(&key).is_some_and(|t| t.busy.is_some());
        match act {
            Some("back") if busy => self.stop_answer(key),
            Some("back") if self.writer.say().is_some() => {
                self.writer.end_say();
                self.composer_mut().input_mut().take();
                self.composer_mut().drop_context();
                self.focus = Focus::Note;
            }
            Some("back") => {
                if self.composer_mut().input_mut().is_empty() {
                    self.threads.hide();
                    self.threads.unpin();
                    self.composer_mut().clear_chips();
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
            Some("send") if self.composer().ends_with_backslash() => {
                self.composer_mut().input_mut().backspace();
                self.composer_mut().input_mut().enter(true);
            }
            // In a code block, and after the fence that opens it, Enter is a new line (TEXT-10).
            Some("send") if self.composer().in_code() => self.composer_mut().input_mut().enter(true),
            Some("send" | "send_now") if self.cli.running(self.state.session.min(SESSIONS - 1)) => {
                self.remember_input();
                self.send_to_cli();
            }
            Some("send" | "send_now") => {
                self.remember_input();
                self.send_input();
            }
            None => self.input_key(k),
            Some(a) => self.input_action(a),
        }
        self.update_at();
        self.composer_mut().update_slash();
    }

    /// `esc` while the agent answers: stop it, and put the queued questions back into the input
    /// (CLAUDE-34).
    fn stop_answer(&mut self, key: String) {
        self.cancel_question(&key);
        self.link.send(Cmd::Cancel { key: key.clone() });
        self.queue_to_input(&key);
    }

    /// The queued questions of a thread go back into the chat input, before what it holds.
    pub(crate) fn queue_to_input(&mut self, key: &str) {
        let Some(t) = self.threads.get_mut(key).filter(|t| !t.queue.is_empty()) else { return };
        let mut parts: Vec<String> = t.queue.drain(..).map(|o| o.question).collect();
        // The input of the session of the thread, also while another one is active (CLAUDE-47).
        let n = (0..SESSIONS).find(|n| session_key(*n) == key).unwrap_or(self.state.session);
        let input = self.chat.composer_mut(n).input_mut();
        let now = input.take();
        if !now.is_empty() {
            parts.push(now);
        }
        input.type_str(&parts.join("\n"));
    }

    fn new_thread(&mut self, key: String) {
        self.link.send(Cmd::Forget { key: key.clone() });
        // A new thread is a new Claude session: the name of the old one goes (NAMES-4).
        if let Some(id) = self.state.take_session(&self.cfg.agent, &key) {
            self.state.names.remove(&id);
        }
        // The messages of the old thread go into the common history (HISTORY-2).
        self.state.history.merge(self.state.session.min(SESSIONS - 1), chrono::Local::now());
        self.state.save(&self.cfg.root);
        self.threads.remove(&key);
        self.flash("new thread");
    }

    /// The edit actions of the chat input; the shared ones are in `text_edit.rs` (TEXT-4, TEXT-5).
    fn input_action(&mut self, a: &str) {
        let w = self.frame.input_width().max(10);
        match a {
            "newline" => self.composer_mut().input_mut().enter(true),
            // The clear can be undone; the chips stay for their labels (UNDO-1).
            "clear" => self.composer_mut().input_mut().clear_text(),
            "paste" => {
                // An image on the clipboard is an attachment; text may be dropped files.
                if let Some(p) = crate::text::select::paste_image() {
                    self.attach_pasted_image(p);
                } else if let Some(t) = crate::text::select::paste().or_else(|| self.pointer.last_copy()) {
                    self.paste_into_chat(&t);
                }
            }
            "templates" => self.pick_template(),
            "select_up" => self.composer_mut().input_mut().select_with(|e| e.vertical(-1, w)),
            "select_down" => self.composer_mut().input_mut().select_with(|e| e.vertical(1, w)),
            _ => drop(text_edit::rich_action(self.chat.composer_mut(self.state.session).input_mut(), a, &mut self.pointer)),
        }
    }

    /// A key with no action: typing and the arrows in the chat input.
    fn input_key(&mut self, k: KeyEvent) {
        if self.composer_mut().input_mut().selection().is_none() && self.composer_mut().chip_key(k.code) {
            return;
        }
        let w = self.frame.input_width().max(10);
        if k.modifiers.is_empty() && matches!(k.code, KeyCode::Up | KeyCode::Down) && self.recall(k.code == KeyCode::Up, w) {
            return;
        }
        let ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
        let e = self.composer_mut().input_mut();
        match k.code {
            KeyCode::Char(c) if !ctrl => {
                e.type_over(c);
                if c == '/' {
                    Self::slashes_to_date(e);
                }
            }
            KeyCode::Backspace => e.delete_selection_or(Editor::backspace),
            KeyCode::Delete => e.delete(),
            KeyCode::Left => e.left(),
            KeyCode::Right => e.right(),
            KeyCode::Up => e.vertical(-1, w),
            KeyCode::Down => e.vertical(1, w),
            _ => {}
        }
    }

    /// The message that goes now goes into the history of the active session, with its text
    /// chips (HISTORY-1, HISTORY-7). `/clear` starts a new conversation, as `ctrl-n` does: the
    /// history of the session goes into the common history (HISTORY-2).
    fn remember_input(&mut self) {
        let (text, chips) = self.composer_mut().history_entry();
        let session = self.state.session.min(SESSIONS - 1);
        self.state.history.push(session, &text, chips, chrono::Local::now());
        if text.split_whitespace().next() == Some("/clear") {
            self.state.history.merge(session, chrono::Local::now());
        }
        self.state.save(&self.cfg.root);
    }

    /// `up` on the first row or `down` on the last row: the history of the session (HISTORY-5).
    /// False when the cursor can move, or the history has nothing there.
    fn recall(&mut self, back: bool, w: usize) -> bool {
        if !self.composer().input().on_edge(if back { -1 } else { 1 }, w) {
            return false;
        }
        let list = self.state.history.list(self.state.session.min(SESSIONS - 1));
        if back { self.composer_mut().recall_back(&list) } else { self.composer_mut().recall_forward(&list) }
    }
}
