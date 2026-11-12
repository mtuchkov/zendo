//! Typing and deleting.
use super::*;

impl Editor {
    /// Type a char: it takes the place of the selection.
    pub fn type_char(&mut self, c: char) {
        self.delete_selection();
        self.insert(c);
    }

    /// Paste or type a text: it takes the place of the selection.
    pub fn type_str(&mut self, s: &str) {
        self.delete_selection();
        self.insert_str(s);
    }

    pub fn move_to_end(&mut self) {
        self.row = self.lines.len() - 1;
        self.col = self.len(self.row);
    }

    pub fn insert(&mut self, c: char) {
        let b = self.byte(self.row, self.col);
        self.lines[self.row].insert(b, c);
        self.col += 1;
        self.touch();
    }

    pub fn insert_str(&mut self, s: &str) {
        for c in s.replace("\r\n", "\n").replace('\r', "\n").chars() {
            if c == '\n' { self.split_line(false) } else if c == '\t' { self.insert_str("    ") } else { self.insert(c) }
        }
    }

    /// Enter. With `continue_list`, a bullet line starts the next bullet.
    pub fn split_line(&mut self, continue_list: bool) {
        let b = self.byte(self.row, self.col);
        let tail = self.lines[self.row].split_off(b);
        let mut prefix = String::new();
        if continue_list {
            let line = &self.lines[self.row];
            let indent: String = line.chars().take_while(|c| *c == ' ').collect();
            let body = &line[indent.len()..];
            for p in ["- [ ] ", "- ", "* "] {
                if body.starts_with(p) {
                    if body.len() == p.len() && tail.is_empty() {
                        // Enter on an empty bullet ends the list.
                        self.lines[self.row].clear();
                        self.col = 0;
                        self.touch();
                        return;
                    }
                    prefix = format!("{indent}{p}");
                    break;
                }
            }
        }
        self.row += 1;
        self.lines.insert(self.row, format!("{prefix}{tail}"));
        self.col = prefix.chars().count();
        self.touch();
    }

    pub fn backspace(&mut self) {
        if self.col > 0 {
            let b = self.byte(self.row, self.col - 1);
            self.lines[self.row].remove(b);
            self.col -= 1;
        } else if self.row > 0 {
            let line = self.lines.remove(self.row);
            self.row -= 1;
            self.col = self.len(self.row);
            self.lines[self.row].push_str(&line);
        } else {
            return;
        }
        self.touch();
    }

    pub fn delete(&mut self) {
        if self.col < self.len(self.row) {
            let b = self.byte(self.row, self.col);
            self.lines[self.row].remove(b);
        } else if self.row + 1 < self.lines.len() {
            let next = self.lines.remove(self.row + 1);
            self.lines[self.row].push_str(&next);
        } else {
            return;
        }
        self.touch();
    }

    pub fn delete_word_back(&mut self) {
        if self.col == 0 {
            self.backspace();
            return;
        }
        let target = self.word_left_col();
        let a = self.byte(self.row, target);
        let b = self.byte(self.row, self.col);
        self.lines[self.row].replace_range(a..b, "");
        self.col = target;
        self.touch();
    }
}
