//! A file that is not markdown shows as code: its lines wrapped at the column, in the colors of its
//! language, with line numbers. Read mode and write mode show it the same way.
use crate::text::editor::{TAB_WIDTH, wrap_line};
use crate::text::highlight::{self, Kind};
use crate::text::markdown::Rendered;
use crate::ui::theme::Theme;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use std::path::Path;

/// The text column of a code file, in chars, without the line numbers.
pub const CODE_WIDTH: u16 = 120;

/// A file shows as code when it is not markdown.
pub fn is_code(p: &Path) -> bool {
    p.extension().is_none_or(|e| e != "md")
}

/// The language of a file for the highlighter: its extension, or its name when it has none.
pub fn lang_of(p: &Path) -> String {
    p.extension().or_else(|| p.file_name()).map(|e| e.to_string_lossy().into_owned()).unwrap_or_default()
}

/// The width of the line numbers of a file of `n` lines: the digits and one space.
pub fn gutter(n: usize) -> u16 {
    n.max(1).to_string().len() as u16 + 1
}

/// The rows of a file: each line wrapped at `width`, in pieces of a kind. `rows` has the first row of
/// each line, and `starts` the line and the char where each row starts.
pub fn render(lines: &[String], lang: &str, width: usize, t: &Theme) -> Rendered {
    let pieces = highlight::file_pieces(lang, lines);
    let mut out = Rendered { lines: Vec::new(), rows: Vec::new(), words: Vec::new(), starts: Vec::new() };
    for (i, line) in lines.iter().enumerate() {
        out.rows.push(out.lines.len());
        let chars: Vec<char> = line.chars().collect();
        let kinds = kinds_of(pieces.as_ref().and_then(|p| p.get(i)), chars.len());
        for (a, b) in wrap_line(line, width) {
            out.starts.push(Some((i, a)));
            out.lines.push(Line::from(spans(&chars[a..b], &kinds[a..b], t)));
        }
    }
    out.words = vec![0; out.lines.len()];
    out
}

/// The kind of each char of a line; plain when the pieces do not fit the line.
fn kinds_of(pieces: Option<&Vec<(Kind, String)>>, n: usize) -> Vec<Kind> {
    let kinds: Vec<Kind> = pieces.into_iter().flatten().flat_map(|(k, s)| s.chars().map(move |_| *k)).collect();
    if kinds.len() == n { kinds } else { vec![Kind::Plain; n] }
}

/// One span for each run of chars of one kind. A tab shows as spaces.
fn spans(chars: &[char], kinds: &[Kind], t: &Theme) -> Vec<Span<'static>> {
    let base = Style::new().fg(t.fg);
    let mut out: Vec<(Kind, String)> = Vec::new();
    for (c, k) in chars.iter().zip(kinds) {
        let text = if *c == '\t' { " ".repeat(TAB_WIDTH) } else { c.to_string() };
        match out.last_mut() {
            Some((lk, s)) if lk == k => s.push_str(&text),
            _ => out.push((*k, text)),
        }
    }
    out.into_iter().map(|(k, s)| Span::styled(s, k.style(base, t))).collect()
}

#[cfg(test)]
#[path = "code_tests.rs"]
mod tests;
