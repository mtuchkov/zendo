//! Chips in the chat input: `⟦label⟧` tokens for a text (a selection, a paste) or a file. The
//! arrows and the delete keys take a chip as one unit.
use super::*;

pub const OPEN: char = '⟦';
pub const CLOSE: char = '⟧';

/// A paste longer than this, in chars, is a chip even on one line.
pub const PASTE_CHIP_CHARS: usize = 200;

/// A text chip says the first 15 chars of the text and, always, the number of its lines (CLAUDE-35).
pub fn text_label(text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let head: String = flat.chars().take(15).collect();
    let head = if flat.chars().count() > 15 { format!("{head}…") } else { head };
    match text.trim().lines().count().max(1) {
        1 => format!("{head} · 1 line"),
        n => format!("{head} · {n} lines"),
    }
}

/// A paste of more than one line, or a long line, goes into the chat input as a chip (CLAUDE-36).
pub fn paste_is_chip(text: &str) -> bool {
    let t = text.trim();
    t.lines().count() > 1 || t.chars().count() > PASTE_CHIP_CHARS
}

/// A file chip says the last 4 chars of its folder and the first 10 chars of its name: `ates/specificat`.
pub fn file_label(path: &Path) -> String {
    let name: String = path.file_name().map(|n| n.to_string_lossy().chars().take(10).collect()).unwrap_or_default();
    let dir = path.parent().and_then(|p| p.file_name()).map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
    if dir.is_empty() {
        return name;
    }
    let tail: String = dir.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
    format!("{tail}/{name}")
}

/// The chip spans of a line: `(start, end)` in chars, end after the close mark.
pub fn chip_ranges(line: &str) -> Vec<(usize, usize)> {
    let chars: Vec<char> = line.chars().collect();
    chars
        .iter()
        .enumerate()
        .filter(|(_, c)| **c == OPEN)
        .filter_map(|(i, _)| chars[i..].iter().position(|c| *c == CLOSE).map(|j| (i, i + j + 1)))
        .collect()
}

impl App {
    /// Put a chip into the chat input: at the cursor, or at the end. The label is unique.
    pub(crate) fn add_chip(&mut self, base: String, context: Option<String>, attach: Option<Attachment>, at_end: bool) -> String {
        let taken = |l: &str| self.chips.iter().any(|c| c.label == l);
        let label = (1..).map(|n| if n == 1 { format!("{OPEN}{base}{CLOSE}") } else { format!("{OPEN}{base} {n}{CLOSE}") }).find(|l| !taken(l)).expect("a free label");
        if at_end {
            self.input.move_to_end();
        }
        let before = self.input.lines[self.input.row].chars().nth(self.input.col.wrapping_sub(1));
        if self.input.col > 0 && before.is_some_and(|c| !c.is_whitespace()) {
            self.input.type_str(" ");
        }
        self.input.type_str(&format!("{label} "));
        self.chips.push(Chip { label: label.clone(), context, attach });
        label
    }

    /// A pasted text as a chip that starts with the text; the text goes as context.
    pub(crate) fn paste_chip(&mut self, text: &str) {
        let label = self.add_chip(text_label(text), None, None, false);
        let quoted: String = text.trim_matches('\n').lines().map(|l| format!("> {l}\n")).collect();
        if let Some(c) = self.chips.last_mut() {
            c.context = Some(format!("The user pasted this text as {label}:\n{quoted}"));
        }
    }

    /// The chip that ends at the cursor, or starts at it.
    fn chip_at(&self, before: bool) -> Option<(usize, usize)> {
        let col = self.input.col;
        chip_ranges(&self.input.lines[self.input.row]).into_iter().find(|(a, b)| if before { *b == col } else { *a == col })
    }

    /// Left, right, backspace and delete in the chat input: a chip is one unit. True when it was a chip.
    pub(crate) fn chip_key(&mut self, code: KeyCode) -> bool {
        let range = match code {
            KeyCode::Left | KeyCode::Backspace => self.chip_at(true),
            KeyCode::Right | KeyCode::Delete => self.chip_at(false),
            _ => None,
        };
        let Some((a, b)) = range else { return false };
        match code {
            KeyCode::Left => self.input.col = a,
            KeyCode::Right => self.input.col = b,
            _ => {
                let line = &mut self.input.lines[self.input.row];
                let label: String = line.chars().skip(a).take(b - a).collect();
                *line = line.chars().take(a).chain(line.chars().skip(b)).collect();
                self.input.col = a;
                self.input.dirty = true;
                self.chips.retain(|c| c.label != label);
            }
        }
        true
    }

    /// The chip at the cursor: the one the arrows stand on.
    pub fn chip_in_focus(&self) -> Option<(usize, usize, usize)> {
        let (a, b) = self.chip_at(false).or_else(|| self.chip_at(true))?;
        Some((self.input.row, a, b))
    }
}

#[cfg(test)]
#[path = "chips_tests.rs"]
mod tests;
