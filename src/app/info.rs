//! Small getters of the app state.
use super::*;

impl App {
    /// Fill the file tree. In a repo: the repo. Outside a repo (the home folder): only the notes.
    pub fn scan_tree(&mut self) {
        if self.facts.exists.get(&self.cfg.root.join(".git")) == Some(true) {
            // FILES-25: the notes of the notes repo live outside the repository.
            let notes = self.repo_notes_outside();
            // FILES-32: the worktree views show the worktree of the active agent session.
            let worktree = self.worktree_of(self.state.session);
            // SESSIONS-23: agent session 0 works in the main checkout.
            self.files.set_main_checkout(self.state.session == 0);
            self.files.scan(&self.cfg.root, notes, worktree);
        } else {
            self.files.scan_in(&self.cfg.root, &self.cfg.notes_dir);
        }
    }

    /// The notes folder in the notes repo, while the notes repo is on and the folder is outside the
    /// repository (FILES-25, FIND-18).
    pub fn repo_notes_outside(&self) -> Option<PathBuf> {
        self.cfg.notes_repo.as_ref().map(|_| self.cfg.notes_dir.clone()).filter(|d| !d.starts_with(&self.cfg.root))
    }

    /// The agent profile, when zendo knows the agent.
    pub fn profile(&self) -> Option<&'static crate::setup::config::Profile> {
        crate::setup::config::Profile::by_kind(crate::setup::config::kind_of(&self.cfg.agent))
    }

    /// "note" for a file of the notes folder, "document" for any other file.
    pub fn file_word(&self) -> &'static str {
        match self.files.current() {
            Some(p) if !p.starts_with(&self.files.store().dir) => "document",
            _ => "note",
        }
    }

    /// The agent as the user knows it: claude or codex; else the name the agent gave.
    pub fn agent_short(&self) -> String {
        match self.profile() {
            Some(p) => p.name.to_lowercase(),
            None => self.link.name().unwrap_or_default().to_string(),
        }
    }

    pub fn agent_label(&self) -> &'static str {
        self.profile().map_or("this agent", |p| p.name)
    }

    /// The command line for `ctrl-o`, when the agent has one with a resume: the one that the app
    /// sets, else the one of the agent profile.
    pub fn handoff_cli(&self) -> Option<crate::session::Command> {
        self.cli.command.clone().or_else(|| self.profile()?.handoff_command())
    }

    /// A feature that the agent cannot do: its keys stay, and say so.
    pub fn agent_can(&self, action: &str) -> bool {
        action != "handoff" || self.handoff_cli().is_some()
    }

    /// The word limit of a file: from project-structure.toml, else 1,000.
    pub fn word_limit_for(&self, path: &Path) -> usize {
        let rel = path.strip_prefix(&self.cfg.root).unwrap_or(path);
        self.files.structure().and_then(|s| s.words_for(rel)).unwrap_or(DEFAULT_WORDS)
    }

    /// `S`: ask Claude to split the open document, and keep the altitude.
    pub fn split_document(&mut self) {
        if self.frame.over_limit().is_none() {
            self.flash(format!("{} words: within the limit of {}", self.frame.word_count(), self.frame.word_limit()));
            return;
        }
        let Some(path) = self.files.current().cloned() else { return };
        let rel = self.rel(&path);
        let ctx = SPLIT_PROMPT.replace("{{FILE}}", &rel).replace("{{WORDS}}", &self.frame.word_count().to_string()).replace("{{LIMIT}}", &self.frame.word_limit().to_string());
        self.open_panel();
        if !self.threads.shown() {
            return;
        }
        self.composer_mut().set_context(ctx);
        let scope = self.scope();
        self.ask(format!("Split {rel}: {} words, the limit is {}. Keep the altitude.", self.frame.word_count(), self.frame.word_limit()), scope);
    }

    /// What the notes sync wants the user to know, if anything.
    pub fn sync_label(&self) -> Option<String> {
        self.files.sync_label()
    }

    pub fn theme(&self) -> &Theme {
        &THEMES[self.theme]
    }

    pub fn note(&self) -> Option<&Note> {
        self.files.note()
    }

    pub fn note_index(&self) -> Option<usize> {
        self.files.note_index()
    }

    pub fn rel(&self, p: &Path) -> String {
        self.files.rel(p)
    }

    /// The key of the chat thread: the thread of the active agent session, for every file.
    /// The chat input of the active agent session (ADR 0011).
    pub fn composer(&self) -> &Composer {
        self.chat.composer(self.state.session)
    }

    pub fn composer_mut(&mut self) -> &mut Composer {
        self.chat.composer_mut(self.state.session)
    }

    pub fn key(&self) -> String {
        session_key(self.state.session.min(SESSIONS - 1))
    }

    /// What a question is about: the file on screen, a new note, or all notes.
    pub fn scope(&self) -> String {
        if let Some(p) = self.threads.pinned() {
            return p.to_string();
        }
        if self.mode == Mode::Write && self.writer.path().is_none() {
            return NEW_KEY.into();
        }
        match self.files.current() {
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

    /// The repository has no project structure file, and zendo init can make one.
    pub fn structure_missing(&self) -> bool {
        // Ask the memo for both paths in each frame, so that the worker learns both at once.
        let (git, file) = (self.facts.exists.get(&self.cfg.root.join(".git")), self.facts.exists.get(&self.cfg.root.join(crate::docs::structure::FILE)));
        self.files.structure().is_none() && self.files.structure_error().is_none() && git == Some(true) && file == Some(false)
    }

    /// The width of the code column: the one of this repository, else 120.
    pub fn code_width(&self) -> u16 {
        self.state.code_column.unwrap_or(crate::text::code::CODE_WIDTH)
    }

    /// `alt-=` and `alt--`: a wider or a narrower column, kept for the repository: the code column
    /// on a code file, else the text column.
    pub fn change_width(&mut self, by: i16) {
        let code = self.files.current_path().is_some_and(crate::text::code::is_code);
        let w = ((if code { self.code_width() } else { self.column_width() }) as i16 + by).clamp(30, 300) as u16;
        *(if code { &mut self.state.code_column } else { &mut self.state.column }) = Some(w);
        self.save_state();
        self.flash(format!("{} width {w}", if code { "code" } else { "text" }));
    }
}
