//! User config in `~/.zendo/config.toml`, written by `zendo setup`; the user can edit it.

mod paths;
mod profile;

pub use paths::*;
pub use profile::*;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const ADAPTER: &str = CLAUDE.package;
pub const ADAPTER_VERSION: &str = CLAUDE.version;

pub const PROFILES: [&Profile; 2] = [&CLAUDE, &CODEX];

#[derive(Default, Serialize, Deserialize)]
pub struct UserConfig {
    /// ACP agent command. Empty means: the installed adapter, else npx.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Extra environment for the agent and for npm, e.g. NODE_OPTIONS.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    /// A keymap file of the user, as `~/.zendo/keymap.toml`: its keys go in place of the defaults.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keymap: Option<String>,
    /// Other keys for the actions of a pane: `[keys.read] add_to_chat = ["+", "m"]`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub keys: crate::keys::KeyConfig,
    /// Code highlighting: the languages that are on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub highlight: Option<crate::text::highlight::Config>,
    /// The width of the text column, in chars: read mode wraps at it, and write mode breaks a
    /// line that is longer. Default 76; `alt-=` and `alt--` change it for a repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u16>,
    /// The private notes repo, when the user turned it on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<NotesRepo>,
}

/// Notes in a private git repo, one folder for each machine and repo root.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotesRepo {
    /// The local clone of the notes repo.
    pub path: PathBuf,
    /// The name of this machine in the notes repo.
    pub machine: String,
    /// Commit and push every this many seconds.
    #[serde(default = "default_sync_seconds")]
    pub sync_seconds: u64,
}

fn default_sync_seconds() -> u64 {
    crate::setup::sync::DEFAULT_SECONDS
}

impl NotesRepo {
    /// The notes folder of `root` on this machine.
    pub fn location(&self, root: &Path) -> PathBuf {
        let home = std::env::var_os("HOME").filter(|h| !h.is_empty()).map(PathBuf::from);
        crate::setup::sync::location_dir(&self.path, &self.machine, root, home.as_deref())
    }

    /// Make the folder and write its location file: machine, path, git origin.
    pub fn prepare(&self, root: &Path) -> std::io::Result<PathBuf> {
        let dir = self.location(root);
        let loc = crate::setup::sync::Location { machine: self.machine.clone(), path: root.display().to_string(), origin: crate::setup::sync::origin_of(root) };
        crate::setup::sync::prepare(&dir, &loc)?;
        Ok(dir)
    }
}

/// The old name of zendo. Its folders move to the new names once, on the first run.
const OLD_NAME: &str = "dawn";

const CONFIG_HEAD: &str = "# zendo config. `zendo setup` writes it, and you can edit it.\n\n";

impl UserConfig {
    /// The keymap file of the user, when the config names one; `~/` is the home folder.
    pub fn keymap_file(&self) -> Result<Option<crate::keys::KeyConfig>, String> {
        let Some(path) = self.keymap.as_deref() else { return Ok(None) };
        let path = match path.strip_prefix("~/") {
            Some(rest) => home().join(rest),
            None => PathBuf::from(path),
        };
        let text = std::fs::read_to_string(&path).map_err(|e| format!("keymap {}: {e}", path.display()))?;
        toml::from_str(&text).map(Some).map_err(|e| format!("keymap {}: {}", path.display(), e.to_string().lines().next().unwrap_or("")))
    }

    /// The keymap of the user: the keymap file, then `[keys]`. Also what is wrong in them.
    pub fn keymap(&self) -> (crate::keys::Keymap, Vec<String>) {
        let (file, read_problem) = match self.keymap_file() {
            Ok(f) => (f, None),
            Err(e) => (None, Some(e)),
        };
        let (map, mut problems) = crate::keys::Keymap::new(file.as_ref(), &self.keys);
        problems.extend(read_problem);
        (map, problems)
    }
    pub fn load() -> Self {
        Self::read().ok().flatten().unwrap_or_default()
    }

    /// The config, `None` when there is no file, or what is wrong with the file.
    pub fn read() -> Result<Option<Self>, String> {
        match std::fs::read_to_string(config_path()) {
            Err(_) => Ok(None),
            Ok(s) => toml::from_str(&s).map(Some).map_err(|e| e.to_string()),
        }
    }

    pub fn save(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(config_dir())?;
        let text = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(config_path(), format!("{CONFIG_HEAD}{text}"))
    }
}

/// Where the agent command came from, for `zendo doctor`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Flag,
    Env,
    Config,
    Installed,
    Npx,
}

impl Source {
    pub fn label(self) -> &'static str {
        match self {
            Source::Flag => "--agent",
            Source::Env => "$ZENDO_AGENT",
            Source::Config => "config",
            Source::Installed => "installed by zendo setup",
            Source::Npx => "npx (run `zendo setup` to install it once)",
        }
    }
}

/// The agent command: flag, then $ZENDO_AGENT, then config, then Claude. A profile name (`claude`,
/// `codex`) stands for the installed adapter of that agent, else npx.
pub fn resolve_agent(flag: Option<String>, cfg: &UserConfig) -> (String, Source) {
    let (value, source) = if let Some(a) = flag {
        (a, Source::Flag)
    } else if let Ok(a) = std::env::var("ZENDO_AGENT") {
        (a, Source::Env)
    } else if let Some(a) = &cfg.agent {
        (a.clone(), Source::Config)
    } else {
        return CLAUDE.command();
    };
    match Profile::by_id(&value) {
        Some(p) => p.command(),
        None => (value, source),
    }
}

/// The config env merged over the process env, NODE_OPTIONS appended rather than replaced.
pub fn agent_env(cfg: &UserConfig) -> BTreeMap<String, String> {
    let mut env = cfg.env.clone();
    if let (Some(extra), Ok(now)) = (env.get("NODE_OPTIONS").cloned(), std::env::var("NODE_OPTIONS")) {
        if !now.contains(&extra) {
            env.insert("NODE_OPTIONS".into(), format!("{now} {extra}"));
        } else {
            env.insert("NODE_OPTIONS".into(), now);
        }
    }
    env
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
