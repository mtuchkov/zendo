//! The keymap: for each pane, the actions and their keys. The default keys are in
//! resources/keymap-linux.toml or resources/keymap-mac.toml; a keymap file of the user (`keymap = "~/.zendo/keymap.toml"` in the config)
//! changes them, and `[keys.<pane>]` in `~/.zendo/config.toml` changes single keys after the file:
//!
//! ```toml
//! [keys.read]
//! add_to_chat = ["+", "m"]
//! select_down = "J"
//! ```
//!
//! A key is a name with modifiers: `j`, `J`, `+`, `ctrl-b`, `alt-i`, `shift-down`, `ctrl-shift-left`,
//! `enter`, `esc`, `tab`, `space`, `backspace`, `delete`, `up`, `down`, `left`, `right`, `home`, `end`,
//! `pageup`, `pagedown`, `f1`…`f12`. The keys of a pane come first, then the global keys. While
//! text is selected where the user writes, the keys of `[selection]` come before both.

mod actions;
mod free;
mod legacy;
mod key;

pub use actions::*;
pub use free::*;
pub use key::*;

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Pane {
    Global,
    #[default]
    Read,
    Write,
    Chat,
    /// Keys while text is selected where the user writes: the note in write mode, the chat input.
    /// They come before the keys of the pane (PANES-37).
    Selected,
    Transcript,
    Tree,
    Delta,
    Notes,
    /// The session view: a Claude Code session on the whole screen.
    Session,
    /// The search box, the find panel and the file name filter of the tree (PANES-48).
    List,
    /// The theme list (PANES-48).
    Theme,
    /// The question of a file: a name, a folder, or yes for a delete (PANES-48).
    Question,
}

impl Pane {
    pub const ALL: [Pane; 13] = [Pane::Global, Pane::Read, Pane::Write, Pane::Chat, Pane::Selected, Pane::Transcript, Pane::Tree, Pane::Delta, Pane::Notes, Pane::Session, Pane::List, Pane::Theme, Pane::Question];

    pub fn name(self) -> &'static str {
        match self {
            Pane::Global => "global",
            Pane::Read => "read",
            Pane::Write => "write",
            Pane::Chat => "chat",
            Pane::Selected => "selection",
            Pane::Transcript => "transcript",
            Pane::Tree => "tree",
            Pane::Delta => "delta",
            Pane::Notes => "notes",
            Pane::Session => "session",
            Pane::List => "list",
            Pane::Theme => "theme",
            Pane::Question => "question",
        }
    }

    pub fn from_name(s: &str) -> Option<Pane> {
        Pane::ALL.into_iter().find(|p| p.name() == s)
    }
}

/// A group of actions in the help and in the keymap file (PANES-30).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Group {
    Navigation,
    Selection,
    Editing,
    Features,
    Sessions,
    View,
    App,
}

impl Group {
    pub const ALL: [Group; 7] = [Group::Navigation, Group::Selection, Group::Editing, Group::Features, Group::Sessions, Group::View, Group::App];

    pub fn name(self) -> &'static str {
        match self {
            Group::Navigation => "navigation",
            Group::Selection => "text selection",
            Group::Editing => "editing",
            Group::Features => "features",
            Group::Sessions => "sessions and terminals",
            Group::View => "panes and view",
            Group::App => "zendo",
        }
    }
}

/// An action of a pane, its group, and what the help says about it.
pub struct Action {
    pub pane: Pane,
    pub group: Group,
    pub name: &'static str,
    pub help: &'static str,
}

/// The default keys on Linux.
pub const LINUX_KEYMAP: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/keymap-linux.toml"));
/// The default keys on the Mac: alt-} for the previous session, because Apple Terminal sends alt-[
/// as ESC [, the start of a CSI sequence (TERM-49).
pub const MAC_KEYMAP: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/keymap-mac.toml"));
/// The default keys of this system, as zendo ships them: `zendo keymap` prints the file.
pub const DEFAULT_KEYMAP: &str = if cfg!(target_os = "macos") { MAC_KEYMAP } else { LINUX_KEYMAP };

