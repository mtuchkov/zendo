//! Keys and paste of the session view, as the bytes that a terminal sends (xterm).

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// The modifier parameter of xterm: 1 + shift + 2 alt + 4 ctrl.
fn modifier(m: KeyModifiers) -> u8 {
    1 + u8::from(m.contains(KeyModifiers::SHIFT)) + 2 * u8::from(m.contains(KeyModifiers::ALT)) + 4 * u8::from(m.contains(KeyModifiers::CONTROL))
}

/// A cursor key: `ESC [ A`, `ESC O A` in application cursor mode, `ESC [ 1 ; m A` with modifiers.
fn cursor(c: char, m: KeyModifiers, app: bool) -> Vec<u8> {
    match modifier(m) {
        1 if app => format!("\x1bO{c}"),
        1 => format!("\x1b[{c}"),
        p => format!("\x1b[1;{p}{c}"),
    }
    .into_bytes()
}

/// A key with a number: `ESC [ n ~`, `ESC [ n ; m ~` with modifiers.
fn tilde(n: u8, m: KeyModifiers) -> Vec<u8> {
    match modifier(m) {
        1 => format!("\x1b[{n}~"),
        p => format!("\x1b[{n};{p}~"),
    }
    .into_bytes()
}

fn function(n: u8, m: KeyModifiers) -> Vec<u8> {
    match n {
        1..=4 => format!("\x1bO{}", (b'P' + n - 1) as char).into_bytes(),
        5 => tilde(15, m),
        6..=10 => tilde(n + 11, m),
        11 | 12 => tilde(n + 12, m),
        _ => Vec::new(),
    }
}

fn char_key(c: char, m: KeyModifiers) -> Vec<u8> {
    let mut out = Vec::new();
    if m.contains(KeyModifiers::ALT) {
        out.push(0x1b);
    }
    if m.contains(KeyModifiers::CONTROL) {
        match c {
            ' ' | '@' | '2' => out.push(0),
            'a'..='z' | 'A'..='Z' => out.push(c.to_ascii_lowercase() as u8 & 0x1f),
            '[' | '3' => out.push(0x1b),
            '\\' | '4' => out.push(0x1c),
            ']' | '5' => out.push(0x1d),
            '^' | '6' => out.push(0x1e),
            '_' | '-' | '7' => out.push(0x1f),
            '?' | '8' => out.push(0x7f),
            _ => out.extend(c.to_string().bytes()),
        }
    } else {
        out.extend(c.to_string().bytes());
    }
    out
}

/// The bytes of a key. `app_cursor`: the program set the application cursor mode.
pub fn encode(k: &KeyEvent, app_cursor: bool) -> Vec<u8> {
    let m = k.modifiers;
    let alt = |b: &[u8]| if m.contains(KeyModifiers::ALT) { [&[0x1b], b].concat() } else { b.to_vec() };
    match k.code {
        KeyCode::Char(c) => char_key(c, m),
        // Shift-enter and alt-enter: a new line in Claude Code.
        KeyCode::Enter if m.intersects(KeyModifiers::SHIFT | KeyModifiers::ALT) => b"\x1b\r".to_vec(),
        KeyCode::Enter => b"\r".to_vec(),
        KeyCode::Tab if m.contains(KeyModifiers::SHIFT) => b"\x1b[Z".to_vec(),
        KeyCode::Tab => b"\t".to_vec(),
        KeyCode::BackTab => b"\x1b[Z".to_vec(),
        KeyCode::Backspace if m.contains(KeyModifiers::CONTROL) => alt(&[0x08]),
        KeyCode::Backspace => alt(&[0x7f]),
        KeyCode::Esc => vec![0x1b],
        KeyCode::Up => cursor('A', m, app_cursor),
        KeyCode::Down => cursor('B', m, app_cursor),
        KeyCode::Right => cursor('C', m, app_cursor),
        KeyCode::Left => cursor('D', m, app_cursor),
        KeyCode::Home => cursor('H', m, app_cursor),
        KeyCode::End => cursor('F', m, app_cursor),
        KeyCode::Insert => tilde(2, m),
        KeyCode::Delete => tilde(3, m),
        KeyCode::PageUp => tilde(5, m),
        KeyCode::PageDown => tilde(6, m),
        KeyCode::F(n) => function(n, m),
        _ => Vec::new(),
    }
}

/// Pasted text, in brackets when the program turned bracketed paste on.
pub fn paste(s: &str, bracketed: bool) -> Vec<u8> {
    if bracketed { format!("\x1b[200~{s}\x1b[201~").into_bytes() } else { s.as_bytes().to_vec() }
}
