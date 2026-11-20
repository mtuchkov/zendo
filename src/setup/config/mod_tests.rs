use super::*;

// CODEXCLI-1, CODEXCLI-2: the profile builds the command line of the hand-off; only Claude Code
// gets the options of Claude Code.
#[test]
fn the_profile_builds_the_command_line_of_the_handoff() {
    let codex = CODEX.handoff_command().expect("Codex has a hand-off");
    assert_eq!((codex.program.as_str(), codex.args, codex.claude), ("codex", vec!["resume".to_string()], false));
    let claude = CLAUDE.handoff_command().expect("Claude has a hand-off");
    assert_eq!((claude.program.as_str(), claude.args, claude.claude), ("claude", vec!["--resume".to_string()], true));
}

// The JSON config of before 0.11.1 becomes the TOML config once.
#[test]
fn the_json_config_becomes_toml_once() {
    let base = std::env::temp_dir().join(format!("zendo-move-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (legacy, config) = (base.join(".config/zendo"), base.join(".zendo/config.toml"));
    std::fs::create_dir_all(&legacy).unwrap();
    std::fs::write(legacy.join("config.json"), "{\"agent\": \"node /x/agent/index.js\"}").unwrap();
    // The folders are arguments: a test must not change `HOME` of the process, which the tests share.
    assert_eq!(json_config_to_toml(&legacy, &config), Some((legacy.join("config.json"), config.clone())));
    assert!(!legacy.exists(), "the JSON config became TOML, and its folder is gone");
    assert!(std::fs::read_to_string(&config).unwrap().contains("node /x/agent/index.js"));
    assert_eq!(json_config_to_toml(&legacy, &config), None, "the config moves only once");
    let _ = std::fs::remove_dir_all(&base);
}

// ATTN-8, ATTN-9: `sound` in the config, and a config file in another place.
#[test]
fn sound_goes_into_the_config_and_back() {
    let c: UserConfig = toml::from_str("sound = false").unwrap();
    assert_eq!(c.sound, Some(false));
    assert_eq!(UserConfig::default().sound, None, "no setting: the default");
    let path = std::env::temp_dir().join(format!("zendo-sound-{}/config.toml", std::process::id()));
    let c = UserConfig { sound: Some(true), ..Default::default() };
    c.save_to(&path).unwrap();
    assert!(std::fs::read_to_string(&path).unwrap().contains("sound = true"));
    assert_eq!(UserConfig::load_from(&path).sound, Some(true));
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

// NAMES-12, NAMES-13: the model that names the agent sessions: `haiku` for Claude and `luna` for
// Codex by default, the one of `[name_models]` in the config, none for another agent or when the
// config sets an empty name.
#[test]
fn the_name_model_comes_from_the_config_with_defaults() {
    let none = UserConfig::default();
    assert_eq!(none.name_model(Kind::Claude).as_deref(), Some("haiku"));
    assert_eq!(none.name_model(Kind::Codex).as_deref(), Some("luna"));
    assert_eq!(none.name_model(Kind::Other), None);
    let set: UserConfig = toml::from_str("[name_models]\nclaude = \"sonnet\"\ncodex = \"\"\nother = \"small\"\n").unwrap();
    assert_eq!(set.name_model(Kind::Claude).as_deref(), Some("sonnet"));
    assert_eq!(set.name_model(Kind::Codex), None);
    assert_eq!(set.name_model(Kind::Other).as_deref(), Some("small"));
}

// FAIL-2, CMD-19: a config that is a link into a dotfiles repository stays a link when zendo saves a
// setting; the file that the link names gets only the changed line.
#[cfg(unix)]
#[test]
fn a_saved_setting_keeps_a_linked_config_a_link() {
    let base = std::env::temp_dir().join(format!("zendo-config-link-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("dotfiles")).unwrap();
    std::fs::write(base.join("dotfiles/config.toml"), "# my settings\nsound = true\n").unwrap();
    std::os::unix::fs::symlink(base.join("dotfiles/config.toml"), base.join("config.toml")).unwrap();
    let path = base.join("config.toml");
    let before = UserConfig::load_from(&path);
    let mut after = UserConfig::load_from(&path);
    after.sound = Some(false);
    after.save_changes(&path, &before).unwrap();
    assert!(std::fs::symlink_metadata(&path).unwrap().file_type().is_symlink(), "the link stays");
    assert_eq!(std::fs::read_to_string(base.join("dotfiles/config.toml")).unwrap(), "# my settings\nsound = false\n");
    let _ = std::fs::remove_dir_all(&base);
}

// TELEM-12: `stats = false` turns off the telemetry log; the old name `usage = false` works the
// same, and `stats` wins when both are there.
#[test]
fn stats_false_turns_the_stats_off() {
    let stats = |text: &str| toml::from_str::<UserConfig>(text).unwrap().stats();
    assert!(stats(""), "on by default");
    assert!(!stats("stats = false"));
    assert!(!stats("usage = false"), "the old name");
    assert!(stats("stats = true\nusage = false"), "the new name wins");
}
