//! The agent environment (docs/zendo/agent-env.md) and its profiles (docs/zendo/agent-profiles.md):
//! the variables that zendo gives to each agent that it starts, over the environment of zendo.

use super::envrc::{read_envrc, skipped_problem};
use super::{UserConfig, config_path};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The option of Node.js that makes it use the CA certificates of the system.
pub const SYSTEM_CA: &str = "--use-system-ca";
const NODE: &str = "NODE_OPTIONS";

/// The variables over the environment of zendo, the profile that gave some of them, and what zendo
/// shows once at the start (ENV-7).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AgentEnv {
    pub vars: BTreeMap<String, String>,
    pub profile: Option<String>,
    pub problems: Vec<String>,
    /// The `NODE_OPTIONS` of `.envrc` and of the profile, when zendo put the system CA into `vars`
    /// (ENV-5).
    pub node_before_ca: Option<Option<String>>,
    /// The folder is not trusted, so zendo read no `.envrc` and starts no agent in it (TRUST-1):
    /// what zendo says about it (TRUST-6).
    pub untrusted: Option<String>,
}

impl AgentEnv {
    /// The variables for the shell of a terminal: without the system CA that zendo adds for the
    /// agents, because an old Node.js of the user does not start with it (ENV-5).
    pub fn shell_vars(&self) -> BTreeMap<String, String> {
        let mut vars = self.vars.clone();
        match &self.node_before_ca {
            Some(Some(old)) => drop(vars.insert(NODE.into(), old.clone())),
            Some(None) => drop(vars.remove(NODE)),
            None => {}
        }
        vars
    }
}

/// The agent environment with the profile of the folder `at`, and the `.envrc` of the folder
/// `envrc_of`. `env` gives a variable of the environment of zendo. The product builds it through
/// `trust::run_env`, after the question about the trust.
#[cfg(test)]
pub(super) fn build(cfg: &UserConfig, at: &Path, envrc_of: Option<&Path>, flag: Option<&str>, home: &Path, env: &dyn Fn(&str) -> Option<String>) -> Result<AgentEnv, String> {
    Ok(with_profile(cfg, select_profile(cfg, flag, at, home)?, envrc_of, home, env))
}

/// The agent environment: `.envrc`, then the variables of `profile`, then the system CA.
pub(super) fn with_profile(cfg: &UserConfig, profile: Option<String>, envrc_of: Option<&Path>, home: &Path, env: &dyn Fn(&str) -> Option<String>) -> AgentEnv {
    let (mut vars, problems) = envrc_of.map(|m| envrc(&m.join(".envrc"), env, home)).unwrap_or_default();
    let set = profile.as_ref().and_then(|p| cfg.profiles.get(p)).into_iter().flatten();
    vars.extend(set.map(|(k, v)| (k.clone(), expand(v, home).display().to_string())));
    let before = vars.get(NODE).cloned();
    let added = cfg.system_ca == Some(true) && add_system_ca(&mut vars, env(NODE));
    AgentEnv { vars, profile, problems, node_before_ca: added.then_some(before), ..Default::default() }
}

/// The variables of the file `.envrc` at `path`, and what zendo says about the file (ENV-7).
fn envrc(path: &Path, env: &dyn Fn(&str) -> Option<String>, home: &Path) -> (BTreeMap<String, String>, Vec<String>) {
    match std::fs::read_to_string(path) {
        Ok(text) => {
            // `$PWD` is the folder of the file, as when a shell runs the file there (ENV-8).
            let folder = path.parent().map(|p| p.display().to_string());
            let (vars, skipped) = read_envrc(&text, &|k| if k == "PWD" { folder.clone() } else { env(k) }, home);
            (vars, skipped_problem(path, &skipped).into_iter().collect())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Default::default(),
        Err(e) => (BTreeMap::new(), vec![format!("cannot read {}: {e}", path.display())]),
    }
}

/// Put the option of the system CA into the `NODE_OPTIONS` of `vars`, after the options that are
/// there, or after `inherited`, the options of zendo itself. False when the option is there
/// already (ENV-5).
pub fn add_system_ca(vars: &mut BTreeMap<String, String>, inherited: Option<String>) -> bool {
    let now = vars.get(NODE).cloned().or(inherited).unwrap_or_default();
    let missing = !now.split_whitespace().any(|o| o == SYSTEM_CA);
    if missing {
        vars.insert(NODE.into(), format!("{now} {SYSTEM_CA}").trim().to_string());
    }
    missing
}

/// The root of the main checkout of the repository that holds `dir`; None outside a repository.
/// zendo reads the files of git itself, so the answer needs no program (ENV-3). A checkout whose
/// repository has no main checkout, as a submodule and a worktree of a bare repository, counts as
/// its own main checkout.
pub fn main_checkout(dir: &Path) -> Option<PathBuf> {
    checkout(dir).map(|(_, main)| main)
}

/// The root of the checkout that holds `dir`, and the root of its main checkout.
pub(super) fn checkout(dir: &Path) -> Option<(PathBuf, PathBuf)> {
    let root = dir.ancestors().find(|p| p.join(".git").exists())?;
    let root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let main = common_dir(&root).filter(|c| c.file_name().is_some_and(|n| n == ".git")).and_then(|c| c.parent().map(Path::to_path_buf));
    let main = main.unwrap_or_else(|| root.clone());
    Some((root, main))
}

/// The folder of git that the worktree at `root` shares with the main checkout: its `.git` file
/// names the folder of the worktree, and the file `commondir` there names the common folder.
fn common_dir(root: &Path) -> Option<PathBuf> {
    let text = std::fs::read_to_string(root.join(".git")).ok()?;
    let own = root.join(text.lines().find_map(|l| l.strip_prefix("gitdir:"))?.trim());
    let common = std::fs::read_to_string(own.join("commondir")).ok()?;
    std::fs::canonicalize(own.join(common.trim())).ok()
}

/// The profile of the agents: the one of `--profile`, else the one that `[profile_dirs]` maps to
/// the nearest folder that holds `at` (PROFILE-1, PROFILE-2, PROFILE-3).
pub fn select_profile(cfg: &UserConfig, flag: Option<&str>, at: &Path, home: &Path) -> Result<Option<String>, String> {
    let (name, from) = match flag {
        Some(f) => (Some(f.to_string()), "--profile"),
        None => (nearest(&cfg.profile_dirs, at, home), "[profile_dirs]"),
    };
    match name {
        Some(n) if !cfg.profiles.contains_key(&n) => Err(format!("no profile `{n}` (from {from}) in {}: add [profiles.{n}]", config_path().display())),
        n => Ok(n),
    }
}

/// The profile of the nearest folder of `dirs` that holds `at`.
fn nearest(dirs: &BTreeMap<String, String>, at: &Path, home: &Path) -> Option<String> {
    let holds = |(dir, p): (&String, &String)| {
        let dir = expand(dir, home);
        let dir = std::fs::canonicalize(&dir).unwrap_or(dir);
        at.starts_with(&dir).then(|| (dir.components().count(), p.clone()))
    };
    dirs.iter().filter_map(holds).max_by_key(|(depth, _)| *depth).map(|(_, p)| p)
}

/// `~` and `~/…` in the home folder; any other text as it is.
pub(super) fn expand(text: &str, home: &Path) -> PathBuf {
    match text.strip_prefix('~') {
        Some("") => home.to_path_buf(),
        Some(rest) if rest.starts_with('/') => home.join(&rest[1..]),
        _ => PathBuf::from(text),
    }
}

#[cfg(test)]
#[path = "agent_env_tests.rs"]
mod tests;
