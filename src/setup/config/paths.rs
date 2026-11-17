//! Where zendo keeps its files, and the move from the old folders.
use super::*;

pub(super) fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
}

/// `~/.zendo`: the config of the user.
pub fn config_dir() -> PathBuf {
    home().join(".zendo")
}

/// Before 0.11.1 the config was `~/.config/zendo/config.json`.
pub(super) fn legacy_config_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".config")).join("zendo")
}

pub fn data_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".local/share")).join("zendo")
}

/// Cargo's build folder for installs and upgrades, kept so that an upgrade builds only zendo.
pub fn build_dir() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME").map(PathBuf::from).unwrap_or_else(|| home().join(".cache")).join("zendo/target")
}

/// The folders and the config file of zendo of one user.
pub(super) struct Dirs {
    pub legacy_config: PathBuf,
    pub data: PathBuf,
    pub cache: PathBuf,
    pub config: PathBuf,
}

impl Dirs {
    /// The folders of this user, from the environment.
    fn of_user() -> Self {
        let cache = build_dir().parent().map(Path::to_path_buf).unwrap_or_default();
        Self { legacy_config: legacy_config_dir(), data: data_dir(), cache, config: config_path() }
    }
}

/// Move the folders of the old name to the new ones: config, data, build cache, and `.dawn` in the
/// repository. A folder moves only when the new one does not exist. Returns what moved.
pub fn move_old_folders(repo: &Path) -> Vec<(PathBuf, PathBuf)> {
    move_old_folders_in(repo, &Dirs::of_user())
}

/// `move_old_folders` for the folders `dirs`.
pub(super) fn move_old_folders_in(repo: &Path, dirs: &Dirs) -> Vec<(PathBuf, PathBuf)> {
    let old_of = |new: &Path| new.with_file_name(OLD_NAME);
    let pairs = [
        (old_of(&dirs.legacy_config), dirs.legacy_config.clone()),
        (old_of(&dirs.data), dirs.data.clone()),
        (old_of(&dirs.cache), dirs.cache.clone()),
        (repo.join(format!(".{OLD_NAME}")), crate::setup::state::dir(repo)),
    ];
    let mut moved: Vec<_> = pairs.into_iter().filter(|(old, new)| old.is_dir() && !new.exists() && std::fs::rename(old, new).is_ok()).collect();
    // The saved agent command can name the adapter in the old data folder.
    let json = dirs.legacy_config.join("config.json");
    let (old, new) = (old_of(&dirs.data).display().to_string(), dirs.data.display().to_string());
    for file in [&json, &dirs.config] {
        let text = std::fs::read_to_string(file).unwrap_or_default();
        if text.contains(&old) {
            let _ = std::fs::write(file, text.replace(&old, &new));
        }
    }
    moved.extend(json_config_to_toml(&json, dirs));
    moved
}

/// The JSON config of before 0.11.1 becomes `~/.zendo/config.toml`. Returns the move, if it happened.
fn json_config_to_toml(json: &Path, dirs: &Dirs) -> Option<(PathBuf, PathBuf)> {
    if !json.is_file() || dirs.config.exists() {
        return None;
    }
    let cfg: UserConfig = serde_json::from_str(&std::fs::read_to_string(json).ok()?).ok()?;
    cfg.save_to(&dirs.config).ok()?;
    let _ = std::fs::remove_file(json);
    let _ = std::fs::remove_dir(&dirs.legacy_config);
    Some((json.to_path_buf(), dirs.config.clone()))
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

/// Where `zendo setup` installs the adapter.
pub fn adapter_dir() -> PathBuf {
    data_dir().join("agent")
}

pub fn adapter_entry() -> PathBuf {
    adapter_dir().join("node_modules").join(ADAPTER).join("dist/index.js")
}

pub fn adapter_installed_version() -> Option<String> {
    let pkg = adapter_dir().join("node_modules").join(ADAPTER).join("package.json");
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(pkg).ok()?).ok()?;
    v["version"].as_str().map(str::to_string)
}

pub fn quote(p: &Path) -> String {
    let s = p.display().to_string();
    if s.contains(char::is_whitespace) { format!("'{s}'") } else { s }
}
