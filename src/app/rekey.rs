//! The change of a key in place (docs/zendo/panes/key-change.md): a click on a key of the status
//! bar or of the help makes it a button, a click on the button opens the key float, and the next
//! key replaces the key, joins the keys of the action, or a click removes it.
use super::*;
use crate::keys::{Clash, Key, Pane};
use std::path::PathBuf;

/// A key of an action on the screen: what a click on it changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyTarget {
    pub pane: Pane,
    pub action: &'static str,
    pub key: Key,
}

/// What the next key does in the float: it replaces the key, or joins the keys of the action
/// (REKEY-25).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Choice {
    #[default]
    Replace,
    Add,
}

/// A click in the float: a choice, `[remove]`, or a suggested key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloatButton {
    Choice(Choice),
    Remove,
    Suggestion(Key),
}

/// The open key float.
pub struct KeyFloat {
    pub target: KeyTarget,
    pub choice: Choice,
    message: Option<String>,
    suggestions: Vec<Key>,
    /// The suggestions came from the model (REKEY-11).
    pub from_model: bool,
    pub opened: Instant,
}

impl KeyFloat {
    /// The clash or the error of the last key (REKEY-3, REKEY-27, REKEY-31).
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// The suggested keys: of the rules first, then of the model (REKEY-11).
    pub fn suggestions(&self) -> &[Key] {
        &self.suggestions
    }

    /// The model gives its checked keys; they take the place of the keys of the rules.
    pub fn set_suggestions(&mut self, keys: Vec<Key>) {
        if !keys.is_empty() {
            self.suggestions = keys;
            self.from_model = true;
        }
    }
}

/// The key button, the key float, and the config where the keys go.
#[derive(smart_default::SmartDefault)]
pub struct Rekey {
    /// The key that a click made a button, and the focus and the mode at that time (REKEY-30).
    button: Option<(KeyTarget, Focus, Mode)>,
    float: Option<KeyFloat>,
    /// The number of the open question to the model; an answer of an older one goes (REKEY-5).
    pub generation: u64,
    #[default(crate::setup::config::config_path())]
    pub config: PathBuf,
    /// The API of the light model of the hints, with low effort (REKEY-6, ADR 0012).
    pub api: Option<crate::agent::api::Api>,
    /// The questions to the model on their threads.
    pub(super) asker: crate::agent::api::Asker,
    /// The keymap in effect, as text, for the model; it comes when the float opens (REKEY-10).
    pub(super) keymap_text: String,
}

impl Rekey {
    pub fn button(&self) -> Option<KeyTarget> {
        self.button.map(|(t, _, _)| t)
    }

    pub fn float(&self) -> Option<&KeyFloat> {
        self.float.as_ref()
    }

    pub fn float_mut(&mut self) -> Option<&mut KeyFloat> {
        self.float.as_mut()
    }

    /// The draw: when the focus or the mode moved, the button is text again (REKEY-30).
    pub fn follow(&mut self, focus: Focus, mode: Mode) {
        if self.button.is_some_and(|(_, f, m)| f != focus || m != mode) {
            self.button = None;
        }
    }

    /// Close the float; the open question to the model is cancelled (REKEY-5, REKEY-14).
    fn close(&mut self) {
        self.float = None;
        self.generation += 1;
    }
}

impl App {
    /// A click on a key of the screen: the first makes it a button, the second opens the float
    /// (REKEY-1). True when the click was on a key.
    pub(super) fn click_key(&mut self, x: u16, y: u16) -> bool {
        let at = ratatui::layout::Position::new(x, y);
        let Some(target) = self.frame.key_targets().iter().find(|(r, _)| r.contains(at)).map(|(_, t)| *t) else {
            self.rekey.button = None;
            return false;
        };
        if self.rekey.button() == Some(target) {
            self.rekey.button = None;
            self.open_key_float(target);
        } else {
            self.rekey.button = Some((target, self.focus, self.mode));
        }
        true
    }

    fn open_key_float(&mut self, target: KeyTarget) {
        self.rekey.generation += 1;
        self.rekey.float = Some(KeyFloat { target, choice: Choice::Replace, message: None, suggestions: Vec::new(), from_model: false, opened: Instant::now() });
        self.hints_open();
    }

