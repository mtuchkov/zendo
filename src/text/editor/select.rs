//! The selection, and bold, italic and underline marks around it.
use super::*;

impl Editor {
    /// The selection, start before end, when it is not empty.
    pub fn selection(&self) -> Option<((usize, usize), (usize, usize))> {
        let a = self.anchor?;
        let c = (self.row, self.col);
        if a == c {
            return None;
        }
        Some(if a < c { (a, c) } else { (c, a) })
    }

    pub fn selected_text(&self) -> Option<String> {
        let ((r1, c1), (r2, c2)) = self.selection()?;
        let mut out = String::new();
        for r in r1..=r2 {
            let chars: Vec<char> = self.lines[r].chars().collect();
            let from = if r == r1 { c1 } else { 0 };
            let to = if r == r2 { c2.min(chars.len()) } else { chars.len() };
            out.extend(&chars[from.min(to)..to]);
            if r < r2 {
                out.push('\n');
            }
        }
        Some(out)
    }

    /// A delete key: delete the selected text, or, when there is none, do `delete`.
    pub fn delete_selection_or(&mut self, delete: impl FnOnce(&mut Self)) {
        if !self.delete_selection() {
            delete(self);
        }
    }

    /// Delete the selected text; true when there was one.
    pub fn delete_selection(&mut self) -> bool {
        let Some(((r1, c1), (r2, c2))) = self.selection() else {
            self.anchor = None;
            return false;
        };
        let (b1, b2) = (self.byte(r1, c1), self.byte(r2, c2));
        let tail = self.lines[r2][b2..].to_string();
        self.lines[r1].truncate(b1);
        self.lines[r1].push_str(&tail);
        self.lines.drain(r1 + 1..=r2);
        self.row = r1;
        self.col = c1;
        self.anchor = None;
        self.touch();
        true
    }

    /// Move the cursor with the selection: the selection starts where the cursor is.
    pub fn select_with(&mut self, f: impl FnOnce(&mut Self)) {
        let here = (self.row, self.col);
        let anchor = self.anchor.unwrap_or(here);
        f(self);
        self.anchor = Some(anchor);
    }

    pub fn clear_selection(&mut self) {
        self.anchor = None;
    }

    /// The word around the cursor, as a char range of the line.
    pub(super) fn word_at(&self) -> Option<(usize, usize)> {
        let chars: Vec<char> = self.lines[self.row].chars().collect();
        let is_word = |c: char| c.is_alphanumeric() || c == '_' || c == '-' || c == '\'';
        let mut a = self.col.min(chars.len());
        let mut b = a;
        while a > 0 && is_word(chars[a - 1]) {
            a -= 1;
        }
        while b < chars.len() && is_word(chars[b]) {
            b += 1;
        }
        (a < b).then_some((a, b))
    }

    /// Bold, italic or underline: put `open` and `close` around the selection, or around the word at
    /// the cursor. When they are there already, take them away. With no word, put both and the cursor
    /// between them.
    pub fn wrap_marks(&mut self, open: &str, close: &str) {
        let (start, end) = match self.selection().and_then(|r| self.trim_range(r)) {
            Some(r) => r,
            None => match self.word_at() {
                Some((a, b)) => ((self.row, a), (self.row, b)),
                None => {
                    self.insert_str(open);
                    let col = self.col;
                    self.insert_str(close);
                    self.col = col;
                    return;
                }
            },
        };
        let (on, cn) = (open.chars().count(), close.chars().count());
        let before = |e: &Self| -> bool {
            let line: Vec<char> = e.lines[start.0].chars().collect();
            start.1 >= on && line[start.1 - on..start.1].iter().collect::<String>() == open
        };
        let after = |e: &Self| -> bool {
            let line: Vec<char> = e.lines[end.0].chars().collect();
            end.1 + cn <= line.len() && line[end.1..end.1 + cn].iter().collect::<String>() == close
        };
        let inside = self.text_between(start, end);
        // A bold mark is not an italic one: "**x**" is not "*x*" around "*x*".
        let marks_around = before(self) && after(self) && !(open == "*" && (self.char_at(start.0, start.1.wrapping_sub(2)) == Some('*')));
        if marks_around {
            let b = self.byte(end.0, end.1);
            let e = self.byte(end.0, end.1 + cn);
            self.lines[end.0].replace_range(b..e, "");
            let b = self.byte(start.0, start.1 - on);
            let e = self.byte(start.0, start.1);
            self.lines[start.0].replace_range(b..e, "");
            self.row = end.0;
            self.col = if start.0 == end.0 { end.1 - on } else { end.1 };
        } else if inside.starts_with(open) && inside.ends_with(close) && inside.chars().count() >= on + cn && !(open == "*" && inside.starts_with("**") && !inside.starts_with("***")) {
            let inner: String = inside.chars().skip(on).take(inside.chars().count() - on - cn).collect();
            self.anchor = Some(start);
            self.row = end.0;
            self.col = end.1;
            self.delete_selection();
            self.insert_str(&inner);
        } else {
            let b = self.byte(end.0, end.1);
            self.lines[end.0].insert_str(b, close);
            let b = self.byte(start.0, start.1);
            self.lines[start.0].insert_str(b, open);
            self.row = end.0;
            self.col = end.1 + if start.0 == end.0 { on } else { 0 } + cn;
        }
        self.anchor = None;
        self.touch();
    }

    /// The range without the spaces at its ends: marks must touch the text. None: only spaces.
    pub(super) fn trim_range(&self, (mut a, mut b): ((usize, usize), (usize, usize))) -> Option<((usize, usize), (usize, usize))> {
        let at = |p: (usize, usize)| self.char_at(p.0, p.1);
        while a < b && at(a).is_none_or(char::is_whitespace) {
            a = if at(a).is_none() { (a.0 + 1, 0) } else { (a.0, a.1 + 1) };
        }
        while a < b && (b.1 == 0 || at((b.0, b.1 - 1)).is_some_and(char::is_whitespace)) {
            b = if b.1 == 0 { (b.0 - 1, self.len(b.0 - 1)) } else { (b.0, b.1 - 1) };
        }
        (a < b).then_some((a, b))
    }
}
