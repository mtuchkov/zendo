//! Where zendo keeps its files, and the move from the old folders.
use super::*;

pub(super) fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
}

/// The XDG folder in `value`, or `home/default` when it is not set, is empty or is relative: an
/// empty or relative value would name a folder in the current one (CMD-21).
pub(super) fn xdg_dir(value: Option<std::ffi::OsString>, home: &Path, default: &str) -> PathBuf {
    value.map(PathBuf::from).filter(|d| d.is_absolute()).unwrap_or_else(|| home.join(default))
}

/// The XDG folder of the variable `var` of the environment (CMD-21).
pub(crate) fn xdg(var: &str, default: &str) -> PathBuf {
    xdg_dir(std::env::var_os(var), &home(), default)
}

/// `~/.zendo`: the config of the user.
pub fn config_dir() -> PathBuf {
    home().join(".zendo")
}

/// Before 0.11.1 the config was `~/.config/zendo/config.json`.
pub(super) fn legacy_config_dir() -> PathBuf {
    xdg("XDG_CONFIG_HOME", ".config").join("zendo")
}

pub fn data_dir() -> PathBuf {
    xdg("XDG_DATA_HOME", ".local/share").join("zendo")
}

/// Cargo's build folder for installs and upgrades, kept so that an upgrade builds only zendo.
pub fn build_dir() -> PathBuf {
    xdg("XDG_CACHE_HOME", ".cache").join("zendo/target")
}

/// The JSON config of before 0.11.1 becomes `~/.zendo/config.toml`. Returns the move, if it
/// happened.
pub fn old_config_to_toml() -> Option<(PathBuf, PathBuf)> {
    json_config_to_toml(&legacy_config_dir(), &config_path())
}

/// `old_config_to_toml` for the old config folder `legacy` and the config file `config`.
pub(super) fn json_config_to_toml(legacy: &Path, config: &Path) -> Option<(PathBuf, PathBuf)> {
    let json = legacy.join("config.json");
    if !json.is_file() || config.exists() {
        return None;
    }
    let cfg: UserConfig = serde_json::from_str(&std::fs::read_to_string(&json).ok()?).ok()?;
    cfg.save_to(config).ok()?;
    let _ = std::fs::remove_file(&json);
    let _ = std::fs::remove_dir(legacy);
    Some((json, config.to_path_buf()))
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

#[cfg(test)]
#[path = "paths_tests.rs"]
mod tests;
