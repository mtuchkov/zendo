//! Without the kitty keyboard protocol, a terminal sends ctrl-shift-k as ctrl-k.
use super::*;

impl Keymap {
    /// Without the kitty keyboard protocol, ctrl-shift-k comes as ctrl-k: when ctrl-k has no action,
    /// it does the action of ctrl-shift-k.
    pub(super) fn shifted(&self, pane: Pane, key: Key) -> Option<&'static str> {
        let alpha = matches!(key.code, KeyCode::Char(c) if c.is_ascii_alphabetic());
        (self.legacy && key.ctrl && !key.shift && alpha).then(|| self.by_key.get(&(pane, Key { shift: true, ..key })).copied()).flatten()
    }
}
