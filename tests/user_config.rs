//! The config file of the user that the app reads and writes: the file that `Config` names, not
//! the `~/.zendo/config.toml` of the user who runs the tests (AGENTS.md, Tests).

mod common;
use common::*;
use zendo::app::{App, Config};

fn app_with_config(root: &std::path::Path, user_config: std::path::PathBuf) -> App {
    App::new(Config {
        root: root.to_path_buf(),
        notes_dir: root.join("docs/notes"),
        agent: env!("CARGO_BIN_EXE_zendo-mock-agent").to_string(),
        agent_env: Default::default(),
        offline: false,
        notes_repo: None,
        user_config,
    })
    .unwrap()
}

// LOFI-11, ATTN-9: the app starts with the settings of the config file that `Config` names.
#[test]
fn the_app_reads_the_config_file_that_config_names() {
    let root = repo("user-config-read", &[]);
    let file = root.join("zendo-config.toml");
    std::fs::write(&file, "music = false\nsound = false\n").unwrap();
    let a = app_with_config(&root, file);
    assert!(!a.music.on, "music = false");
    assert!(!a.attention.sound, "sound = false");
    let _ = std::fs::remove_dir_all(&root);
}

// LOFI-11, ATTN-9: without the file, the app starts with the defaults.
#[test]
fn without_the_config_file_the_app_starts_with_the_defaults() {
    let root = repo("user-config-none", &[]);
    let a = app_with_config(&root, root.join("zendo-config.toml"));
    assert!(a.music.on && !a.music.asked, "the music plays and the question comes");
    assert!(a.attention.sound);
    let _ = std::fs::remove_dir_all(&root);
}

// ATTN-8, LOFI-8, TIPS-7: the buttons save their settings in the config file that `Config` names.
#[test]
fn the_settings_go_to_the_config_file_that_config_names() {
    let root = repo("user-config-write", &[]);
    let file = root.join("zendo-config.toml");
    let a = app_with_config(&root, file.clone());
    assert_eq!([&a.attention.config, &a.music.config, &a.tips.config, &a.rekey.config], [&file; 4]);
    let mut a = a;
    a.toggle_sound();
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("sound = false"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
}

// LOFI-8, ATTN-8, TIPS-7: an app that `App::default()` makes has no config file, so a button
// never writes `~/.zendo/config.toml`.
#[test]
fn a_default_app_names_no_config_file() {
    let a = App::default();
    let none = std::path::PathBuf::new();
    assert_eq!([&a.attention.config, &a.music.config, &a.tips.config, &a.rekey.config], [&none; 4]);
    let saved = zendo::setup::config::UserConfig::update(&none, |u| u.sound = Some(false));
    assert!(saved.is_err(), "no file to save to");
}
