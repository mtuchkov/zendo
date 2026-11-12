//! Hard wrap in write mode: a line that gets longer than the text column breaks at a word, with the
//! indent of a list item or the `>` of a quote, so read mode joins it again.
use super::*;

/// What a wrapped line goes on with: the indent, the `>` of a quote, or spaces under a list marker.
fn continuation(line: &str) -> String {
    let lead: String = line.chars().take_while(|c| c.is_whitespace()).collect();
    let t = line.trim_start();
    if t.starts_with('>') {
        return format!("{lead}> ");
    }
    let digits = t.chars().take_while(char::is_ascii_digit).count();
    let marker = ["- [ ] ", "- [x] ", "- ", "* ", "+ "].iter().find(|m| t.starts_with(**m)).map(|m| m.len()).or_else(|| (digits > 0 && t[digits..].starts_with(". ")).then_some(digits + 2));
    format!("{lead}{}", " ".repeat(marker.unwrap_or(0)))
}

/// A word that would start a block of its own at the start of a line.
fn starts_block(rest: &[char]) -> bool {
    let s: String = rest.iter().take(4).collect();
    let digits = s.chars().take_while(char::is_ascii_digit).count();
    ["- ", "* ", "+ ", "#", ">", "|", "```"].iter().any(|m| s.starts_with(m)) || (digits > 0 && s[digits..].starts_with(". "))
}

/// Lines that keep their length: headings, tables, code, front matter, HTML.
fn keeps_length(lines: &[String], row: usize) -> bool {
    let t = lines[row].trim_start();
    let fences = lines[..row].iter().filter(|l| l.trim_start().starts_with("```")).count();
    let front = lines.first().is_some_and(|l| l.trim() == "---") && !lines[1..row.max(1)].iter().any(|l| l.trim() == "---");
    t.starts_with('#') || t.starts_with('|') || t.starts_with("```") || t.starts_with('<') || fences % 2 == 1 || (front && row > 0)
}

/// The char where the line breaks: the last space that keeps the line within the width.
fn break_at(chars: &[char], width: usize, from: usize) -> Option<usize> {
    let mut w = 0;
    chars
        .iter()
        .enumerate()
        .take_while(|(_, c)| {
            w += c.width().unwrap_or(0);
            w <= width + 1
        })
        .filter(|(i, c)| **c == ' ' && *i > from && !starts_block(&chars[i + 1..]))
        .map(|(i, _)| i)
        .last()
}

impl Editor {
    /// Break the line of the cursor when it is longer than `width`. True when it broke.
    pub fn hard_wrap(&mut self, width: usize) -> bool {
        let row = self.row;
        let line = self.lines[row].clone();
        if unicode_width::UnicodeWidthStr::width(line.as_str()) <= width || keeps_length(&self.lines, row) {
            return false;
        }
        let chars: Vec<char> = line.chars().collect();
        let prefix = continuation(&line);
        let Some(b) = break_at(&chars, width, prefix.chars().count()) else { return false };
        let head: String = chars[..b].iter().collect::<String>().trim_end().to_string();
        let tail: String = chars[b + 1..].iter().collect();
        self.lines[row] = head;
        self.lines.insert(row + 1, format!("{prefix}{tail}"));
        if self.col > b {
            self.row += 1;
            self.col = self.col - (b + 1) + prefix.chars().count();
        }
        self.dirty = true;
        true
    }
}

#[cfg(test)]
#[path = "wrap_tests.rs"]
mod tests;
