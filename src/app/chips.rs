//! Chips in the chat input: `⟦label⟧` tokens for a text (a selection, a paste) or a file. The
//! arrows and the delete keys take a chip as one unit.
use super::Composer;
use crate::setup::history::SavedChip;
use super::super::*;

pub const OPEN: char = '⟦';
pub const CLOSE: char = '⟧';

/// A paste longer than this, in chars, is a chip even on one line (CLAUDE-42).
pub const PASTE_CHIP_CHARS: usize = 200;

/// A text chip says the first 15 chars of the text and, always, the number of its lines (CLAUDE-20, CLAUDE-41).
pub fn text_label(text: &str) -> String {
    let flat = unescape(text).split_whitespace().collect::<Vec<_>>().join(" ");
    let head: String = flat.chars().take(15).collect();
    let head = if flat.chars().count() > 15 { format!("{head}…") } else { head };
    match text.trim().lines().count().max(1) {
        1 => format!("{head} · 1 line"),
        n => format!("{head} · {n} lines"),
    }
}

/// A paste of more than one line, or of a long line, goes into the chat input as a chip (CLAUDE-42).
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

/// The chip spans of a text: `(start, end)` in chars, end after the close mark. A mark after an
/// odd run of `\\` is text (CLAUDE-61).
pub fn chip_ranges(line: &str) -> Vec<(usize, usize)> {
    let chars: Vec<char> = line.chars().collect();
    let real = |i: &usize| chars[..*i].iter().rev().take_while(|c| **c == '\\').count() % 2 == 0;
    let close_after = |i: usize| (i + 1..chars.len()).filter(real).find(|j| chars[*j] == CLOSE).map(|j| (i, j + 1));
    let mut from = 0;
    std::iter::from_fn(|| {
        let r = (from..chars.len()).filter(real).filter(|i| chars[*i] == OPEN).find_map(close_after)?;
        from = r.1;
        Some(r)
    })
    .collect()
}

/// Escape the marks of a text that goes into a chip: a mark gets a `\\`, and a run of `\\` right
/// before a mark or at the end doubles (CLAUDE-61).
pub fn escape(text: &str) -> String {
    escape_with(text, true)
}

/// The same for a text that goes into the chat input: a run at the end stays.
pub fn escape_marks(text: &str) -> String {
    escape_with(text, false)
}

fn escape_with(text: &str, end: bool) -> String {
    let (mut out, run) = text.chars().fold((String::new(), 0), |(mut out, run), c| {
        if c == '\\' {
            return (out, run + 1);
        }
        let n = if c == OPEN || c == CLOSE { 2 * run + 1 } else { run };
        out.extend(std::iter::repeat_n('\\', n));
        out.push(c);
        (out, 0)
    });
    out.extend(std::iter::repeat_n('\\', if end { 2 * run } else { run }));
    out
}

/// The text of a chip as it was before `escape`.
pub fn unescape(text: &str) -> String {
    let (mut out, run) = text.chars().fold((String::new(), 0), |(mut out, run), c| {
        if c == '\\' {
            return (out, run + 1);
        }
        out.extend(std::iter::repeat_n('\\', if c == OPEN || c == CLOSE { run / 2 } else { run }));
        out.push(c);
        (out, 0)
    });
    out.extend(std::iter::repeat_n('\\', run / 2));
    out
}

/// In a question that the transcript keeps, the label of a chip comes before its text and this
/// char, so the transcript shows the label of the chat input (CLAUDE-62). It does not go to the
/// agent.
pub const LABEL_END: char = '\u{2063}';

/// The text of a chip without the label that the transcript keeps before it.
pub fn body_of(content: &str) -> &str {
    content.split_once(LABEL_END).map_or(content, |(_, b)| b)
}

/// The label of a chip from the text in its marks: the label that the transcript keeps, else
/// (CLAUDE-64) a file for `@path`, the quoted lines with `↩` for text from the chat, or the first
/// 15 chars.
pub fn label_of(content: &str) -> String {
    if let Some((label, _)) = content.split_once(LABEL_END) {
        return label.to_string();
    }
    let text = unescape(content);
    if let Some(p) = text.strip_prefix('@').filter(|p| !p.contains('\n')) {
        return file_label(Path::new(p));
    }
    let quoted: Vec<&str> = text.lines().filter_map(|l| l.strip_prefix("> ").or((l == ">").then_some(""))).collect();
    if quoted.is_empty() {
        let head: String = text.chars().take(15).collect();
        return if text.chars().count() > 15 { format!("{head}…") } else { head };
    }
    let label = text_label(&quoted.join("\n"));
    if text.starts_with("Text from this chat") { format!("{CHAT_MARK} {label}") } else { label }
}

/// A chip whose text is only a short label: zendo does not know what it brought (CLAUDE-65).
pub fn unknown(content: &str) -> bool {
    !content.contains(LABEL_END) && !content.contains('\n') && !content.starts_with('@') && content.chars().count() <= 15
}

/// The text with each chip as `chip` makes it; the chip of a range is the one with its label.
pub fn inline(text: &str, chips: &[Chip], chip: impl Fn(&Chip) -> String) -> String {
    let chars: Vec<char> = text.chars().collect();
    let (mut out, at) = chip_ranges(text).into_iter().fold((String::new(), 0), |(mut out, at), (a, b)| {
        let label: String = chars[a..b].iter().collect();
        out.extend(&chars[at..a]);
        out.push_str(&chips.iter().find(|c| c.label == label).map_or(label, &chip));
        (out, b)
    });
    out.extend(&chars[at..]);
    out
}

/// The text without its chips.
pub fn without_chips(q: &str) -> String {
    cut(q, chip_ranges(q))
}

