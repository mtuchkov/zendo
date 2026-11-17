//! The chat: the agent, threads, questions, answers to proposals, attachments.
use super::*;

impl App {
    pub fn ensure_agent(&mut self) {
        if self.link.running() || self.cfg.offline {
            return;
        }
        let log = state::dir(&self.cfg.root);
        let _ = std::fs::create_dir_all(&log);
        let (h, rx) = agent::spawn(self.cfg.agent.clone(), self.cfg.agent_env.clone(), self.cfg.root.clone(), Some(log.join("agent.log")));
        self.link.connect(h, rx);
    }

    pub fn open_panel(&mut self) {
        if self.cfg.offline {
            self.flash("offline: start zendo without --offline to use Claude");
            return;
        }
        self.ensure_agent();
        if self.mode != Mode::Write {
            self.mode = Mode::Read;
        }
        self.threads.show();
        self.focus = Focus::Claude;
        self.threads.transcript_mut().focus(false);
        self.threads.to_end();
        self.open_session();
    }

    /// F6 to F10, alt-] and alt-[: the chat goes to agent session `n`, after the last one the first. The
    /// chat opens on it.
    pub fn switch_session(&mut self, n: usize) {
        // The place in the document of the session that the user leaves (FILES-54).
        self.track_recent();
        self.cli.pane = None;
        self.state.session = n % SESSIONS;
        self.state.save(&self.cfg.root);
        self.follow_session();
        if self.state.tree {
            self.scan_tree();
        }
        // FILES-58: the tree shows the open document of the session.
        self.follow_open_file();
        self.composer.close_picks();
        self.open_panel();
        let f = self.keys.hint(crate::keys::Pane::Global, &format!("session_{}", self.state.session + 1), "");
        self.flash(format!("agent session {}{}", self.state.session + 1, if f.is_empty() { f } else { format!(" · {f}") }));
    }

    /// Open the session of the chat before the first question: the last session shows its
    /// messages, and the agent sends its slash commands.
    pub(crate) fn open_session(&mut self) {
        let key = self.key();
        let known = self.state.session(&self.cfg.agent, &key);
        let t = self.thread(&key);
        if t.opened || t.session.is_some() {
            return;
        }
        t.opened = true;
        self.link.send(Cmd::Open { key, known });
    }

    pub fn thread(&mut self, key: &str) -> &mut Thread {
        self.threads.entry(key)
    }

    /// Ask in the one chat thread. `scope` is what the question is about: a file key, the new
    /// note, or all notes.
    pub fn ask(&mut self, question: String, scope: String) {
        let question = question.trim().to_string();
        if question.is_empty() || self.cli_blocks() {
            return;
        }
        if self.cfg.offline {
            self.flash("offline: start zendo without --offline to use Claude");
            return;
        }
        if self.mode == Mode::Write && self.writer.editor_mut().dirty {
            self.save();
        }
        self.ensure_agent();
        let key = self.key();
        // A slash command goes to the agent as the user typed it.
        let command = question.starts_with('/');
        let (text, file) = if command { (question.clone(), None) } else { self.prompt_text(&question, &scope, &key) };
        let mut attach = self.composer.take_attach();
        if let Some(p) = &file {
            attach.push(Attachment::File(p.clone()));
        }
        let sent = file.as_ref().map(|p| self.rel(p));
        let out = Outgoing { question, text, attach, sent, command };
        self.threads.to_end();
        match self.threads.get_mut(&key).filter(|t| t.busy.is_some()) {
            Some(t) => t.queue.push(out),
            None => self.dispatch(key, out),
        }
    }

    /// Send a question to the agent now.
    pub(crate) fn dispatch(&mut self, key: String, out: Outgoing) {
        let known = self.state.session(&self.cfg.agent, &key);
        let t = self.thread(&key);
        t.push(Item::User(out.question));
        t.busy = Some(Instant::now());
        t.dropped = false;
        t.primed |= !out.command;
        t.files.push(out.sent);
        self.keep_session(&key);
        self.link.send(Cmd::Prompt { key, known, text: out.text, attach: out.attach });
    }

