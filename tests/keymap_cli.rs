//! `zendo keymap` prints the default keymap file of the platform, ready to copy and change.

// PANES-23, PANES-29: the user starts a keymap file of their own from the default one of the platform.
#[test]
fn zendo_keymap_prints_the_default_keymap() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_zendo")).arg("keymap").output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert_eq!(text, zendo::keys::platform_keymap(zendo::keys::MACOS));
    let parsed: zendo::keys::KeyConfig = toml::from_str(&text).unwrap();
    assert!(parsed.contains_key("read") && parsed.contains_key("write"));
}
