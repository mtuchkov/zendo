//! The keys of every one-line text field: find, search, the tree filter, a file name. The field
//! does the text; the app does the clipboard (TEXT-1, TEXT-2, TEXT-3).
use super::*;
use crate::text::field::{Field, Typed};

/// The field that a key goes to.
pub(crate) type FieldOf = fn(&mut App) -> Option<&mut Field>;

impl App {
    /// A key in a field: the keys of `[textbox]`, then typing, deleting and the arrows.
    /// `Some(changed)` when the field took the key; `None` when the view of the field does it.
    pub(crate) fn field_key(&mut self, k: &KeyEvent, field: FieldOf) -> Option<bool> {
        self.field_action(k, field).or_else(|| match field(self)?.key(None, k) {
            Typed::Changed => Some(true),
            Typed::Not => None,
            _ => Some(false),
        })
    }

    /// A key of `[textbox]` in a field, with the clipboard. `None` when the key has no action there.
    pub(crate) fn field_action(&mut self, k: &KeyEvent, field: FieldOf) -> Option<bool> {
        let act = self.keys.local(crate::keys::Pane::Field, k)?;
        let typed = field(self)?.key(Some(act), k);
        if typed != Typed::Not {
            self.did(crate::keys::Pane::Field, act);
        }
        match typed {
            Typed::Not => None,
            Typed::Changed => Some(true),
            Typed::Moved => Some(false),
            Typed::Copy(t) => {
                self.pointer.copy(t);
                Some(false)
            }
            Typed::Cut(t) => {
                self.pointer.copy(t);
                Some(true)
            }
            // The clipboard is read on a worker; the text comes in `poll_clipboard` (IO-1).
            Typed::Paste => {
                self.paste_from_clipboard(PasteTo::Field);
                Some(false)
            }
        }
    }

    /// A paste from the terminal into the field that has the keys now; false when no field has
    /// them (TEXT-3).
    pub(crate) fn paste_into_field(&mut self, text: &str) -> bool {
        let Some(f) = self.field_now() else { return false };
        f.paste(text);
        match self.mode {
            _ if self.files.prompt().is_some() => {}
            Mode::Find => self.run_find(),
            _ => self.files.filter_changed(),
        }
        true
    }

    /// The field that has the keys now: a file name, the find panel, the tree filter.
    fn field_now(&mut self) -> Option<&mut Field> {
        if self.files.prompt().is_some() {
            return self.files.prompt_field();
        }
        match self.mode {
            Mode::Find => Some(&mut self.files.find.query),
            _ if self.files.filtering() && self.key_pane() == crate::keys::Pane::Tree => self.files.filter_field(),
            _ => None,
        }
    }
}
