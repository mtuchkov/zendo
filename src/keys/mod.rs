//! The keymap: for each pane, a table of actions with their default keys.
//!
//! `[keys.<pane>]` in `~/.zendo/config.toml` gives an action other keys:
//!
//! ```toml
//! [keys.read]
//! add_to_chat = ["+", "ctrl-s"]
//! select_down = "J"
//! ```
//!
//! A key is a name with modifiers: `j`, `J`, `+`, `ctrl-b`, `alt-i`, `shift-down`, `ctrl-shift-left`,
//! `enter`, `esc`, `tab`, `space`, `backspace`, `delete`, `up`, `down`, `left`, `right`, `home`, `end`,
//! `pageup`, `pagedown`, `f1`…`f12`. The keys of a pane come first, then the global keys.

mod actions;
mod key;

pub use actions::*;
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
    Tree,
    Delta,
    Notes,
}

impl Pane {
    pub const ALL: [Pane; 7] = [Pane::Global, Pane::Read, Pane::Write, Pane::Chat, Pane::Tree, Pane::Delta, Pane::Notes];

    pub fn name(self) -> &'static str {
        match self {
            Pane::Global => "global",
            Pane::Read => "read",
            Pane::Write => "write",
            Pane::Chat => "chat",
            Pane::Tree => "tree",
            Pane::Delta => "delta",
            Pane::Notes => "notes",
        }
    }

    pub fn from_name(s: &str) -> Option<Pane> {
        Pane::ALL.into_iter().find(|p| p.name() == s)
    }
}

/// An action of a pane, its default keys, and what the help says about it.
pub struct Action {
    pub pane: Pane,
    pub name: &'static str,
    pub keys: &'static [&'static str],
    pub help: &'static str,
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
}

impl Keymap {
    /// The default keys, with the keys of the config in place of them. Also the problems of the
    /// config: unknown panes, actions and keys. The defaults stay for those.
    pub fn new(cfg: &KeyConfig) -> (Keymap, Vec<String>) {
        let mut problems = Vec::new();
        for (pane, actions) in cfg {
            let Some(p) = Pane::from_name(pane) else {
                problems.push(format!("[keys.{pane}]: no such pane (panes: {})", Pane::ALL.map(Pane::name).join(", ")));
                continue;
            };
            for name in actions.keys() {
                if !ACTIONS.iter().any(|a| a.pane == p && a.name == name) {
                    problems.push(format!("[keys.{pane}] {name}: no such action"));
                }
            }
        }
        let mut map = Keymap::default();
        for act in ACTIONS {
            let custom = cfg.get(act.pane.name()).and_then(|m| m.get(act.name));
            let mut keys = Vec::new();
            let source: Vec<&str> = custom.map(KeySpec::list).unwrap_or_else(|| act.keys.to_vec());
            for s in source {
                match Key::parse(s) {
                    Ok(k) => keys.push(k),
                    Err(e) => problems.push(format!("[keys.{}] {}: {e}", act.pane.name(), act.name)),
                }
            }
            if keys.is_empty() && custom.is_some() {
                keys = act.keys.iter().filter_map(|s| Key::parse(s).ok()).collect();
            }
            map.keys.insert((act.pane, act.name), keys);
        }
        // Later entries do not take a key from an earlier one; a custom key takes it from a default.
        let mut owner: HashMap<(Pane, Key), (&'static str, bool)> = HashMap::new();
        for act in ACTIONS {
            let custom = cfg.get(act.pane.name()).is_some_and(|m| m.contains_key(act.name));
            for k in &map.keys[&(act.pane, act.name)] {
                match owner.get(&(act.pane, *k)) {
                    Some((other, other_custom)) if *other_custom || !custom => {
                        if custom {
                            problems.push(format!("[keys.{}] {}: {} is also the key of {other}", act.pane.name(), act.name, k.label()));
                        }
                    }
                    _ => {
                        owner.insert((act.pane, *k), (act.name, custom));
                    }
                }
            }
        }
        map.by_key = owner.into_iter().map(|(k, (n, _))| (k, n)).collect();
        (map, problems)
    }

    pub fn defaults() -> Keymap {
        Keymap::new(&KeyConfig::new()).0
    }

    /// The action of this key in this pane; else the global action.
    pub fn action(&self, pane: Pane, k: &KeyEvent) -> Option<&'static str> {
        let key = Key::from_event(k);
        self.by_key.get(&(pane, key)).or_else(|| self.by_key.get(&(Pane::Global, key))).copied()
    }

    /// The action of this key in this pane only.
    pub fn local(&self, pane: Pane, k: &KeyEvent) -> Option<&'static str> {
        self.by_key.get(&(pane, Key::from_event(k))).copied()
    }

    /// The keys of an action, for the help: `j ↓`.
    pub fn label(&self, pane: Pane, name: &str) -> String {
        self.keys.iter().find(|((p, n), _)| *p == pane && *n == name).map(|(_, ks)| ks.iter().map(Key::label).collect::<Vec<_>>().join(" ")).unwrap_or_default()
    }

    /// The first key of an action, for the status bar.
    pub fn first(&self, pane: Pane, name: &str) -> String {
        self.keys.iter().find(|((p, n), _)| *p == pane && *n == name).and_then(|(_, ks)| ks.first()).map(Key::label).unwrap_or_default()
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
