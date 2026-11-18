//! The free-key rules (WIZARD-10): which keys an action can take. The key float uses them for its
//! clashes and its suggestions, and checks each key of the model with them (REKEY-3, REKEY-7,
//! REKEY-11, REKEY-28, ADR 0012).
use super::*;

/// Why a key is not free for an action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clash {
    /// Another action has the key: its pane and its name.
    Taken(Pane, &'static str),
    /// A key without ctrl or alt, where the user types text.
    Text,
    /// `esc` stays fixed (REKEY-28, PANES-48).
    Esc,
}

impl Clash {
    /// What the float says about the clash: `tab: global · next_pane`.
    pub fn text(&self, key: Key) -> String {
        match self {
            Clash::Taken(p, n) => format!("{}: {} · {n}", key.label(), p.name()),
            Clash::Text => format!("{}: it types text there; take a key with ctrl or alt", key.label()),
            Clash::Esc => "esc stays fixed: it leaves a mode, a float or a box".to_string(),
        }
    }
}

/// The panes whose keys count for an action of `pane`: the pane, the panes that come before it or
/// after it, and the global pane; for a global action, every pane.
fn related(pane: Pane) -> Vec<Pane> {
    match pane {
        Pane::Global => Pane::ALL.to_vec(),
        Pane::Write | Pane::Chat => vec![pane, Pane::Selected, Pane::Global],
        Pane::Selected => vec![pane, Pane::Write, Pane::Chat, Pane::Global],
        p => vec![p, Pane::Global],
    }
}

/// Why `key` is not free for `action` of `pane`, or None when it is free. A key of the action
/// itself is free.
pub fn clash(map: &Keymap, pane: Pane, action: &str, key: Key) -> Option<Clash> {
    if key.code == KeyCode::Esc {
        return Some(Clash::Esc);
    }
    if map.keys_of(pane, action).contains(&key) {
        return None;
    }
    let plain = matches!(key.code, KeyCode::Char(_)) && !key.ctrl && !key.alt;
    if plain && matches!(pane, Pane::Global | Pane::Write | Pane::Chat) {
        return Some(Clash::Text);
    }
    related(pane).into_iter().find_map(|p| map.owner(p, key).filter(|n| *n != action).map(|n| Clash::Taken(p, n)))
}

/// A free key that is not a key of the action yet: a new key for it.
fn new_and_free(map: &Keymap, pane: Pane, action: &str, key: Key) -> bool {
    !map.keys_of(pane, action).contains(&key) && clash(map, pane, action, key).is_none()
}

/// The same key with other modifiers: alt, ctrl, ctrl-shift, alt-shift.
fn variants(k: Key) -> impl Iterator<Item = Key> {
    let code = match k.code {
        KeyCode::Char(c) => KeyCode::Char(c.to_ascii_lowercase()),
        c => c,
    };
    let mods = [(false, true, false), (true, false, false), (true, false, true), (false, true, true)];
    mods.into_iter().map(move |(ctrl, alt, shift)| Key { code, ctrl, alt, shift })
}

/// Up to `n` free keys for an action, by the rules: the pressed and the old keys with other
/// modifiers first, then `alt` and the letters of the action name, then any `alt` letter, then the
/// function keys (REKEY-11).
pub fn suggest(map: &Keymap, pane: Pane, action: &str, near: &[Key], n: usize) -> Vec<Key> {
    let alt = |c: char| Key { code: KeyCode::Char(c), ctrl: false, alt: true, shift: false };
    let named = action.chars().filter(char::is_ascii_lowercase).map(alt);
    let letters = ('a'..='z').map(alt);
    let fkeys = (1..=12).map(|i| Key { code: KeyCode::F(i), ctrl: false, alt: false, shift: false });
    let mut seen = std::collections::HashSet::new();
    near.iter()
        .copied()
        .flat_map(variants)
        .chain(named)
        .chain(letters)
        .chain(fkeys)
        .filter(|k| seen.insert(*k) && new_and_free(map, pane, action, *k))
        .take(n)
        .collect()
}

/// The keys of `specs` that parse and are new and free for the action, each once: the check of
/// the suggestions of the model (REKEY-7).
pub fn checked(map: &Keymap, pane: Pane, action: &str, specs: &[&str]) -> Vec<Key> {
    let mut seen = std::collections::HashSet::new();
    specs.iter().filter_map(|s| Key::parse(s).ok()).filter(|k| seen.insert(*k) && new_and_free(map, pane, action, *k)).collect()
}

/// The actions that the status bar shows as hints: the frequently used actions, until zendo ships
/// the usage counts from real use (PANES-18). Their last key does not go (REKEY-31).
const FREQUENT: &[(Pane, &[&str])] = &[
    (Pane::Global, &["next_pane", "next_session", "help", "toggle_tree", "terminal", "terminal_run", "quit"]),
    (Pane::Read, &["down", "up", "left", "right", "select_down", "select_up", "select_left", "select_right", "ask", "say", "add_to_chat", "chat", "write", "search", "help", "split", "copy", "back"]),
    (Pane::Write, &["done", "chat", "add_to_chat", "say", "bold", "italic", "underline", "toggle_tree", "delta"]),
    (Pane::Chat, &["send", "newline", "new_thread", "handoff", "back"]),
    (Pane::Selected, &["copy", "cut"]),
    (Pane::Transcript, &["down", "select_down", "copy", "add_to_chat", "input"]),
    (Pane::Tree, &["down", "open", "view", "search", "add_to_chat", "rename", "move", "delete", "close"]),
    (Pane::Delta, &["base", "ask", "recap", "close"]),
    (Pane::Notes, &["down", "open", "new", "rename", "tags", "close"]),
    (Pane::Session, &["leave"]),
];

/// The action is frequently used: it keeps at least one key (PANES-18, REKEY-31).
pub fn frequent(pane: Pane, action: &str) -> bool {
    FREQUENT.iter().any(|(p, names)| *p == pane && names.contains(&action))
}

#[cfg(test)]
#[path = "free_tests.rs"]
mod tests;
