//! The suggestions of the light model in the key float (docs/zendo/panes/key-hints.md, ADR 0012):
//! one question in the background for each clash; the answer of an older question goes, and the
//! keys of the model pass the free-key rules before they show.
use super::*;
use crate::agent::api::Question;
use crate::keys::{ACTIONS, Key, Keymap, Pane};

impl App {
    /// The float opens: the hint channel takes the keymap in effect (REKEY-10).
    pub(super) fn hints_open(&mut self) {
        self.rekey.keymap_text = keymap_text(&self.keys);
    }

    /// A key clashed: one question to the model, on a thread; the float does not wait (REKEY-3,
    /// REKEY-4, REKEY-6).
    pub(super) fn hints_ask(&mut self, pressed: Key) {
        self.rekey.generation += 1;
        let (Some(api), Some(f)) = (self.rekey.api.as_ref(), self.rekey.float()) else { return };
        let prompt = prompt(&self.rekey.keymap_text, f.target, pressed, f.message().unwrap_or_default());
        let q = Question::Keys(self.rekey.generation);
        self.rekey.asker.ask(api, q, prompt, None);
    }

    /// Each tick: the answer of the current question gives the checked keys of the model; an
    /// answer of an older question, or an error, changes nothing (REKEY-5, REKEY-7, REKEY-13).
    pub(crate) fn hints_tick(&mut self) {
        let current = self.rekey.generation;
        let text = self.rekey.asker.answers().into_iter().filter_map(|(q, a)| (q == Question::Keys(current)).then_some(a.ok()).flatten()).next_back();
        let (Some(text), Some(t)) = (text, self.rekey.float().map(|f| f.target)) else { return };
        let words: Vec<&str> = text.split(|c: char| c.is_whitespace() || c == ',').map(|w| w.trim_matches(|c| matches!(c, '`' | '"' | '\'' | '.'))).filter(|w| !w.is_empty()).collect();
        let keys = crate::keys::checked(&self.keys, t.pane, t.action, &words);
        if let Some(f) = self.rekey.float_mut() {
            f.set_suggestions(keys.into_iter().take(3).collect());
        }
    }
}

/// The keymap in effect as the keymap file writes it: `[pane]`, then `action = key, key`.
fn keymap_text(map: &Keymap) -> String {
    Pane::ALL
        .iter()
        .map(|p| {
            let rows: Vec<String> = ACTIONS.iter().filter(|a| a.pane == *p).map(|a| format!("{} = {}", a.name, map.keys_of(*p, a.name).iter().map(Key::spec).collect::<Vec<_>>().join(", "))).collect();
            format!("[{}]\n{}\n", p.name(), rows.join("\n"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The question for the model: the keymap, the action, the old key, the pressed key and why it
/// clashed.
fn prompt(keymap: &str, t: rekey::KeyTarget, pressed: Key, why: &str) -> String {
    let help = ACTIONS.iter().find(|a| a.pane == t.pane && a.name == t.action).map_or("", |a| a.help);
    format!(
        "You suggest keys for zendo, a terminal app. The keys of an action of a pane come first, then the \
         global keys; where the user types text (write, chat), a key needs ctrl or alt. esc stays fixed.\n\n\
         The keymap in effect:\n{keymap}\n\
         The user changes the key {old} of the action {action} ({help}) in the pane {pane}. The user pressed \
         {new}, but it clashes: {why}.\n\n\
         Suggest three free keys for this action that keep the sense of {old} and {new}; short keys first. \
         Answer only with the keys, one on each line, as the keymap writes them: alt-t, ctrl-shift-y, f7.",
        old = t.key.spec(),
        action = t.action,
        pane = t.pane.name(),
        new = pressed.spec(),
    )
}
