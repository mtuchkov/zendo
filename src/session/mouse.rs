//! The mouse of a session, as the bytes that a terminal sends (xterm mouse tracking): a program
//! that reports the mouse gets the events itself (TERM-43).

use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use vt100::{MouseProtocolEncoding as Encoding, MouseProtocolMode as Mode};

/// The flag of a move with or without a button, and the base of the wheel.
const MOTION: u16 = 32;
const WHEEL: u16 = 64;
/// The button number of a release, where the encoding does not name the button.
const RELEASE: u16 = 3;

fn button(b: MouseButton) -> u16 {
    match b {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
    }
}

/// The button code of xterm, and whether the event is a release; `None`: `mode` does not report it.
fn code(kind: MouseEventKind, mode: Mode) -> Option<(u16, bool)> {
    let level = match mode {
        Mode::None => return None,
        Mode::Press => 0,
        Mode::PressRelease => 1,
        Mode::ButtonMotion => 2,
        Mode::AnyMotion => 3,
    };
    let (code, release, needs) = match kind {
        MouseEventKind::Down(b) => (button(b), false, 0),
        MouseEventKind::ScrollUp => (WHEEL, false, 0),
        MouseEventKind::ScrollDown => (WHEEL + 1, false, 0),
        MouseEventKind::ScrollLeft => (WHEEL + 2, false, 0),
        MouseEventKind::ScrollRight => (WHEEL + 3, false, 0),
        MouseEventKind::Up(b) => (button(b), true, 1),
        MouseEventKind::Drag(b) => (button(b) + MOTION, false, 2),
        MouseEventKind::Moved => (RELEASE + MOTION, false, 3),
    };
    (level >= needs).then_some((code, release))
}

fn modifiers(m: KeyModifiers) -> u16 {
    4 * u16::from(m.contains(KeyModifiers::SHIFT)) + 8 * u16::from(m.contains(KeyModifiers::ALT)) + 16 * u16::from(m.contains(KeyModifiers::CONTROL))
}

/// One value of the old encodings: the value plus 32, as a byte, or as a char in UTF-8. `None`:
/// the value is too large for the encoding.
fn value(v: u16, utf8: bool) -> Option<Vec<u8>> {
    let v = u32::from(v) + 32;
    match (utf8, v) {
        (_, ..=127) => Some(vec![v as u8]),
        (false, ..=255) => Some(vec![v as u8]),
        (true, ..=2047) => char::from_u32(v).map(|c| c.to_string().into_bytes()),
        _ => None,
    }
}

/// The bytes of mouse event `m` for a program with `mode` and `encoding`; `m` has the cell on the
/// screen of the program, from 0. Empty: the program does not get this event.
pub fn encode(m: &MouseEvent, mode: Mode, encoding: Encoding) -> Vec<u8> {
    let Some((code, release)) = code(m.kind, mode) else { return Vec::new() };
    let code = code + modifiers(m.modifiers);
    let (x, y) = (m.column + 1, m.row + 1);
    if encoding == Encoding::Sgr {
        return format!("\x1b[<{code};{x};{y}{}", if release { 'm' } else { 'M' }).into_bytes();
    }
    let utf8 = encoding == Encoding::Utf8;
    let code = if release { RELEASE + modifiers(m.modifiers) } else { code };
    let parts = [value(code, utf8), value(x, utf8), value(y, utf8)];
    if parts.iter().any(Option::is_none) {
        return Vec::new();
    }
    [b"\x1b[M".to_vec()].into_iter().chain(parts.into_iter().flatten()).flatten().collect()
}

#[cfg(test)]
#[path = "mouse_tests.rs"]
mod tests;
