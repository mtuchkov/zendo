//! The chat: the agent, threads, questions, answers to proposals, attachments.
use super::*;

impl App {
    pub fn ensure_agent(&mut self) {
        if self.agent.is_some() || self.cfg.offline {
            return;
        }
        let log = state::dir(&self.cfg.root);
        let _ = std::fs::create_dir_all(&log);
        let (h, rx) = agent::spawn(self.cfg.agent.clone(), self.cfg.agent_env.clone(), self.cfg.root.clone(), Some(log.join("agent.log")));
        self.agent = Some(h);
        self.agent_events = Some(rx);
        self.agent_down = None;
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
        self.panel = true;
        self.focus = Focus::Claude;
        self.panel_scroll = 0;
    }

    pub fn thread(&mut self, key: &str) -> &mut Thread {
        self.threads.entry(key.to_string()).or_default()
    }

    pub fn ask(&mut self, question: String, key: String) {
        let question = question.trim().to_string();
        if question.is_empty() {
            return;
        }
        if self.cfg.offline {
            self.flash("offline: start zendo without --offline to use Claude");
            return;
        }
        if self.mode == Mode::Write && self.editor.dirty {
            self.save();
        }
        self.ensure_agent();
        let mut text = String::new();
        let primed = self.threads.get(&key).is_some_and(|t| t.primed);
        if !primed {
            text.push_str(&state::prompt(&self.cfg.root));
            text.push_str("\n\n");
            // The rules of project-structure.toml for this file: project rules first, then each folder.
            if let (Some(st), Some(p)) = (self.tree.structure.as_ref(), self.current.as_ref()) {
                let rel = self.store.rel(p).to_path_buf();
                let rules = st.rules_for(&rel);
                if !rules.is_empty() && key != ALL_KEY {
                    text.push_str(&rules);
                    text.push('\n');
                }
            }
        }
        if key == ALL_KEY {
            text.push_str(&format!("Notes folder: {}\nSearch across all notes in that folder.\n", self.cfg.notes_dir.display()));
        } else if let Some(p) = self.current.clone().filter(|_| key != "(new note)") {
            text.push_str(&format!("Note file: {}\n", p.display()));
            if self.mode == Mode::Write
                && let Some(line) = self.editor.lines.get(self.editor.row).filter(|l| !l.trim().is_empty()) {
                    text.push_str(&format!("Current line: {line}\n"));
                }
        } else {
            text.push_str(&format!("The note is not saved yet. Its text:\n{}\n", self.editor.text()));
        }
        self.ask_default = None;
        if let Some(c) = self.ask_context.take() {
            text.push_str(&c);
            text.push('\n');
        }
        text.push_str(&format!("Question: {question}"));

        let known = self.state.session(&self.cfg.agent, &key);
        let t = self.thread(&key);
        t.items.push(Item::User(question));
        t.busy = Some(Instant::now());
        t.primed = true;
        self.panel_scroll = 0;
        let attach = std::mem::take(&mut self.pending_attach);
        if let Some(a) = &self.agent {
            a.send(Cmd::Prompt { key, known, text, attach });
        }
    }

    pub fn answer(&mut self, how: char) {
        let key = self.key();
        let Some((i, request)) = self.threads.get(&key).and_then(|t| t.pending()) else { return };
        let Item::Proposal { choices, diffs, .. } = &self.threads[&key].items[i] else { return };
        let allow_once = choices.iter().find(|c| c.allow && !c.always).or_else(|| choices.iter().find(|c| c.allow));
        let allow_always = choices.iter().find(|c| c.allow && c.always);
        let reject = choices.iter().find(|c| !c.allow && !c.always).or_else(|| choices.iter().find(|c| !c.allow));
        let (option, state) = match how {
            'a' => (allow_once.map(|c| c.id.clone()), Answered::Applied),
            'A' => (allow_always.or(allow_once).map(|c| c.id.clone()), Answered::Applied),
            'e' => (reject.map(|c| c.id.clone()), Answered::EditFirst),
            _ => (reject.map(|c| c.id.clone()), Answered::Discarded),
        };
        let proposed = diffs.first().cloned();
        if let Some(a) = &self.agent {
            a.send(Cmd::Answer { request, option });
        }
        if let Some(Item::Proposal { state: s, .. }) = self.threads.get_mut(&key).and_then(|t| t.items.get_mut(i)) {
            *s = state.clone();
        }
        if state == Answered::EditFirst
            && let Some(d) = proposed {
                // Open the proposed text in the editor; nothing is written yet.
                self.current = Some(d.path.clone());
                self.editor = Editor::new(d.new.trim_end_matches('\n'));
                self.editor_path = Some(d.path.clone());
                self.editor.dirty = true;
                self.last_edit = Instant::now() + Duration::from_secs(3600);
                self.mode = Mode::Write;
                self.focus = Focus::Note;
                self.flash("edit the proposal, then esc to save it");
            }
    }

    /// The agent may have written a note. Pick up the change.
    pub fn reload_after_agent(&mut self) {
        self.reload();
        if self.mode == Mode::Write && !self.editor.dirty
            && let Some(p) = &self.editor_path
                && let Ok(body) = std::fs::read_to_string(p) {
                    let (row, col) = (self.editor.row, self.editor.col);
                    self.editor = Editor::new(body.trim_end_matches('\n'));
                    self.editor.row = row.min(self.editor.lines.len() - 1);
                    self.editor.col = col;
                    self.editor.end();
                }
    }

    /// Put a file or an image into the chat input as a chip; it goes with the next prompt.
    pub fn attach(&mut self, a: Attachment) {
        let label = self.add_chip(chips::file_label(a.path()), None, Some(a), false);
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
            None => self.input.type_str(s),
        }
    }

    pub fn on_paste(&mut self, s: String) {
        if self.panel && self.focus == Focus::Claude {
            self.paste_into_chat(&s);
        } else if self.mode == Mode::Write {
            self.editor.type_str(&s);
            self.last_edit = Instant::now();
        } else if self.mode == Mode::Search {
            self.query.push_str(s.lines().next().unwrap_or(""));
            self.run_search();
        }
    }

    /// `c` and `[ chat ]`: open the chat, or close it when it is open.
    pub fn toggle_chat(&mut self) {
        if self.panel {
            self.panel = false;
            self.pinned = None;
            if self.focus == Focus::Claude {
                self.focus = Focus::Note;
            }
        } else {
            self.open_panel();
        }
    }

    /// Send the chat input: the chips in it bring their text as context.
    pub(crate) fn send_input(&mut self, key: String) {
        let mut q = self.input.take();
        let chips: Vec<Chip> = std::mem::take(&mut self.chips).into_iter().filter(|c| q.contains(c.label.as_str())).collect();
        let ctx: Vec<String> = chips.iter().filter_map(|c| c.context.clone()).collect();
        self.pending_attach = chips.into_iter().filter_map(|c| c.attach).collect();
        if !self.agent_images && self.pending_attach.iter().any(|a| matches!(a, Attachment::Image(_))) {
            self.flash(format!("{} takes no images here: they go as file links", self.agent_label()));
        }
        if !ctx.is_empty() {
            let mut all = self.ask_context.take().into_iter().collect::<Vec<_>>();
            all.extend(ctx);
            self.ask_context = Some(all.join("\n"));
        }
        if q.trim().is_empty() && self.ask_context.is_some() {
            q = self.ask_default.take().unwrap_or_else(|| DELTA_QUESTION.into());
        }
        self.ask(q, key);
    }
}
