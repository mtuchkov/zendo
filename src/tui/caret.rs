//! The look of the terminal cursor: its shape, and its color in the float (FLOAT-16).
use crate::ui::theme::CARET;
use ratatui::style::Color;

/// The shape and the color of the cursor. No color: the color of the terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Caret {
    pub block: bool,
    pub color: Option<Color>,
}

/// A blinking block on the char in read mode and in the float; a blinking bar where you type. In
/// the float the block has the caret color (FLOAT-16).
pub(super) fn look(read: bool, float: bool) -> Caret {
    Caret { block: read || float, color: float.then_some(CARET) }
}

/// The escape sequences that give the cursor its look: DECSCUSR for the shape, OSC 12 for the
/// color and OSC 112 to give the terminal its own color back.
pub(super) fn seq(c: Caret) -> String {
    let shape = if c.block { "\x1b[1 q" } else { "\x1b[5 q" };
    let color = match c.color {
        Some(Color::Rgb(r, g, b)) => format!("\x1b]12;#{r:02X}{g:02X}{b:02X}\x1b\\"),
        _ => "\x1b]112\x1b\\".to_string(),
    };
    format!("{shape}{color}")
}

/// The escape sequence that gives the terminal its own cursor color back.
pub(super) const RESET_COLOR: &str = "\x1b]112\x1b\\";

#[cfg(test)]
#[path = "caret_tests.rs"]
mod tests;
