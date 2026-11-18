use super::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn key(f: &mut Field, act: Option<&str>, code: KeyCode) -> Typed {
    f.key(act, &KeyEvent::new(code, KeyModifiers::NONE))
}

fn typed(text: &str) -> Field {
    let mut f = Field::default();
    text.chars().for_each(|c| drop(key(&mut f, None, KeyCode::Char(c))));
    f
}

// TEXT-1: the arrows, home, end and the word moves move the cursor in the field; typing goes in at the cursor.
#[test]
fn the_cursor_moves_by_chars_words_and_to_the_ends() {
    let mut f = typed("docs zendo");
    assert_eq!(f.cursor(), 10);
    key(&mut f, Some("word_left"), KeyCode::Null);
    assert_eq!(f.cursor(), 5);
    key(&mut f, None, KeyCode::Left);
    key(&mut f, None, KeyCode::Char('/'));
    assert_eq!(f.text(), "docs/ zendo");
    key(&mut f, Some("home"), KeyCode::Null);
    assert_eq!(f.cursor(), 0);
    key(&mut f, Some("end"), KeyCode::Null);
    assert_eq!(f.cursor(), 11);
    key(&mut f, None, KeyCode::Home);
    assert_eq!(f.cursor(), 0, "the home key works without the keymap too");
    key(&mut f, None, KeyCode::Delete);
    assert_eq!(f.text(), "ocs/ zendo");
}

// TEXT-2: a selection by the keys gives the text to copy and to cut; typing replaces it.
#[test]
fn select_copy_cut_and_type_over() {
    let mut f = typed("one two");
    key(&mut f, Some("select_word_left"), KeyCode::Null);
    assert_eq!(f.selection(), Some((4, 7)));
    assert_eq!(key(&mut f, Some("copy"), KeyCode::Null), Typed::Copy("two".into()));
    assert_eq!(key(&mut f, Some("cut"), KeyCode::Null), Typed::Cut("two".into()));
    assert_eq!(f.text(), "one ");
    key(&mut f, Some("select_home"), KeyCode::Null);
    key(&mut f, None, KeyCode::Char('x'));
    assert_eq!(f.text(), "x");
}

// TEXT-2, TEXT-3: paste goes in at the cursor, one line: a new line becomes a space.
#[test]
fn paste_is_one_line() {
    let mut f = typed("ab");
    key(&mut f, None, KeyCode::Left);
    assert_eq!(key(&mut f, Some("paste"), KeyCode::Null), Typed::Paste);
    f.paste("1\n2\r\n");
    assert_eq!(f.text(), "a1 2b");
}

// TEXT-1: the keys that are not for the field go to the view of the field.
#[test]
fn other_keys_are_not_taken() {
    let mut f = typed("x");
    for code in [KeyCode::Enter, KeyCode::Esc, KeyCode::Up, KeyCode::Down, KeyCode::Tab] {
        assert_eq!(key(&mut f, None, code), Typed::Not, "{code:?}");
    }
    let ctrl = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::CONTROL);
    assert_eq!(f.key(None, &ctrl), Typed::Not);
    assert_eq!(f.text(), "x");
}

// TEXT-1: delete_word and clear edit the field.
#[test]
fn delete_word_and_clear() {
    let mut f = typed("one two");
    assert_eq!(key(&mut f, Some("delete_word"), KeyCode::Null), Typed::Changed);
    assert_eq!(f.text(), "one ");
    key(&mut f, Some("clear"), KeyCode::Null);
    assert!(f.is_empty());
}
