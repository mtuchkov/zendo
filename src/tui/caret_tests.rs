use super::*;
use crate::ui::theme::THEMES;

// FLOAT-16: in the float the cursor is a blinking block in the caret color of the float.
#[test]
fn the_cursor_of_the_float_is_a_rust_block() {
    let c = look(false, true);
    assert_eq!(c, Caret { block: true, color: Some(CARET) });
    assert_eq!(seq(c), "\x1b[1 q\x1b]12;#C8442B\x1b\\");
}

// Out of the float the cursor has the color of the terminal: a block in read mode, a bar where you
// type.
#[test]
fn out_of_the_float_the_cursor_has_the_terminal_color() {
    assert_eq!(seq(look(true, false)), "\x1b[1 q\x1b]112\x1b\\");
    assert_eq!(seq(look(false, false)), "\x1b[5 q\x1b]112\x1b\\");
}

// FLOAT-16: the cursor shows on the float of each theme, sand or umber.
#[test]
fn the_caret_shows_on_each_float() {
    for t in THEMES {
        assert!(contrast(CARET, t.on_float().bg) >= 3.0, "{}", t.name);
    }
}

/// The contrast ratio of two RGB colors (WCAG 2).
fn contrast(a: Color, b: Color) -> f64 {
    let lum = |c: Color| {
        let Color::Rgb(r, g, b) = c else { panic!("{c:?} is not RGB") };
        let lin = |v: u8| {
            let s = f64::from(v) / 255.0;
            if s <= 0.039_28 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
    };
    let (x, y) = (lum(a), lum(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}