/// The text without the char ranges, in the order of `chip_ranges`.
fn cut(q: &str, ranges: Vec<(usize, usize)>) -> String {
    ranges.into_iter().rev().fold(q.to_string(), |t, (a, b)| t.chars().take(a).chain(t.chars().skip(b)).collect())
}

impl Composer {
    /// Put a chip into the chat input: at the cursor, or at the end. The label is unique.
    pub fn add_chip(&mut self, base: String, context: Option<String>, attach: Option<Attachment>, at_end: bool) -> String {
        let label = self.free_label(&base);
        if at_end {
            self.input.move_to_end();
        }
        let before = self.input.lines[self.input.row].chars().nth(self.input.col.wrapping_sub(1));
        if self.input.col > 0 && before.is_some_and(|c| !c.is_whitespace()) {
            self.input.type_str(" ");
        }
        self.input.type_str(&format!("{label} "));
        // The marks in a text of a chip are escaped (CLAUDE-61).
        self.chips.push(Chip { label: label.clone(), context: context.map(|c| escape(c.trim_end_matches('\n'))), attach });
        label
    }

    /// `text` and a chip `label` with `context`, before what the input holds: the context goes only
    /// with that text (SPEC-10).
    pub fn put_first(&mut self, text: &str, label: &str, context: &str) {
        let chip = Chip { label: format!("{OPEN}{label}{CLOSE}"), context: Some(escape(context)), attach: None };
        self.put_back(&format!("{text} {}", super::shown_chip(&chip)));
    }

    /// Questions that go back into the input, before what it holds: each chip in them is a chip
    /// again (CLAUDE-34, CLAUDE-62).
    pub fn put_back(&mut self, shown: &str) {
        let rest = self.input.take();
        let chars: Vec<char> = shown.chars().collect();
        let at = chip_ranges(shown).into_iter().fold(0, |at, (a, b)| {
            self.input.type_str(&chars[at..a].iter().collect::<String>());
            let content: String = chars[a + 1..b - 1].iter().collect();
            let body = unescape(body_of(&content));
            let file = body.strip_prefix('@').filter(|p| !p.contains('\n')).map(|p| Attachment::of(PathBuf::from(p)));
            let context = file.is_none().then_some(body);
            self.add_chip(label_of(&content), context, file, false);
            b
        });
        self.input.type_str(&chars[at..].iter().collect::<String>());
        if !rest.is_empty() {
            self.input.type_str(&format!("\n{rest}"));
        }
    }

    /// A pasted text as a chip at the cursor that starts with the text; the text goes as context
    /// (CLAUDE-42). Its source line names no label: a label has marks (CLAUDE-61).
    pub fn paste_chip(&mut self, text: &str) -> String {
        let quoted: Vec<String> = text.trim_matches('\n').lines().map(|l| format!("> {l}")).collect();
        self.add_chip(text_label(text), Some(format!("Text that the user pasted:\n{}", quoted.join("\n"))), None, false)
    }

    /// `⟦base⟧`, or `⟦base 2⟧`, `⟦base 3⟧`… when a chip in the input has it.
    fn free_label(&self, base: &str) -> String {
        // A label has no marks of its own: they would end the chip (CLAUDE-61).
        let base = base.replace(OPEN, "[").replace(CLOSE, "]");
        let taken = |l: &str| self.chips.iter().any(|c| c.label == l);
        (1..).map(|n| if n == 1 { format!("{OPEN}{base}{CLOSE}") } else { format!("{OPEN}{base} {n}{CLOSE}") }).find(|l| !taken(l)).expect("a free label")
    }

    /// Label the chip `old` with a free label from `base`, in the input and in the chips; the
    /// cursor stays at its place in the text.
    pub fn rename_chip(&mut self, old: &str, base: &str) {
        let new = self.free_label(base);
        let Some(row) = self.input.lines.iter().position(|l| l.contains(old)) else { return };
        let line = &self.input.lines[row];
        let start = line[..line.find(old).unwrap_or(0)].chars().count();
        if row == self.input.row && self.input.col > start {
            self.input.col = (self.input.col + new.chars().count()).saturating_sub(old.chars().count()).max(start);
        }
        self.input.lines[row] = line.replacen(old, &new, 1);
        self.chips.iter_mut().filter(|c| c.label == old).for_each(|c| c.label = new.clone());
    }

    /// The chip that ends at the cursor, or starts at it.
    fn chip_at(&self, before: bool) -> Option<(usize, usize)> {
        let col = self.input.col;
        chip_ranges(&self.input.lines[self.input.row]).into_iter().find(|(a, b)| if before { *b == col } else { *a == col })
    }

    /// Left, right, backspace and delete in the chat input: a chip is one unit. True when it was a chip.
    pub fn chip_key(&mut self, code: KeyCode) -> bool {
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

    /// What the history keeps of the message in the input: the text with the labels of its text
    /// chips, and those chips; the labels of file and image chips leave the text (HISTORY-7).
    pub fn history_entry(&self) -> (String, Vec<SavedChip>) {
        let text = self.input.text();
        let is_text = |c: &Chip| c.attach.is_none() && c.context.is_some() && text.contains(c.label.as_str());
        let kept: Vec<SavedChip> = self.chips.iter().filter(|c| is_text(c)).map(|c| SavedChip { label: c.label.clone(), text: c.context.clone().unwrap_or_default() }).collect();
        let label = |&(a, b): &(usize, usize)| text.chars().skip(a).take(b - a).collect::<String>();
        let drop = chip_ranges(&text).into_iter().filter(|r| !kept.iter().any(|c| c.label == label(r))).collect();
        (cut(&text, drop), kept)
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
