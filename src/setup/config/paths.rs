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

/// Move the folders of the old name to the new ones: config, data, build cache, and `.dawn` in the
/// repository. A folder moves only when the new one does not exist. Returns what moved.
pub fn move_old_folders(repo: &Path) -> Vec<(PathBuf, PathBuf)> {
    let cache = build_dir().parent().map(Path::to_path_buf).unwrap_or_default();
    let pairs = [
        (legacy_config_dir().with_file_name(OLD_NAME), legacy_config_dir()),
        (data_dir().with_file_name(OLD_NAME), data_dir()),
        (cache.with_file_name(OLD_NAME), cache),
        (repo.join(format!(".{OLD_NAME}")), crate::setup::state::dir(repo)),
    ];
    let mut moved = Vec::new();
    for (old, new) in pairs {
        if old.is_dir() && !new.exists() && std::fs::rename(&old, &new).is_ok() {
            moved.push((old, new));
        }
    }
    // The saved agent command can name the adapter in the old data folder.
    let json = legacy_config_dir().join("config.json");
    for file in [json.clone(), config_path()] {
        if let Ok(text) = std::fs::read_to_string(&file) {
            let (old, new) = (data_dir().with_file_name(OLD_NAME), data_dir());
            let (old, new) = (old.display().to_string(), new.display().to_string());
            if text.contains(&old) {
                let _ = std::fs::write(&file, text.replace(&old, &new));
            }
        }
    }
    // The JSON config of before 0.11.1 becomes `~/.zendo/config.toml`.
    if json.is_file() && !config_path().exists() {
        let cfg = std::fs::read_to_string(&json).ok().and_then(|s| serde_json::from_str::<UserConfig>(&s).ok());
        if let Some(cfg) = cfg
            && cfg.save().is_ok() {
                let _ = std::fs::remove_file(&json);
                let _ = std::fs::remove_dir(legacy_config_dir());
                moved.push((json, config_path()));
            }
    }
    moved
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
