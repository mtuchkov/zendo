//! One key: from a terminal event, from the config text, and as a label.
use super::*;

/// One key with its modifiers. For a character, the character carries Shift (`J`, `+`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    pub code: KeyCode,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Key {
    pub fn from_event(k: &KeyEvent) -> Key {
        let m = k.modifiers;
        let (ctrl, alt, mut shift) = (m.contains(KeyModifiers::CONTROL), m.contains(KeyModifiers::ALT), m.contains(KeyModifiers::SHIFT));
        let code = match k.code {
            KeyCode::BackTab => {
                shift = true;
                KeyCode::Tab
            }
            // The kitty keyboard protocol sends alt-} as `]` with alt and shift: make it `}`.
            KeyCode::Char(c) if shift && alt && !ctrl && !c.is_ascii_alphabetic() => {
                shift = false;
                KeyCode::Char(us_shifted(c))
            }
            KeyCode::Char(c) => {
                // ctrl-shift-b keeps its Shift (kitty keyboard protocol); a plain char carries it.
                shift = shift && ctrl && c.is_ascii_alphabetic();
                // ctrl-B is ctrl-b: terminals do not agree on the case.
                KeyCode::Char(if ctrl { c.to_ascii_lowercase() } else { c })
            }
            c => c,
        };
        Key { code, ctrl, alt, shift }
    }

    pub fn parse(s: &str) -> Result<Key, String> {
        let mut k = Key { code: KeyCode::Null, ctrl: false, alt: false, shift: false };
        let mut rest = s.trim();
        if rest.is_empty() {
            return Err("an empty key".into());
        }
        loop {
            let lower = rest.to_ascii_lowercase();
            let (strip, flag) = if lower.starts_with("ctrl-") {
                (5, 0)
            } else if lower.starts_with("alt-") {
                (4, 1)
            } else if lower.starts_with("shift-") {
                (6, 2)
            } else {
                break;
            };
            if rest.len() == strip {
                break; // "ctrl-" alone: not a modifier
            }
            match flag {
                0 => k.ctrl = true,
                1 => k.alt = true,
                _ => k.shift = true,
            }
            rest = &rest[strip..];
        }
        k.code = match rest.to_ascii_lowercase().as_str() {
            "enter" | "return" => KeyCode::Enter,
            "esc" | "escape" => KeyCode::Esc,
            "tab" => KeyCode::Tab,
            "space" => KeyCode::Char(' '),
            "backspace" => KeyCode::Backspace,
            "delete" | "del" => KeyCode::Delete,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "pageup" => KeyCode::PageUp,
            "pagedown" => KeyCode::PageDown,
            f if f.len() > 1 && f.starts_with('f') && f[1..].parse::<u8>().is_ok_and(|n| (1..=12).contains(&n)) => KeyCode::F(f[1..].parse().expect("checked")),
            _ => {
                let mut cs = rest.chars();
                match (cs.next(), cs.next()) {
                    (Some(c), None) => {
                        if k.shift && k.ctrl && c.is_ascii_alphabetic() {
                            KeyCode::Char(c.to_ascii_lowercase())
                        } else if k.shift && c.is_ascii_alphabetic() {
                            k.shift = false;
                            KeyCode::Char(c.to_ascii_uppercase())
                        } else {
                            k.shift = false;
                            KeyCode::Char(if k.ctrl { c.to_ascii_lowercase() } else { c })
                        }
                    }
                    _ => return Err(format!("unknown key \"{s}\"")),
                }
            }
        };
        Ok(k)
    }

    /// The key as the keymap file writes it, so that `parse` gives it back: `alt-b`,
    /// `ctrl-shift-b`, `J`, `pageup` (REKEY-16).
    pub fn spec(&self) -> String {
        let name = match self.code {
            KeyCode::Enter => "enter".to_string(),
            KeyCode::Esc => "esc".to_string(),
            KeyCode::Tab => "tab".to_string(),
            KeyCode::Char(' ') => "space".to_string(),
            KeyCode::Backspace => "backspace".to_string(),
            KeyCode::Delete => "delete".to_string(),
            KeyCode::PageUp => "pageup".to_string(),
            KeyCode::PageDown => "pagedown".to_string(),
            KeyCode::F(n) => format!("f{n}"),
            KeyCode::Char(c) => c.to_string(),
            c => format!("{c:?}").to_lowercase(),
        };
        let mods = [(self.ctrl, "ctrl-"), (self.alt, "alt-"), (self.shift, "shift-")];
        mods.iter().filter(|(on, _)| *on).map(|(_, m)| *m).collect::<String>() + &name
    }

    /// A short label for the help and the status bar: `^b`, `⇧↓`, `alt-i`, `J`.
    pub fn label(&self) -> String {
        let name = match self.code {
            KeyCode::Enter => "enter".to_string(),
            KeyCode::Esc => "esc".to_string(),
            KeyCode::Tab => "tab".to_string(),
            KeyCode::Char(' ') => "space".to_string(),
            KeyCode::Backspace => "⌫".to_string(),
            KeyCode::Delete => "del".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::Home => "home".to_string(),
            KeyCode::End => "end".to_string(),
            KeyCode::PageUp => "pgup".to_string(),
            KeyCode::PageDown => "pgdn".to_string(),
            KeyCode::F(n) => format!("f{n}"),
            KeyCode::Char(c) => c.to_string(),
            _ => "?".to_string(),
        };
        // ctrl and `+` needs Shift on most keyboards: say so.
        if self.ctrl && !self.alt && self.code == KeyCode::Char('+') {
            return "^⇧+".to_string();
        }
        let mut s = String::new();
        if self.ctrl {
            s.push('^');
        }
        if self.alt {
            s.push_str("alt-");
        }
        if self.shift {
            s.push('⇧');
        }
        s + &name
    }
}

/// The character that Shift gives on a US keyboard: `]` gives `}`. Other characters stay.
fn us_shifted(c: char) -> char {
    const PAIRS: [(char, char); 21] = [
        ('`', '~'), ('1', '!'), ('2', '@'), ('3', '#'), ('4', '$'), ('5', '%'), ('6', '^'), ('7', '&'), ('8', '*'), ('9', '('),
        ('0', ')'), ('-', '_'), ('=', '+'), ('[', '{'), (']', '}'), ('\\', '|'), (';', ':'), ('\'', '"'), (',', '<'), ('.', '>'),
        ('/', '?'),
    ];
    PAIRS.iter().find(|(k, _)| *k == c).map_or(c, |(_, s)| *s)
}
