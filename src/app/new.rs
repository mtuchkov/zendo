//! Start: the app state from the config.
use super::*;

impl App {
    pub fn new(cfg: Config) -> anyhow::Result<Self> {
        let store = Store::new(cfg.root.clone(), cfg.notes_dir.clone());
        let files = Files::start(store)?;
        let state = State::load(&cfg.root);
        let theme = theme::index_of(&state.theme);
        let branch = notes::git_branch(&cfg.root);
        let user = crate::setup::config::UserConfig::load();
        let (keys, key_problems) = user.keymap();
        let name_model = user.name_model(crate::setup::config::kind_of(&cfg.agent));
        crate::text::highlight::init(user.highlight.as_ref());
        let mut app = Self {
            cfg,
            files,
            theme,
            theme_before: theme,
            state,
            branch,
            editor: Editor::new(""),
            keys,
            key_problems,
            keymap_file: user.keymap.clone(),
            frame: crate::ui::frame::Frame::sized(20, 70, DEFAULT_WORDS),
            width_default: user.width.unwrap_or(crate::ui::COLUMN).clamp(30, 300),
            attention: Attention::from_config(&user),
            music: crate::music::Music::from_config(&user),
            name_model,
            ..Default::default()
        };
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