/// The default keys, read once.
fn default_keys() -> &'static KeyConfig {
    static KEYS: std::sync::OnceLock<KeyConfig> = std::sync::OnceLock::new();
    KEYS.get_or_init(|| toml::from_str(DEFAULT_KEYMAP).expect("the default keymap parses"))
}


/// One key or a list of keys in the config.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KeySpec {
    One(String),
    Many(Vec<String>),
}

impl KeySpec {
    fn list(&self) -> Vec<&str> {
        match self {
            KeySpec::One(s) => vec![s.as_str()],
            KeySpec::Many(v) => v.iter().map(String::as_str).collect(),
        }
    }
}

/// `[keys.<pane>] <action> = "key"` or `["key", "key"]`.
pub type KeyConfig = BTreeMap<String, BTreeMap<String, KeySpec>>;

#[derive(Clone, Debug, Default)]
pub struct Keymap {
    by_key: HashMap<(Pane, Key), &'static str>,
    keys: HashMap<(Pane, &'static str), Vec<Key>>,
    /// The terminal has no kitty keyboard protocol: ctrl-shift-k comes as ctrl-k.
    pub legacy: bool,
}

impl Keymap {
    /// The keys in three layers: the default keymap, the keymap file of the user, and `[keys]` of the
    /// config; a later layer gives an action other keys. Also the problems of the user's layers:
    /// unknown panes, actions and keys. The keys of the layer before stay for those.
    pub fn new(file: Option<&KeyConfig>, cfg: &KeyConfig) -> (Keymap, Vec<String>) {
        let layers: Vec<(&str, &KeyConfig)> = file.map(|f| ("keymap", f)).into_iter().chain([("keys", cfg)]).collect();
        let mut problems: Vec<String> = layers.iter().flat_map(|(name, l)| unknown(name, l)).collect();
        let mut map = Keymap::default();
        let mut custom = std::collections::HashSet::new();
        for act in ACTIONS {
            let mut keys = parse_keys(default_keys(), act, "default", &mut Vec::new()).unwrap_or_default();
            for (name, l) in &layers {
                if let Some(mine) = parse_keys(l, act, name, &mut problems) {
                    keys = mine;
                    custom.insert((act.pane, act.name));
                }
            }
            map.keys.insert((act.pane, act.name), keys);
        }
        map.by_key = owners(&map, &custom, &mut problems);
        (map, problems)
    }

    pub fn defaults() -> Keymap {
        Keymap::new(None, &KeyConfig::new()).0
    }

    /// The action of this key in this pane; else the global action.
    pub fn action(&self, pane: Pane, k: &KeyEvent) -> Option<&'static str> {
        let key = Key::from_event(k);
        let exact = self.by_key.get(&(pane, key)).or_else(|| self.by_key.get(&(Pane::Global, key))).copied();
        exact.or_else(|| self.shifted(pane, key)).or_else(|| self.shifted(Pane::Global, key))
    }

