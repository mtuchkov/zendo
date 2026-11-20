//! The agent environment (docs/zendo/agent-env.md) and its profiles (docs/zendo/agent-profiles.md):
//! the variables that zendo gives to each agent that it starts, over the environment of zendo.

use super::{UserConfig, config_path, home};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The option of Node.js that makes it use the CA certificates of the system.
pub const SYSTEM_CA: &str = "--use-system-ca";

/// The variables over the environment of zendo, the profile that gave some of them, and what zendo
/// shows once at the start (ENV-7).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AgentEnv {
    pub vars: BTreeMap<String, String>,
    pub profile: Option<String>,
    pub problems: Vec<String>,
}

/// The agent environment of the repository that holds `dir`, with the profile `flag` of
/// `--profile` (ENV-2, ENV-3, PROFILE-1, PROFILE-2). An error when the profile is not in the
/// config (PROFILE-3).
pub fn agent_env(cfg: &UserConfig, dir: &Path, flag: Option<&str>) -> Result<AgentEnv, String> {
    let main = PathBuf::from(crate::board::repo_of(dir));
    build(cfg, &main, flag, &home(), std::env::var("NODE_OPTIONS").ok())
}

/// The agent environment of the main checkout `main`: `.envrc`, then the profile, then the system
/// CA over the `NODE_OPTIONS` of zendo, `node_options`.
pub(super) fn build(cfg: &UserConfig, main: &Path, flag: Option<&str>, home: &Path, node_options: Option<String>) -> Result<AgentEnv, String> {
    let profile = select_profile(cfg, flag, main, home)?;
    let path = main.join(".envrc");
    let (mut vars, problems) = match std::fs::read_to_string(&path) {
        Ok(text) => {
            let (vars, skipped) = read_envrc(&text);
            (vars, skipped_problem(&path, &skipped).into_iter().collect())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Default::default(),
        Err(e) => (BTreeMap::new(), vec![format!("cannot read {}: {e}", path.display())]),
    };
    let set = profile.as_ref().and_then(|p| cfg.profiles.get(p)).into_iter().flatten();
    vars.extend(set.map(|(k, v)| (k.clone(), expand(v, home).display().to_string())));
    if cfg.system_ca == Some(true) {
        let now = vars.get("NODE_OPTIONS").cloned().or(node_options).unwrap_or_default();
        if !now.split_whitespace().any(|o| o == SYSTEM_CA) {
            vars.insert("NODE_OPTIONS".into(), format!("{now} {SYSTEM_CA}").trim().to_string());
        }
    }
    Ok(AgentEnv { vars, profile, problems })
}

/// The message about the lines of `.envrc` that zendo skipped, if there are any (ENV-7).
fn skipped_problem(path: &Path, skipped: &[usize]) -> Option<String> {
    let lines: Vec<String> = skipped.iter().map(usize::to_string).collect();
    let word = if lines.len() == 1 { "line" } else { "lines" };
    (!lines.is_empty()).then(|| format!("{}: zendo skipped {word} {}: it reads only `NAME=value` lines", path.display(), lines.join(", ")))
}

/// The variables of `.envrc` and the numbers of the lines that are not variables, from 1 (ENV-4).
/// zendo runs nothing: a value is taken as it is written.
pub fn read_envrc(text: &str) -> (BTreeMap<String, String>, Vec<usize>) {
    let mut vars = BTreeMap::new();
    let mut skipped = Vec::new();
    let lines = text.lines().enumerate().map(|(n, l)| (n + 1, l.trim()));
    for (n, line) in lines.filter(|(_, l)| !l.is_empty() && !l.starts_with('#')) {
        match variable(line) {
            Some((k, v)) => drop(vars.insert(k, v)),
            None => skipped.push(n),
        }
    }
    (vars, skipped)
}

/// `export NAME=value` or `NAME=value`, with a value in quotes or a value without a char that the
/// shell treats in a special way.
fn variable(line: &str) -> Option<(String, String)> {
    let line = line.strip_prefix("export").filter(|r| r.starts_with([' ', '\t'])).map_or(line, str::trim_start);
    let (name, value) = line.split_once('=')?;
    let first = name.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_');
    (first && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')).then_some(())?;
    Some((name.to_string(), unquote(value)?))
}

fn unquote(value: &str) -> Option<String> {
    for q in ['\'', '"'] {
        if let Some(inner) = value.strip_prefix(q) {
            return inner.strip_suffix(q).filter(|i| !i.contains(q)).map(str::to_string);
        }
    }
    let plain = !value.contains(|c: char| c.is_whitespace() || "'\"`;|&()<>\\".contains(c));
    plain.then(|| value.to_string())
}

/// The profile of the agents: the one of `--profile`, else the one that `[profile_dirs]` maps to
/// the nearest folder that holds `main` (PROFILE-1, PROFILE-2, PROFILE-3).
pub fn select_profile(cfg: &UserConfig, flag: Option<&str>, main: &Path, home: &Path) -> Result<Option<String>, String> {
    let (name, from) = match flag {
        Some(f) => (Some(f.to_string()), "--profile"),
        None => (nearest(&cfg.profile_dirs, main, home), "[profile_dirs]"),
    };
    match name {
        Some(n) if !cfg.profiles.contains_key(&n) => Err(format!("no profile `{n}` (from {from}) in {}: add [profiles.{n}]", config_path().display())),
        n => Ok(n),
    }
}

/// The profile of the nearest folder of `dirs` that holds `main`.
fn nearest(dirs: &BTreeMap<String, String>, main: &Path, home: &Path) -> Option<String> {
    let holds = |(dir, p): (&String, &String)| {
        let dir = expand(dir, home);
        let dir = std::fs::canonicalize(&dir).unwrap_or(dir);
        main.starts_with(&dir).then(|| (dir.components().count(), p.clone()))
    };
    dirs.iter().filter_map(holds).max_by_key(|(depth, _)| *depth).map(|(_, p)| p)
}

/// `~` and `~/…` in the home folder; any other text as it is.
fn expand(text: &str, home: &Path) -> PathBuf {
    match text.strip_prefix('~') {
        Some("") => home.to_path_buf(),
        Some(rest) if rest.starts_with('/') => home.join(&rest[1..]),
        _ => PathBuf::from(text),
    }
}

#[cfg(test)]
#[path = "agent_env_tests.rs"]
mod tests;
