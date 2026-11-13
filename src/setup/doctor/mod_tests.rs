use super::*;

#[test]
fn npm_errors_keep_the_reason_and_the_log() {
    let out = "npm error code ECONNREFUSED\nnpm error syscall connect\nnpm error errno ECONNREFUSED\nnpm error FetchError: request to https://r/x failed\nnpm error more\nnpm error A complete log of this run can be found in: /x.log";
    let e = npm_errors(out);
    assert!(e.contains("ECONNREFUSED") && e.contains("FetchError") && e.contains("/x.log"), "{e}");
    assert!(!e.contains("npm error more"));
    assert_eq!(npm_errors("weird\noutput\n"), "weird\noutput");
}

#[test]
fn this_checkout_is_the_zendo_repo() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(is_zendo_repo(here));
    assert!(!is_zendo_repo(&here.join("src")));
    assert!(version_line().starts_with(&format!("zendo {} (", env!("CARGO_PKG_VERSION"))), "{}", version_line());
}

#[test]
fn short_and_long_commits_match() {
    assert!(same_commit("a809f25d", "a809f25d3c1e"));
    assert!(same_commit("a809f25d3c1e", "a809f25"));
    assert!(!same_commit("a809f25", "865312a"));
    assert!(!same_commit("", "865312a"));
}

#[test]
fn claude_is_detected_by_package_name() {
    assert!(is_claude_agent("npx -y @agentclientprotocol/claude-agent-acp@0.106.0"));
    assert!(is_claude_agent("node /home/x/.local/share/zendo/agent/node_modules/@agentclientprotocol/claude-agent-acp/dist/index.js"));
    assert!(!is_claude_agent("/home/claude/zendo/target/debug/zendo-mock-agent"));
}

#[test]
fn detects_the_npm_tls_error_from_the_field() {
    let s = "npm error code UNABLE_TO_VERIFY_LEAF_SIGNATURE\nnpm error request to https://registry.npmjs.org/x failed, reason: unable to verify the first certificate";
    assert!(is_tls_error(s));
    assert!(!is_tls_error("npm error code E404"));
}

#[test]
fn system_ca_is_appended_once() {
    let mut env = BTreeMap::new();
    env.insert("NODE_OPTIONS".to_string(), "--max-old-space-size=4096".to_string());
    let e = with_system_ca(&env);
    assert_eq!(e["NODE_OPTIONS"], "--max-old-space-size=4096 --use-system-ca");
    assert_eq!(with_system_ca(&e)["NODE_OPTIONS"], e["NODE_OPTIONS"]);
}

// PANES-28: zendo doctor names the hand-off key of the keymap of the user, not ctrl-o.
#[test]
fn doctor_names_the_handoff_key_of_the_user() {
    let cfg = UserConfig { keys: toml::from_str("[chat]\nhandoff = [\"ctrl-k\"]\n").unwrap(), ..UserConfig::default() };
    let key = handoff_key(&cfg);
    assert_eq!(key, "^k");
    assert!(codex_note(&key).contains("(^k)") && !codex_note(&key).contains("ctrl-o"));
}

// PANES-31: the help shows only the version, and the commit and the local copy only for a build
// with uncommitted changes.
#[test]
fn the_help_shows_the_details_of_the_version_only_for_a_build_with_changes() {
    assert_eq!(help_version_for(false), format!("zendo {}", env!("CARGO_PKG_VERSION")));
    assert_eq!(help_version_for(true), version_line());
}
