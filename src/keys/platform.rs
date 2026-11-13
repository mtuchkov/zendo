//! The default keys of the platform. On macOS, the keys of resources/keymap-macos.toml come over the
//! default keymap (PANES-28), and `zendo keymap` prints the two as one file (PANES-29).
use super::*;
use toml_edit::{DocumentMut, Item};

/// The keys of macOS that differ from the default keys.
pub const MACOS_KEYMAP: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/keymap-macos.toml"));

/// This build runs on macOS: the installer builds zendo on the machine where it runs.
pub const MACOS: bool = cfg!(target_os = "macos");

const MACOS_HEADER: &str = "# macOS: the keys of resources/keymap-macos.toml are in the lines that end with macOS.\n";

/// The default keymap file of the platform: on macOS, each action of the macOS keymap gets its keys,
/// and its line keeps the help of the action, marked `(macOS)`.
pub fn platform_keymap(mac: bool) -> String {
    if !mac {
        return DEFAULT_KEYMAP.to_string();
    }
    let mut doc: DocumentMut = DEFAULT_KEYMAP.parse().expect("resources/keymap.toml parses");
    let overlay: DocumentMut = MACOS_KEYMAP.parse().expect("resources/keymap-macos.toml parses");
    let actions = overlay.iter().filter_map(|(pane, item)| item.as_table().map(|t| (pane, t))).flat_map(|(pane, t)| t.iter().map(move |(name, keys)| (pane, name, keys)));
    for (pane, name, keys) in actions {
        set_keys(&mut doc[pane][name], keys);
    }
    format!("{MACOS_HEADER}{doc}")
}

/// Put the keys of the macOS keymap into the line of the default keymap, and mark the line.
fn set_keys(line: &mut Item, keys: &Item) {
    let Some(keys) = keys.as_value() else { return };
    let help = line.as_value().and_then(|v| v.decor().suffix()).and_then(|s| s.as_str()).unwrap_or("").trim_end();
    let mark = if help.is_empty() { "  # (macOS)".to_string() } else { format!("{help} (macOS)") };
    let mut keys = keys.clone();
    keys.decor_mut().set_prefix(" ");
    keys.decor_mut().set_suffix(mark);
    *line = Item::Value(keys);
}

/// The default keys of the platform, read once.
pub(super) fn default_keys(mac: bool) -> &'static KeyConfig {
    static DEFAULT: std::sync::OnceLock<KeyConfig> = std::sync::OnceLock::new();
    static MAC: std::sync::OnceLock<KeyConfig> = std::sync::OnceLock::new();
    let parse = move || toml::from_str(&platform_keymap(mac)).expect("the default keymap of the platform parses");
    if mac { MAC.get_or_init(parse) } else { DEFAULT.get_or_init(parse) }
}
