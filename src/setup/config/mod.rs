//! User config in `~/.zendo/config.toml`, written by `zendo setup`; the user can edit it.

mod keys_write;
mod paths;
mod profile;
mod read;
mod write;

pub use keys_write::write_keys;
pub use paths::*;
pub use profile::*;
pub use read::Problem;

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
    /// The terminal bell when the turn of an agent session or the command of a terminal ends;
    /// default on. The sound button of zendo changes it (docs/zendo/attention.md).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sound: Option<bool>,
    /// The lofi music at the start; default on. Without it, zendo asks once at the start; the
    /// music button changes it (docs/zendo/lofi.md, LOFI-11, LOFI-12).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub music: Option<bool>,
    /// The usage counts in the data folder; default on (docs/zendo/usage.md, USAGE-14).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<bool>,
    /// The model that names the agent sessions, for each agent: `claude`, `codex` or `other`; an
    /// empty name: the default model of the agent (docs/zendo/session-names/session-names.md, NAMES-12).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub name_models: BTreeMap<String, String>,
    /// The model of the key hints, for each agent: `claude`, `codex` or `other`; by default `haiku`
    /// and `luna` (docs/zendo/panes/key-hints.md, REKEY-6).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub hint_models: BTreeMap<String, String>,
    /// The tip menu at the start; default on. `Do not show tips at start` turns it off (TIPS-4,
    /// TIPS-7).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tips: Option<bool>,
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
        let Some(path) = self.keymap_path() else { return Ok(None) };
        let text = std::fs::read_to_string(&path).map_err(|e| format!("keymap {}: {e}", path.display()))?;
        toml::from_str(&text).map(Some).map_err(|e| format!("keymap {}: {}", path.display(), e.to_string().lines().next().unwrap_or("")))
    }

    /// The path of the keymap file that the config names; `~/` is the home folder.
    pub fn keymap_path(&self) -> Option<PathBuf> {
        let path = self.keymap.as_deref()?;
        Some(path.strip_prefix("~/").map_or_else(|| PathBuf::from(path), |rest| home().join(rest)))
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
    /// The model that names the agent sessions of an agent of `kind`: the one of `[name_models]`,
    /// else `haiku` for Claude and `luna` for Codex (NAMES-12, NAMES-13).
    pub fn name_model(&self, kind: Kind) -> Option<String> {
        model_of(&self.name_models, kind)
    }

    /// The model of the key hints of an agent of `kind`: the one of `[hint_models]`, else `haiku`
    /// for Claude and `luna` for Codex (REKEY-6).
    pub fn hint_model(&self, kind: Kind) -> Option<String> {
        model_of(&self.hint_models, kind)
    }

    pub fn load() -> Self {
        Self::load_from(&config_path())
    }

    /// The config in `path`: every setting without an error; the defaults when there is no file
    /// (CMD-17).
    pub fn load_from(path: &Path) -> Self {
        Self::read_from(path).0
    }

    /// The config in `path`, and the lines with an error (CMD-17, CMD-18). A file that zendo cannot
    /// read is one problem without a line.
    pub fn read_from(path: &Path) -> (Self, Vec<Problem>) {
        match std::fs::read(path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (Self::default(), Vec::new()),
            Err(e) => (Self::default(), vec![Problem { line: None, error: e.to_string() }]),
            Ok(bytes) => read::read_text(&String::from_utf8_lossy(&bytes)),
        }
    }

    /// A copy, to compare with after a change (`save_changes`).
    pub fn copy(&self) -> Self {
        write::copy(self)
    }

    /// Change settings in `path` with `change` and save them: only their lines change (CMD-19).
    pub fn update(path: &Path, change: impl FnOnce(&mut Self)) -> Result<(), String> {
        let mut cfg = Self::load_from(path);
        let before = cfg.copy();
        change(&mut cfg);
        cfg.save_changes(path, &before)
    }

    /// Save in `path` the settings that differ from `before`: only their lines change, every other
    /// line stays as it is (CMD-19). Without a file, a new one.
    pub fn save_changes(&self, path: &Path, before: &Self) -> Result<(), String> {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => CONFIG_HEAD.as_bytes().to_vec(),
            Err(e) => return Err(format!("cannot read {}: {e}", path.display())),
        };
        let write = || -> std::io::Result<()> {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            crate::atomic::write(path, write::edit_bytes(&bytes, before, self))
        };
        write().map_err(|e| format!("cannot write {}: {e}", path.display()))
    }

    /// Write the whole file: only for a new file. To change a file of the user, use `update` or
    /// `save_changes` (CMD-19).
    pub fn save_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let text = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        crate::atomic::write(path, format!("{CONFIG_HEAD}{text}"))
    }
}

/// The model of `models` for an agent of `kind`, else `haiku` for Claude and `luna` for Codex.
fn model_of(models: &BTreeMap<String, String>, kind: Kind) -> Option<String> {
    let (id, default) = match kind {
        Kind::Claude => ("claude", Some("haiku")),
        Kind::Codex => ("codex", Some("luna")),
        Kind::Other => ("other", None),
    };
    let name = models.get(id).map(String::as_str).or(default)?;
    (!name.trim().is_empty()).then(|| name.trim().to_string())
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
            Source::Flag => "--agent-cmd",
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
