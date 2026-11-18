use super::*;
use crate::keys::KeyConfig;

/// A clean folder of the test: what an interrupted run left goes first.
fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-keys-write-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("a test folder");
    d
}

fn keys(list: &[&str]) -> Vec<Key> {
    list.iter().map(|s| Key::parse(s).expect("a key")).collect()
}

/// REKEY-16: the keymap file that the config names gets the keys; its comments stay.
#[test]
fn the_keymap_file_gets_the_keys_and_keeps_its_comments() {
    let d = dir("file");
    let file = d.join("keymap.toml");
    std::fs::write(&file, "# my keys\n[global]\n# the tree\ntoggle_tree = [\"alt-b\", \"ctrl-shift-b\"]  # file tree\nhelp = [\"f1\"]\n").unwrap();
    let cfg = d.join("config.toml");
    std::fs::write(&cfg, format!("keymap = \"{}\"\n", file.display())).unwrap();
    let got = write_keys(&cfg, Pane::Global, "toggle_tree", &keys(&["alt-t", "ctrl-shift-b"])).expect("written");
    assert_eq!(got, file);
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("toggle_tree = [\"alt-t\", \"ctrl-shift-b\"]  # file tree"), "{text}");
    assert!(text.starts_with("# my keys\n[global]\n# the tree\n"), "{text}");
    assert!(text.contains("help = [\"f1\"]"), "{text}");
}

/// REKEY-16: a pane that the file does not have comes in as a table.
#[test]
fn a_new_pane_comes_into_the_keymap_file() {
    let d = dir("pane");
    let file = d.join("keymap.toml");
    std::fs::write(&file, "[global]\nhelp = [\"f1\"]\n").unwrap();
    let cfg = d.join("config.toml");
    std::fs::write(&cfg, format!("keymap = \"{}\"\n", file.display())).unwrap();
    write_keys(&cfg, Pane::Read, "ask", &keys(&["alt-a"])).expect("written");
    let text = std::fs::read_to_string(&file).unwrap();
    let parsed: KeyConfig = toml::from_str(&text).expect("toml");
    assert_eq!(parsed["read"]["ask"], KeySpec::Many(vec!["alt-a".into()]));
    assert_eq!(parsed["global"]["help"], KeySpec::Many(vec!["f1".into()]));
}

/// REKEY-16: where `[keys.<pane>]` of the config sets the action, the config gets the keys.
#[test]
fn keys_of_the_config_change_in_the_config() {
    let d = dir("config");
    let cfg = d.join("config.toml");
    std::fs::write(&cfg, "# mine\nwidth = 80\n\n[keys.read]\nask = \"x\" # ask\n").unwrap();
    let got = write_keys(&cfg, Pane::Read, "ask", &keys(&["alt-x"])).expect("written");
    assert_eq!(got, cfg);
    let text = std::fs::read_to_string(&cfg).unwrap();
    assert!(text.starts_with("# mine\nwidth = 80\n"), "{text}");
    let user = UserConfig::load_from(&cfg);
    assert_eq!(user.keys["read"]["ask"], KeySpec::Many(vec!["alt-x".into()]));
}

/// REKEY-16: without a keymap file, a new `keymap.toml` beside the config, named in the config.
#[test]
fn without_a_keymap_file_a_new_one_comes() {
    let d = dir("new");
    let cfg = d.join("config.toml");
    std::fs::write(&cfg, "width = 80\n").unwrap();
    let got = write_keys(&cfg, Pane::Global, "toggle_tree", &keys(&["alt-t"])).expect("written");
    assert_eq!(got, d.join("keymap.toml"));
    let user = UserConfig::load_from(&cfg);
    assert_eq!(user.width, Some(80));
    let file = user.keymap_file().expect("readable").expect("named");
    assert_eq!(file.len(), 1, "only the changed action: {file:?}");
    assert_eq!(file["global"]["toggle_tree"], KeySpec::Many(vec!["alt-t".into()]));
}

/// REKEY-29: no key is an empty list.
#[test]
fn no_key_is_an_empty_list() {
    let d = dir("empty");
    let cfg = d.join("config.toml");
    write_keys(&cfg, Pane::Read, "peek_disk", &[]).expect("written");
    let file = UserConfig::load_from(&cfg).keymap_file().expect("readable").expect("named");
    assert_eq!(file["read"]["peek_disk"], KeySpec::Many(Vec::new()));
}

/// REKEY-27: a write that fails says why.
#[test]
fn a_failed_write_says_why() {
    let d = dir("fail");
    let cfg = d.join("config.toml");
    std::fs::write(&cfg, format!("keymap = \"{}\"\n", d.join("no/such/dir/keymap.toml").display())).unwrap();
    let err = write_keys(&cfg, Pane::Read, "ask", &keys(&["alt-a"])).expect_err("no folder");
    assert!(err.contains("keymap.toml"), "{err}");
}