    /// A click while the float is open: on a choice, on `[remove]` or on a suggested key.
    pub(super) fn click_key_float(&mut self, x: u16, y: u16) {
        let at = ratatui::layout::Position::new(x, y);
        let Some(b) = self.frame.float_buttons().iter().find(|(r, _)| r.contains(at)).map(|(_, b)| *b) else { return };
        match b {
            FloatButton::Choice(c) => self.rekey.float.iter_mut().for_each(|f| f.choice = c),
            FloatButton::Remove => self.remove_key(),
            FloatButton::Suggestion(k) => self.take_key(k),
        }
    }

    /// A key while the float is open: `esc` closes it, any other key is the new key (REKEY-2,
    /// REKEY-3, REKEY-14).
    pub(super) fn key_key_float(&mut self, k: KeyEvent) {
        if k.code == KeyCode::Esc {
            return self.rekey.close();
        }
        self.take_key(Key::from_event(&k));
    }

    /// The new key: a free one goes to the keymap, one that clashes stays out, with the reason and
    /// suggestions.
    fn take_key(&mut self, key: Key) {
        let Some(f) = self.rekey.float.as_ref() else { return };
        let (t, choice) = (f.target, f.choice);
        if choice == Choice::Replace && key == t.key {
            return self.rekey.close();
        }
        if let Some(c) = crate::keys::clash(&self.keys, t.pane, t.action, key) {
            return self.clash(key, c);
        }
        // The new key once; it takes the place of the old key, or comes first (REKEY-9, REKEY-25).
        let mut keys: Vec<Key> = self.keys.keys_of(t.pane, t.action).into_iter().filter(|k| *k != key).collect();
        match keys.iter().position(|k| *k == t.key).filter(|_| choice == Choice::Replace) {
            Some(i) => keys[i] = key,
            None => keys.insert(0, key),
        }
        let word = match choice {
            Choice::Add => format!("{}: {} added", t.action, key.label()),
            Choice::Replace => format!("{} → {}", t.key.label(), key.label()),
        };
        self.save_keys(t, &keys, &word);
    }

    fn clash(&mut self, key: Key, c: Clash) {
        let Some(f) = self.rekey.float.as_mut() else { return };
        f.message = Some(c.text(key));
        if c != Clash::Esc {
            f.suggestions = crate::keys::suggest(&self.keys, f.target.pane, f.target.action, &[f.target.key, key], 3);
            f.from_model = false;
            self.hints_ask(key);
        }
    }

    /// `[remove]`: the key goes, but not the last key of a frequent action (REKEY-29, REKEY-31).
    fn remove_key(&mut self) {
        let Some(t) = self.rekey.float.as_ref().map(|f| f.target) else { return };
        let mut keys = self.keys.keys_of(t.pane, t.action);
        keys.retain(|k| *k != t.key);
        if keys.is_empty() && crate::keys::frequent(t.pane, t.action) {
            let msg = format!("{} is the last key of {}: a frequent action keeps a key", t.key.label(), t.action);
            return self.rekey.float.iter_mut().for_each(|f| f.message = Some(msg.clone()));
        }
        self.save_keys(t, &keys, &format!("{}: {} removed", t.action, t.key.label()));
    }

    /// Write the keys, apply the keymap at once and say what changed; a failed write keeps the
    /// float open with the error (REKEY-2, REKEY-16, REKEY-27).
    fn save_keys(&mut self, t: KeyTarget, keys: &[Key], word: &str) {
        match crate::setup::config::write_keys(&self.rekey.config, t.pane, t.action, keys) {
            Ok(path) => {
                self.reload_keys();
                self.rekey.close();
                self.flash(format!("{word}, saved in {}", path.display()));
            }
            Err(e) => self.rekey.float.iter_mut().for_each(|f| f.message = Some(e.clone())),
        }
    }

    /// The keymap again from the config: the hints, the help and the keys show the change.
    fn reload_keys(&mut self) {
        let user = crate::setup::config::UserConfig::load_from(&self.rekey.config);
        let (mut keys, problems) = user.keymap();
        keys.legacy = self.keys.legacy;
        self.keys = keys;
        self.key_help = KeyHelp::new(problems, user.keymap.clone());
    }
}
