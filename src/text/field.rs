//! A one-line text field with a cursor and a selection: find, search, the tree filter, a file
//! name. Every field has the same keys, from `[textbox]` of the keymap (TEXT-1, TEXT-2, TEXT-11).
use super::editor::{Editor, char_width};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// A one-line text: the editor with one line.
#[derive(Clone, Debug)]
pub struct Field {
    e: Editor,
}

/// What a key did in a field. The view of the field does the clipboard.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Typed {
    /// The text changed.
    Changed,
    /// Only the cursor or the selection moved.
    Moved,
    /// Copy this text.
    Copy(String),
    /// The text changed; copy what went out.
    Cut(String),
    /// Paste the clipboard with `paste`.
    Paste,
    /// No key of the field: the view of the field does it.
    Not,
}

impl Default for Field {
    fn default() -> Self {
        Self { e: Editor::new("") }
    }
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        self.text() == other.text() && self.cursor() == other.cursor()
    }
}

impl Eq for Field {}

/// A field reads as its text: `field.trim()`, `field.starts_with('?')`.
impl std::ops::Deref for Field {
    type Target = str;
    fn deref(&self) -> &str {
        self.text()
    }
}

impl From<&str> for Field {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}

impl From<String> for Field {
    fn from(text: String) -> Self {
        Self::new(&text)
    }
}

impl Field {
    /// A field with this text and the cursor at its end.
    pub fn new(text: &str) -> Self {
        let mut f = Self::default();
        f.set(text);
        f
    }

    pub fn text(&self) -> &str {
        &self.e.lines[0]
    }

    pub fn is_empty(&self) -> bool {
        self.text().is_empty()
    }

    /// The new text, the cursor at its end.
    pub fn set(&mut self, text: &str) {
        self.e = Editor::new(&one_line(text));
        self.e.end();
    }

    pub fn clear(&mut self) {
        self.set("");
    }

    /// The cursor: the char index in the text.
    pub fn cursor(&self) -> usize {
        self.e.col
    }

    /// The columns on the screen before the cursor.
    pub fn cursor_width(&self) -> usize {
        self.text().chars().take(self.cursor()).map(char_width).sum()
    }

    /// The selection as a char range, start before end.
    pub fn selection(&self) -> Option<(usize, usize)> {
        self.e.selection().map(|((_, a), (_, b))| (a, b))
    }

    /// Put the text in at the cursor, in place of the selection; a new line becomes a space (TEXT-3).
    pub fn paste(&mut self, text: &str) {
        self.e.type_str(&one_line(text));
    }

    /// A key: `act` is its action in `[textbox]` of the keymap, when it has one.
    pub fn key(&mut self, act: Option<&str>, k: &KeyEvent) -> Typed {
        match act.map(|a| self.action(a)) {
            Some(Typed::Not) | None => self.plain(k),
            Some(t) => t,
        }
    }

    fn action(&mut self, a: &str) -> Typed {
        let sel = self.e.selected_text();
        match a {
            "copy" => sel.map_or(Typed::Moved, Typed::Copy),
            "cut" => sel.map_or(Typed::Moved, |t| {
                self.e.delete_selection();
                Typed::Cut(t)
            }),
            "paste" => Typed::Paste,
            "delete_word" => self.changed(|e| e.delete_selection_or(Editor::delete_word_back)),
            "clear" => self.changed(|e| *e = Editor::new("")),
            _ if self.e.line_move(a) => Typed::Moved,
            _ => Typed::Not,
        }
    }

    /// A key that is no action: typing, deleting and the arrows.
    fn plain(&mut self, k: &KeyEvent) -> Typed {
        let typed = !k.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        let name = match k.code {
            KeyCode::Char(c) if typed => return self.changed(|e| e.type_char(c)),
            KeyCode::Backspace => return self.changed(|e| e.delete_selection_or(Editor::backspace)),
            KeyCode::Delete => return self.changed(|e| e.delete_selection_or(Editor::delete)),
            KeyCode::Left => "left",
            KeyCode::Right => "right",
            KeyCode::Home => "home",
            KeyCode::End => "end",
            _ => return Typed::Not,
        };
        self.e.line_move(name);
        Typed::Moved
    }

    fn changed(&mut self, f: impl FnOnce(&mut Editor)) -> Typed {
        f(&mut self.e);
        Typed::Changed
    }
}

/// The text in one line: the new lines at the end go, the others become spaces.
fn one_line(text: &str) -> String {
    text.trim_end_matches(['\n', '\r']).replace("\r\n", " ").replace(['\n', '\r'], " ")
}

#[cfg(test)]
#[path = "field_tests.rs"]
mod tests;