    /// The action of this key in this pane only.
    pub fn local(&self, pane: Pane, k: &KeyEvent) -> Option<&'static str> {
        let key = Key::from_event(k);
        self.by_key.get(&(pane, key)).copied().or_else(|| self.shifted(pane, key))
    }

    /// The keys of an action, for the help: `j ↓`.
    pub fn label(&self, pane: Pane, name: &str) -> String {
        self.keys.iter().find(|((p, n), _)| *p == pane && *n == name).map(|(_, ks)| ks.iter().map(Key::label).collect::<Vec<_>>().join(" ")).unwrap_or_default()
    }

    /// The first key of an action, for a hint or a message; `None` when the action has no key
    /// (PANES-28, PANES-29).
    pub fn key(&self, pane: Pane, name: &str) -> Option<String> {
        Some(self.first(pane, name)).filter(|k| !k.is_empty())
    }

    /// A hint: the first key of an action and `text`; empty when the action has no key (PANES-29).
    pub fn hint(&self, pane: Pane, name: &str, text: &str) -> String {
        self.key(pane, name).map(|k| format!("{k}{text}")).unwrap_or_default()
    }

    /// The action has the first key that zendo ships for it.
    pub fn has_default_key(&self, pane: Pane, name: &str) -> bool {
        static DEFAULTS: std::sync::OnceLock<Keymap> = std::sync::OnceLock::new();
        self.first(pane, name) == DEFAULTS.get_or_init(Keymap::defaults).first(pane, name)
    }

    /// The action of a key in one pane, without the global keys and the legacy keys.
    pub fn owner(&self, pane: Pane, key: Key) -> Option<&'static str> {
        self.by_key.get(&(pane, key)).copied()
    }

    /// All keys of an action, the first first.
    pub fn keys_of(&self, pane: Pane, name: &str) -> Vec<Key> {
        self.keys.iter().find(|((p, n), _)| *p == pane && *n == name).map(|(_, ks)| ks.clone()).unwrap_or_default()
    }

    /// The first key of an action, for the status bar.
    pub fn first(&self, pane: Pane, name: &str) -> String {
        self.keys.iter().find(|((p, n), _)| *p == pane && *n == name).and_then(|(_, ks)| ks.first()).map(Key::label).unwrap_or_default()
    }
}

/// The panes and actions of a layer that zendo does not know.
fn unknown(layer: &str, cfg: &KeyConfig) -> Vec<String> {
    cfg.iter()
        .flat_map(|(pane, actions)| match Pane::from_name(pane) {
            None => vec![format!("[{layer}.{pane}]: no such pane (panes: {})", Pane::ALL.map(Pane::name).join(", "))],
            Some(p) => actions.keys().filter(|n| !ACTIONS.iter().any(|a| a.pane == p && a.name == *n)).map(|n| format!("[{layer}.{pane}] {n}: no such action")).collect(),
        })
        .collect()
}

/// The keys that a layer gives an action: None when it gives none, or none that parse; an empty
/// list is no key (REKEY-29).
fn parse_keys(cfg: &KeyConfig, act: &Action, layer: &str, problems: &mut Vec<String>) -> Option<Vec<Key>> {
    let list = cfg.get(act.pane.name()).and_then(|m| m.get(act.name))?.list();
    let keys: Vec<Key> = list
        .iter()
        .filter_map(|s| Key::parse(s).map_err(|e| problems.push(format!("[{layer}.{}] {}: {e}", act.pane.name(), act.name))).ok())
        .collect();
    (list.is_empty() || !keys.is_empty()).then_some(keys)
}

/// Which action a key does in each pane. A key goes to the first action that has it; a key that the
/// user set goes to the user's action. Two actions of one layer with one key are a problem.
fn owners(map: &Keymap, custom: &std::collections::HashSet<(Pane, &'static str)>, problems: &mut Vec<String>) -> HashMap<(Pane, Key), &'static str> {
    let mut owner: HashMap<(Pane, Key), (&'static str, bool)> = HashMap::new();
    for act in ACTIONS {
        let mine = custom.contains(&(act.pane, act.name));
        for k in &map.keys[&(act.pane, act.name)] {
            match owner.get(&(act.pane, *k)) {
                Some((other, other_mine)) if *other_mine || !mine => {
                    if mine == *other_mine {
                        problems.push(format!("[{}] {}: {} is also the key of {other}", act.pane.name(), act.name, k.label()));
                    }
                }
                _ => {
                    owner.insert((act.pane, *k), (act.name, mine));
                }
            }
        }
    }
    owner.into_iter().map(|(k, (n, _))| (k, n)).collect()
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
