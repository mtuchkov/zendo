//! Start: the app state from the config.
use super::*;

impl App {
    pub fn new(cfg: Config) -> anyhow::Result<Self> {
        let store = Store::new(cfg.root.clone(), cfg.notes_dir.clone());
        let mut files = Files::start(store)?;
        files.set_branch(notes::git_branch(&cfg.root));
        let mut state = State::load(&cfg.root);
        // Each start shows the structure view, else docs; the view of the last run goes (FILES-66).
        state.tree_view.clear();
        state.tree_all = false;
        let theme = theme::index_of(&state.theme);
        let user = crate::setup::config::UserConfig::load_from(&cfg.user_config);
        let (keys, key_problems) = user.keymap();
        let kind = crate::setup::config::kind_of(&cfg.agent);
        let name_model = user.name_model(kind);
        // The light model gets the variables of the agents (ENV-1).
        let with_env = |a: crate::agent::api::Api| crate::agent::api::Api { env: cfg.agent_env.vars.clone(), ..a };
        let api = crate::agent::api::Api::find(kind, name_model.clone(), &crate::agent::api::on_path).map(with_env);
        let hints = crate::agent::api::Api::find(kind, user.hint_model(kind), &crate::agent::api::on_path).map(with_env);
        crate::text::highlight::init(user.highlight.as_ref());
        let logs = crate::setup::logs::Logs::load(&cfg.root);
        let mut app = Self {
            logs,
            cfg,
            files,
            theme,
            theme_before: theme,
            state,
            keys,
            key_help: KeyHelp::new(key_problems, user.keymap.clone()),
            frame: crate::ui::frame::Frame::sized(20, 70, DEFAULT_WORDS),
            width_default: user.width.unwrap_or(crate::ui::COLUMN).clamp(30, 300),
            attention: Attention::from_config(&user),
            bg: Background::default(),
            music: crate::music::Music::from_config(&user),
            link: AgentLink::with_name_model(name_model),
            ..Default::default()
        };
        // The buttons and floats save their settings in this file (LOFI-8, ATTN-8, TIPS-7).
        for config in [&mut app.attention.config, &mut app.music.config, &mut app.tips.config, &mut app.rekey.config] {
            config.clone_from(&app.cfg.user_config);
        }
        app.cli.env = app.cfg.agent_env.vars.clone();
        app.cli.shell_env = app.cfg.agent_env.shell_vars();
        app.link.start_api(api);
        app.rekey.api = hints.map(|a| crate::agent::api::Api { effort: Some("low".into()), ..a });
        // The chat input of each session, as the user left it (CLAUDE-45).
        let drafts = app.state.drafts.clone();
        app.chat.load(&drafts);
        // The last document of the active session, where the user left it (FILES-55).
        app.open_session_doc();
        app.files.find.seen = app.files.current().cloned();
        app.load_structure();
        // The tree was open last time: fill it; with no note on screen, start in it.
        if app.state.tree {
            app.scan_tree();
            if app.files.current().is_none() {
                app.focus = Focus::Tree;
            }
        }
        Ok(app)
    }
}