    /// The text of a question with its context, and the file to attach: the file on screen when
    /// it did not go with one of the last ATTACH_EVERY questions.
    fn prompt_text(&mut self, question: &str, scope: &str, key: &str) -> (String, Option<PathBuf>) {
        let mut text = String::new();
        if !self.threads.get(key).is_some_and(|t| t.primed) {
            text.push_str(&state::prompt(&self.cfg.root));
            text.push_str("\n\n");
        }
        let mut file = None;
        if scope == ALL_KEY {
            text.push_str(&format!("Notes folder: {}\nSearch across all notes in that folder.\n", self.cfg.notes_dir.display()));
        } else if self.files.current().is_some_and(|p| !p.exists()) {
            // No version of the document in this session: no note file (FILES-40).
        } else if let Some(p) = self.files.current().map(|p| self.session_path(p)).filter(|_| scope != NEW_KEY) {
            let rel = self.rel(&p);
            if !self.threads.get(key).is_some_and(|t| t.sent_lately(&rel)) {
                text.push_str(&self.rules_for(&p));
                file = Some(p.clone());
            }
            text.push_str(&format!("Note file: {}\n", p.display()));
            if self.mode == Mode::Write
                && let Some(line) = self.writer.editor().lines.get(self.writer.editor().row).filter(|l| !l.trim().is_empty()) {
                    text.push_str(&format!("Current line: {line}\n"));
                }
        } else {
            text.push_str(&format!("The note is not saved yet. Its text:\n{}\n", self.writer.editor_mut().text()));
        }
        if let Some(c) = self.composer.take_context() {
            text.push_str(&c);
            text.push('\n');
        }
        text.push_str(&format!("Question: {question}"));
        (text, file)
    }

    /// The rules of project-structure.toml for a file: project rules first, then each folder.
    fn rules_for(&self, p: &Path) -> String {
        let Some(st) = self.files.structure() else { return String::new() };
        let rules = st.rules_for(self.files.store().rel(p));
        if rules.is_empty() { rules } else { rules + "\n" }
    }

    /// The agent may have written a note. Pick up the change. The file of the last save is no
    /// change: the editor keeps the empty lines at the end and the cursor (NOTE-3).
    pub fn reload_after_agent(&mut self) {
        self.reload();
        if self.mode == Mode::Write && !self.writer.editor_mut().dirty
            && let Some(p) = self.writer.path()
                && let Ok(body) = std::fs::read_to_string(p)
                && body.trim_end() != self.writer.editor().text().trim_end() {
                    let (row, col) = (self.writer.editor_mut().row, self.writer.editor_mut().col);
                    *self.writer.editor_mut() = Editor::new(body.trim_end_matches('\n'));
                    self.writer.editor_mut().row = row.min(self.writer.editor_mut().lines.len() - 1);
                    self.writer.editor_mut().col = col;
                    self.writer.editor_mut().end();
                }
    }

    /// Put a file or an image into the chat input as a chip; it goes with the next prompt.
    pub fn attach(&mut self, a: Attachment) {
        let label = chips::file_label(a.path());
        self.attach_as(label, a);
    }

    /// Put an image from the clipboard into the chat input as the chip `Image #n`, with the lowest
    /// number that no chip in the input has (CLAUDE-17).
    pub fn attach_pasted_image(&mut self, path: PathBuf) {
        let free = |n: &usize| !self.composer.chips().iter().any(|c| c.label == format!("{}Image #{n}{}", chips::OPEN, chips::CLOSE));
        let n = (1..).find(free).expect("a free number");
        self.attach_as(format!("Image #{n}"), Attachment::Image(path.clone()));
        self.name_image(path);
    }

    fn attach_as(&mut self, label: String, a: Attachment) {
        let label = self.composer.add_chip(label, None, Some(a), false);
        self.flash(format!("attached {}", label.trim_matches([chips::OPEN, chips::CLOSE])));
    }

    /// Text pasted into the chat: dropped files become chips, the rest is text.
    pub(crate) fn paste_into_chat(&mut self, s: &str) {
        match crate::text::select::dropped_files(s) {
            Some(paths) => {
                for p in paths {
                    self.attach(Attachment::of(p));
                }
            }
            None => self.composer.input_mut().type_str(s),
        }
    }

    pub fn on_paste(&mut self, s: String) {
        if self.hold.is_some() {
            return;
        }
        self.tab_run.end();
        if let Some(n) = self.cli.view.or_else(|| self.pane_terminal()) {
            self.paste_cli(n, &s);
        } else if self.threads.shown() && self.focus == Focus::Claude {
            self.paste_into_chat(&s);
        } else if self.mode == Mode::Write {
            self.writer.editor_mut().type_str(&s);
            self.writer.touch();
        } else if self.mode == Mode::Search {
            self.lists.edit_query(|q| q.push_str(s.lines().next().unwrap_or("")));
            self.run_search();
        }
    }

    /// `c` and `[ chat ]`: open the chat, or close it when it is open.
    pub fn toggle_chat(&mut self) {
        if self.threads.shown() {
            self.threads.hide();
            self.threads.unpin();
            if self.focus == Focus::Claude {
                self.focus = Focus::Note;
            }
        } else {
            self.open_panel();
        }
    }

    /// Send the chat input: the chips in it bring their text as context.
    pub(crate) fn send_input(&mut self) {
        let q = self.composer.take_question();
        if !self.link.images() && self.composer.sends_image() {
            self.flash(format!("{} takes no images here: they go as file links", self.agent_label()));
        }
        let scope = self.scope();
        self.ask(q, scope);
    }
}
