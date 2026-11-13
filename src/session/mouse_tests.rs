// TERM-43: the mouse events of a session go to the program as a terminal sends them (xterm).
use super::encode;
use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use vt100::{MouseProtocolEncoding as Encoding, MouseProtocolMode as Mode};

fn at(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
    MouseEvent { kind, column, row, modifiers: KeyModifiers::NONE }
}

const LEFT: MouseButton = MouseButton::Left;

#[test]
fn a_program_that_does_not_report_the_mouse_gets_nothing() {
    for kind in [MouseEventKind::Down(LEFT), MouseEventKind::Up(LEFT), MouseEventKind::ScrollUp, MouseEventKind::Moved] {
        assert!(encode(&at(kind, 0, 0), Mode::None, Encoding::Sgr).is_empty());
    }
}

#[test]
fn sgr_names_the_button_the_cell_from_1_and_the_release() {
    let sgr = |kind, x, y| String::from_utf8(encode(&at(kind, x, y), Mode::AnyMotion, Encoding::Sgr)).unwrap();
    assert_eq!(sgr(MouseEventKind::Down(LEFT), 0, 0), "\x1b[<0;1;1M");
    assert_eq!(sgr(MouseEventKind::Down(MouseButton::Right), 9, 4), "\x1b[<2;10;5M");
    assert_eq!(sgr(MouseEventKind::Up(LEFT), 9, 4), "\x1b[<0;10;5m");
    assert_eq!(sgr(MouseEventKind::Drag(LEFT), 300, 4), "\x1b[<32;301;5M");
    assert_eq!(sgr(MouseEventKind::Moved, 1, 1), "\x1b[<35;2;2M");
    assert_eq!(sgr(MouseEventKind::ScrollUp, 0, 0), "\x1b[<64;1;1M");
    assert_eq!(sgr(MouseEventKind::ScrollDown, 0, 0), "\x1b[<65;1;1M");
}

#[test]
fn the_modifiers_are_in_the_button_code() {
    let m = MouseEvent { kind: MouseEventKind::Down(LEFT), column: 0, row: 0, modifiers: KeyModifiers::SHIFT | KeyModifiers::CONTROL };
    assert_eq!(encode(&m, Mode::PressRelease, Encoding::Sgr), b"\x1b[<20;1;1M");
    let m = MouseEvent { kind: MouseEventKind::ScrollUp, column: 0, row: 0, modifiers: KeyModifiers::ALT };
    assert_eq!(encode(&m, Mode::PressRelease, Encoding::Sgr), b"\x1b[<72;1;1M");
}

#[test]
fn a_mode_reports_only_its_events() {
    let sent = |kind, mode| !encode(&at(kind, 0, 0), mode, Encoding::Sgr).is_empty();
    let kinds = [MouseEventKind::Down(LEFT), MouseEventKind::ScrollDown, MouseEventKind::Up(LEFT), MouseEventKind::Drag(LEFT), MouseEventKind::Moved];
    let reports = |mode| kinds.map(|k| sent(k, mode));
    assert_eq!(reports(Mode::Press), [true, true, false, false, false]);
    assert_eq!(reports(Mode::PressRelease), [true, true, true, false, false]);
    assert_eq!(reports(Mode::ButtonMotion), [true, true, true, true, false]);
    assert_eq!(reports(Mode::AnyMotion), [true, true, true, true, true]);
}

#[test]
fn the_default_encoding_is_three_bytes_and_a_release_has_no_button() {
    let old = |kind, x, y| encode(&at(kind, x, y), Mode::ButtonMotion, Encoding::Default);
    assert_eq!(old(MouseEventKind::Down(LEFT), 0, 0), b"\x1b[M\x20\x21\x21");
    assert_eq!(old(MouseEventKind::Up(LEFT), 9, 4), b"\x1b[M\x23\x2a\x25");
    assert_eq!(old(MouseEventKind::ScrollUp, 0, 0), b"\x1b[M\x60\x21\x21");
    assert_eq!(old(MouseEventKind::Down(LEFT), 222, 0), b"\x1b[M\x20\xff\x21");
    assert!(old(MouseEventKind::Down(LEFT), 223, 0).is_empty(), "a column past 223 has no byte");
}

#[test]
fn the_utf8_encoding_has_chars_for_the_far_cells() {
    let utf8 = |x| encode(&at(MouseEventKind::Down(LEFT), x, 0), Mode::Press, Encoding::Utf8);
    assert_eq!(utf8(0), b"\x1b[M\x20\x21\x21");
    assert_eq!(utf8(222), [b"\x1b[M\x20".as_slice(), "\u{ff}".as_bytes(), b"\x21"].concat());
    assert!(utf8(2100).is_empty());
}
