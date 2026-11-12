// SESSIONS-9: the keys and paste of the session view go to Claude Code as a terminal sends them.
use super::keys::{encode, paste};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn k(code: KeyCode, m: KeyModifiers) -> Vec<u8> {
    encode(&KeyEvent::new(code, m), false)
}

#[test]
fn chars_and_control_chars() {
    assert_eq!(k(KeyCode::Char('a'), KeyModifiers::NONE), b"a");
    assert_eq!(k(KeyCode::Char('я'), KeyModifiers::NONE), "я".as_bytes());
    assert_eq!(k(KeyCode::Char('A'), KeyModifiers::SHIFT), b"A");
    assert_eq!(k(KeyCode::Char('c'), KeyModifiers::CONTROL), [3]);
    assert_eq!(k(KeyCode::Char('o'), KeyModifiers::CONTROL), [15], "ctrl-o goes to Claude Code");
    assert_eq!(k(KeyCode::Char(' '), KeyModifiers::CONTROL), [0]);
    assert_eq!(k(KeyCode::Char('b'), KeyModifiers::ALT), b"\x1bb");
}

#[test]
fn named_keys() {
    assert_eq!(k(KeyCode::Enter, KeyModifiers::NONE), b"\r");
    assert_eq!(k(KeyCode::Enter, KeyModifiers::SHIFT), b"\x1b\r", "a new line in Claude Code");
    assert_eq!(k(KeyCode::Tab, KeyModifiers::NONE), b"\t");
    assert_eq!(k(KeyCode::BackTab, KeyModifiers::SHIFT), b"\x1b[Z");
    assert_eq!(k(KeyCode::Backspace, KeyModifiers::NONE), [0x7f]);
    assert_eq!(k(KeyCode::Esc, KeyModifiers::NONE), [0x1b]);
    assert_eq!(k(KeyCode::Delete, KeyModifiers::NONE), b"\x1b[3~");
    assert_eq!(k(KeyCode::PageUp, KeyModifiers::NONE), b"\x1b[5~");
    assert_eq!(k(KeyCode::F(1), KeyModifiers::NONE), b"\x1bOP");
    assert_eq!(k(KeyCode::F(5), KeyModifiers::NONE), b"\x1b[15~");
}

#[test]
fn arrows_with_modes_and_modifiers() {
    assert_eq!(k(KeyCode::Up, KeyModifiers::NONE), b"\x1b[A");
    assert_eq!(encode(&KeyEvent::new(KeyCode::Up, KeyModifiers::NONE), true), b"\x1bOA", "application cursor mode");
    assert_eq!(k(KeyCode::Left, KeyModifiers::CONTROL), b"\x1b[1;5D");
    assert_eq!(k(KeyCode::Right, KeyModifiers::ALT | KeyModifiers::SHIFT), b"\x1b[1;4C");
    assert_eq!(k(KeyCode::Home, KeyModifiers::NONE), b"\x1b[H");
}

#[test]
fn paste_is_bracketed_when_the_program_asks() {
    assert_eq!(paste("hi", true), b"\x1b[200~hi\x1b[201~");
    assert_eq!(paste("hi", false), b"hi");
}
