//! The write of the keys of one action, from the key float (docs/zendo/panes/key-change.md,
//! REKEY-16, REKEY-27, REKEY-29): to `[keys.<pane>]` of the config when that sets the action, else to
//! the keymap file that the config names, else to a new `keymap.toml` beside the config.
use super::UserConfig;
use crate::keys::{Key, KeySpec, Pane};
use std::path::{Path, PathBuf};

/// Write `keys` as the keys of `action` of `pane`; an empty list is no key. Only the line of the
/// action changes. Returns the file that changed.
pub fn write_keys(cfg: &Path, pane: Pane, action: &str, keys: &[Key]) -> Result<PathBuf, String> {
    let user = UserConfig::load_from(cfg);
    let specs: Vec<String> = keys.iter().map(Key::spec).collect();
    if user.keys.get(pane.name()).is_some_and(|m| m.contains_key(action)) {
        UserConfig::update(cfg, |u| drop(u.keys.entry(pane.name().into()).or_default().insert(action.into(), KeySpec::Many(specs))))?;
        return Ok(cfg.to_path_buf());
    }
    let named = user.keymap_path();
    let file = named.clone().unwrap_or_else(|| cfg.with_file_name("keymap.toml"));
    edit_file(&file, pane, action, &specs)?;
    if named.is_none() {
        UserConfig::update(cfg, |u| u.keymap = Some(tilde(&file)))?;
    }
    Ok(file)
}

/// The keymap file with the new keys of one action: the comments and the other lines stay.
fn edit_file(file: &Path, pane: Pane, action: &str, specs: &[String]) -> Result<(), String> {
    let text = match std::fs::read_to_string(file) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(format!("cannot read {}: {e}", file.display())),
    };
    let mut doc: toml_edit::DocumentMut = text.parse().map_err(|e| format!("{}: {}", file.display(), first_line(&e)))?;
    let table = doc.entry(pane.name()).or_insert_with(toml_edit::table).as_table_mut().ok_or_else(|| format!("{}: [{}] is not a table", file.display(), pane.name()))?;
    let new: toml_edit::Value = specs.iter().map(String::as_str).collect::<toml_edit::Array>().into();
    match table.get_mut(action).and_then(toml_edit::Item::as_value_mut) {
        Some(old) => {
            let decor = old.decor().clone();
            *old = new;
            *old.decor_mut() = decor;
        }
        None => drop(table.insert(action, toml_edit::value(new))),
    }
    crate::atomic::write(file, doc.to_string()).map_err(|e| format!("cannot write {}: {e}", file.display()))
}

fn first_line(e: &impl std::fmt::Display) -> String {
    e.to_string().lines().next().unwrap_or_default().to_string()
}

/// The path as the config names it: `~/` for the home folder.
fn tilde(path: &Path) -> String {
    match path.strip_prefix(super::paths::home()) {
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}

#[cfg(test)]
#[path = "keys_write_tests.rs"]
mod tests;
