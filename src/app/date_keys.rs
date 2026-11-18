//! The keys of the date selector in the note in write mode, the float and the chat input: they come
//! before every other key while the selector is on (TEXT-15, TEXT-16, TEXT-17).
use super::*;
use crate::text::editor::{DateFormat, DateKey};

impl App {
    /// A key while the date selector is on. True when the selector took it.
    pub(crate) fn date_key(&mut self, pane: crate::keys::Pane, k: &KeyEvent) -> bool {
        use crate::keys::Pane;
        let e = match pane {
            Pane::Write => self.writer.editor_mut(),
            Pane::Chat => self.composer.input_mut(),
            _ => return false,
        };
        if e.date.is_none() {
            return false;
        }
        let plain = !k.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        let key = match k.code {
            KeyCode::Up | KeyCode::Char('k') if plain => DateKey::Up,
            KeyCode::Down | KeyCode::Char('j') if plain => DateKey::Down,
            KeyCode::Left | KeyCode::Char('h') if plain => DateKey::Left,
            KeyCode::Right | KeyCode::Char('l') if plain => DateKey::Right,
            KeyCode::Enter | KeyCode::Esc => DateKey::Enter,
            _ => DateKey::Other,
        };
        let took = e.date_key(key);
        if took && pane == Pane::Write {
            self.writer.touch();
        }
        took
    }

    /// `/` was typed: `//` at the start of a word becomes the date of today (TEXT-15).
    pub(crate) fn slashes_to_date(e: &mut crate::text::editor::Editor) {
        e.slashes_to_date(chrono::Local::now().date_naive(), DateFormat::system());
    }
}
