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

// CMD-4, CMD-20, ADR 0023: zendo asks GitHub at most once a day; while the last check is younger,
// its answer holds, and an empty answer (GitHub did not answer) gives no notice.
#[test]
fn the_update_check_asks_github_once_a_day() {
    let now = 1_000_000;
    let day = UPDATE_EVERY.as_secs();
    assert_eq!(day, 24 * 3600);
    assert_eq!(cached_head(Some((now - 60, "a809f25".into())), now), Some(Some("a809f25".into())), "fresh: no network");
    assert_eq!(cached_head(Some((now - 60, String::new())), now), Some(None), "fresh, GitHub did not answer");
    assert_eq!(cached_head(Some((now - day, "a809f25".into())), now), None, "a day old: ask again");
    assert_eq!(cached_head(None, now), None, "never asked");
}

// ADR 0023: the ssh of a check or an upgrade accepts no new host key without the user.
#[test]
fn ssh_accepts_no_new_host_key() {
    let env = super::upgrade::git_env();
    assert!(env.get("GIT_SSH_COMMAND").is_none_or(|c| !c.contains("accept-new")), "{env:?}");
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
    // The home folder is an argument: another test changes `HOME` of the process while this one runs.
    let home = "/no/such/home";
    assert_eq!(help_version_for(false, home), format!("zendo {}", env!("CARGO_PKG_VERSION")));
    assert_eq!(help_version_for(true, home), version_line_in(home));
}

// CMD-8: after an upgrade, the new program runs `zendo doctor --fix`.
#[test]
fn an_upgrade_ends_with_doctor_fix() {
    assert_eq!(AFTER_UPGRADE, ["doctor", "--fix"]);
}

// ENV-5: an agent that starts after `zendo setup` saved `system_ca` gets the system CA in the same
// run; without the setting, the agent environment as it is.
#[test]
fn the_system_ca_of_the_config_counts_in_the_same_run() {
    let paths = Paths { root: PathBuf::new(), notes_dir: PathBuf::new(), repo_notes: PathBuf::new(), env: Default::default() };
    let on = UserConfig { system_ca: Some(true), ..Default::default() };
    assert!(agent_vars(&on, &paths)["NODE_OPTIONS"].contains(SYSTEM_CA));
    assert!(!agent_vars(&UserConfig::default(), &paths).contains_key("NODE_OPTIONS"));
}

// ENV-1: the checks of the network and of the login look at the agent environment: a CA file and
// an API key that only a profile sets count.
#[test]
fn the_checks_look_at_the_agent_environment() {
    let env = BTreeMap::from([("NODE_EXTRA_CA_CERTS".to_string(), "/zendo-no-such/ca.pem".to_string()), ("ANTHROPIC_API_KEY".to_string(), "k".to_string())]);
    let mut o = Out::new();
    checks::check_network(&env, &mut o);
    assert!(o.failed, "the CA file of the agent environment is missing");
    assert!(has_api_key(&env));
}
