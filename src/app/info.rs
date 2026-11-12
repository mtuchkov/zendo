//! Small getters of the app state.
use super::*;

impl App {
    /// Fill the file tree. In a repo: the repo. Outside a repo (the home folder): only the notes.
    pub fn scan_tree(&mut self) {
        if self.cfg.root.join(".git").exists() {
            self.tree.scan(&self.cfg.root);
        } else {
            let under = self.cfg.notes_dir.clone();
            self.tree.scan_in(&self.cfg.root, &under);
        }
    }

    /// The agent profile, when zendo knows the agent.
    pub fn profile(&self) -> Option<&'static crate::setup::config::Profile> {
        crate::setup::config::Profile::by_kind(crate::setup::config::kind_of(&self.cfg.agent))
    }

    /// "note" for a file of the notes folder, "document" for any other file.
    pub fn file_word(&self) -> &'static str {
        match &self.current {
            Some(p) if !p.starts_with(&self.store.dir) => "document",
            _ => "note",
        }
    }

    /// The agent as the user knows it: claude or codex; else the name the agent gave.
    pub fn agent_short(&self) -> String {
        match self.profile() {
            Some(p) => p.name.to_lowercase(),
            None => self.agent_name.clone().unwrap_or_default(),
        }
    }

    pub fn agent_label(&self) -> &'static str {
        self.profile().map_or("this agent", |p| p.name)
    }

    /// The command line for `ctrl-o`, when the agent has one with a resume: the one that the app
    /// sets, else the one of the agent profile.
    pub fn handoff_cli(&self) -> Option<crate::session::Command> {
        self.cli.command.clone().or_else(|| {
            let p = self.profile()?;
            Some(crate::session::Command { program: p.cli.into(), args: p.handoff?.iter().map(|s| s.to_string()).collect() })
        })
    }

    /// A feature that the agent cannot do: its keys stay, and say so.
    pub fn agent_can(&self, action: &str) -> bool {
        action != "handoff" || self.handoff_cli().is_some()
    }

    /// The word limit of a file: from project-structure.toml, else 1,000.
    pub fn word_limit_for(&self, path: &Path) -> usize {
        let rel = path.strip_prefix(&self.cfg.root).unwrap_or(path);
        self.tree.structure.as_ref().and_then(|s| s.words_for(rel)).unwrap_or(DEFAULT_WORDS)
    }

    /// `S`: ask Claude to split the open document, and keep the altitude.
    pub fn split_document(&mut self) {
        if self.over_limit.is_none() {
            self.flash(format!("{} words: within the limit of {}", self.word_count, self.word_limit));
            return;
        }
        let Some(path) = self.current.clone() else { return };
        let rel = self.rel(&path);
        let ctx = SPLIT_PROMPT.replace("{{FILE}}", &rel).replace("{{WORDS}}", &self.word_count.to_string()).replace("{{LIMIT}}", &self.word_limit.to_string());
        self.open_panel();
        if !self.panel {
            return;
        }
        self.ask_context = Some(ctx);
        let scope = self.scope();
        self.ask(format!("Split {rel}: {} words, the limit is {}. Keep the altitude.", self.word_count, self.word_limit), scope);
    }

    /// What the notes sync wants the user to know, if anything.
    pub fn sync_label(&self) -> Option<String> {
        self.syncer.as_ref().and_then(|s| s.state().label())
    }

    pub fn theme(&self) -> &Theme {
        &THEMES[self.theme]
    }

    pub fn note(&self) -> Option<&Note> {
        let p = self.current.as_ref()?;
        self.notes.iter().find(|n| &n.path == p)
    }

    pub fn note_index(&self) -> Option<usize> {
        let p = self.current.as_ref()?;
        self.notes.iter().position(|n| &n.path == p)
    }

    pub fn rel(&self, p: &Path) -> String {
        self.store.rel(p).display().to_string()
    }

    /// The key of the chat thread: the thread of the active agent session, for every file.
    pub fn key(&self) -> String {
        session_key(self.state.session.min(SESSIONS - 1))
    }

    /// What a question is about: the file on screen, a new note, or all notes.
    pub fn scope(&self) -> String {
        if let Some(p) = &self.pinned {
            return p.clone();
        }
        if self.mode == Mode::Write && self.editor_path.is_none() {
            return NEW_KEY.into();
        }
        match &self.current {
            Some(p) => self.rel(p),
            None => ALL_KEY.into(),
        }
    }

    pub fn flash(&mut self, s: impl Into<String>) {
        self.flash = Some((s.into(), Instant::now()));
    }

    /// The width of the text column: the one of this repository, else the one of the config.
    pub fn column_width(&self) -> u16 {
        self.state.column.unwrap_or(self.width_default)
    }

    /// The repository has no project structure file, and zendo doctor can make one.
    pub fn structure_missing(&self) -> bool {
        self.tree.structure.is_none() && self.structure_error.is_none() && self.cfg.root.join(".git").exists() && !self.cfg.root.join(crate::docs::structure::FILE).exists()
    }

    /// The width of the code column: the one of this repository, else 120.
    pub fn code_width(&self) -> u16 {
        self.state.code_column.unwrap_or(crate::text::code::CODE_WIDTH)
    }

    /// `alt-=` and `alt--`: a wider or a narrower column, kept for the repository: the code column
    /// on a code file, else the text column.
    pub fn change_width(&mut self, by: i16) {
        let code = self.current.as_deref().is_some_and(crate::text::code::is_code);
        let w = ((if code { self.code_width() } else { self.column_width() }) as i16 + by).clamp(30, 300) as u16;
        *(if code { &mut self.state.code_column } else { &mut self.state.column }) = Some(w);
        self.state.save(&self.cfg.root);
        self.flash(format!("{} width {w}", if code { "code" } else { "text" }));
    }
}
