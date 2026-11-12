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
    // SAFETY: this test is the only one that changes these variables.
    unsafe {
        std::env::set_var("XDG_CONFIG_HOME", base.join(".config"));
        std::env::set_var("XDG_DATA_HOME", base.join(".local/share"));
        std::env::set_var("XDG_CACHE_HOME", base.join(".cache"));
    }
    unsafe { std::env::set_var("HOME", &base) };
    assert_eq!(move_old_folders(&repo).len(), 5);
    for d in [".local/share/zendo/agent", ".cache/zendo/target", "repo/.zendo"] {
        assert!(base.join(d).is_dir(), "{d}");
    }
    assert!(!base.join(".config/zendo").exists(), "the JSON config became TOML, and its folder is gone");
    let text = std::fs::read_to_string(base.join(".zendo/config.toml")).unwrap();
    assert!(text.contains("/.local/share/zendo/agent/") && !text.contains("/dawn/"), "{text}");
    std::fs::create_dir_all(base.join(".local/share/dawn")).unwrap();
    assert!(move_old_folders(&repo).is_empty(), "a folder moves only once");
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
