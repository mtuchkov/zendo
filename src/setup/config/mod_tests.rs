use super::*;

#[test]
fn old_folders_move_to_the_new_name_once() {
    let base = std::env::temp_dir().join(format!("zendo-move-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let repo = base.join("repo");
    for d in [".config/dawn", ".local/share/dawn/agent", ".cache/dawn/target", "repo/.dawn"] {
        std::fs::create_dir_all(base.join(d)).unwrap();
    }
    let agent = format!("node {}/.local/share/dawn/agent/index.js", base.display());
    std::fs::write(base.join(".config/dawn/config.json"), format!("{{\"agent\": \"{agent}\"}}")).unwrap();
    // The folders are arguments: a test must not change `HOME` of the process, which the tests share.
    let dirs = Dirs { legacy_config: base.join(".config/zendo"), data: base.join(".local/share/zendo"), cache: base.join(".cache/zendo"), config: base.join(".zendo/config.toml") };
    assert_eq!(move_old_folders_in(&repo, &dirs).len(), 5);
    for d in [".local/share/zendo/agent", ".cache/zendo/target", "repo/.zendo"] {
        assert!(base.join(d).is_dir(), "{d}");
    }
    assert!(!base.join(".config/zendo").exists(), "the JSON config became TOML, and its folder is gone");
    let text = std::fs::read_to_string(base.join(".zendo/config.toml")).unwrap();
    assert!(text.contains("/.local/share/zendo/agent/") && !text.contains("/dawn/"), "{text}");
    std::fs::create_dir_all(base.join(".local/share/dawn")).unwrap();
    assert!(move_old_folders_in(&repo, &dirs).is_empty(), "a folder moves only once");
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

// CMD-17: `a broken config stays as the user wrote it`: a change of a setting fails on a config
// that does not parse and leaves the file as it is; a good config keeps its other settings; without
// a file, the change makes one.
#[test]
fn a_broken_config_stays_as_the_user_wrote_it() {
    let dir = std::env::temp_dir().join(format!("zendo-broken-config-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("config.toml");
    let broken = "agent = \"my-agent\"\nwidth = \"wide\"\n";
    std::fs::write(&path, broken).unwrap();
    let err = UserConfig::update(&path, |c| c.sound = Some(false)).unwrap_err();
    assert!(err.contains("config.toml"), "{err}");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), broken);
    assert!(UserConfig::for_change(&path).is_err());
    // A file that is not UTF-8, as a comment saved in Latin-1, is not a missing file.
    let latin1: &[u8] = b"agent = \"my-agent\" # caf\xE9\n";
    std::fs::write(&path, latin1).unwrap();
    assert!(UserConfig::update(&path, |c| c.sound = Some(false)).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), latin1);
    std::fs::write(&path, "agent = \"my-agent\"\nwidth = 80\n").unwrap();
    UserConfig::update(&path, |c| c.sound = Some(false)).unwrap();
    let c = UserConfig::load_from(&path);
    assert_eq!((c.agent.as_deref(), c.width, c.sound), (Some("my-agent"), Some(80), Some(false)));
    std::fs::remove_file(&path).unwrap();
    UserConfig::update(&path, |c| c.music = Some(true)).unwrap();
    assert_eq!(UserConfig::load_from(&path).music, Some(true));
    let _ = std::fs::remove_dir_all(&dir);
}
