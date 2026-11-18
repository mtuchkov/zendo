//! Marks typed on a selection: `*` bold, `_` italic, `` ` `` code, `>` quote (TEXT-13).
use super::*;

impl Editor {
    /// Type a char. On a selection, `*`, `_`, `` ` `` and `>` mark it bold, italic, code or quote,
    /// or take the mark away; another char takes the place of the selection.
    pub fn type_over(&mut self, c: char) {
        if self.selection().is_none() {
            return self.type_char(c);
        }
        match c {
            '*' => self.wrap_marks("**", "**"),
            '_' => self.wrap_marks("_", "_"),
            '`' => self.wrap_code(),
            '>' => self.quote_lines(),
            _ => self.type_char(c),
        }
    }

    /// `> ` before each line of the selection; when every line has it, take it away.
    pub fn quote_lines(&mut self) {
        let ((r1, _), (r2, c2)) = self.selection().unwrap_or(((self.row, 0), (self.row, 1)));
        // A selection that ends at the start of a line does not take that line.
        let r2 = if c2 == 0 && r2 > r1 { r2 - 1 } else { r2 };
        let quoted = self.lines[r1..=r2].iter().all(|l| l.trim_start().starts_with('>'));
        for line in &mut self.lines[r1..=r2] {
            let indent = line.len() - line.trim_start().len();
            if quoted {
                let rest = line[indent + 1..].strip_prefix(' ').unwrap_or(&line[indent + 1..]).to_string();
                line.replace_range(indent.., &rest);
            } else {
                line.insert_str(indent, "> ");
            }
        }
        (self.row, self.col) = (r2, self.len(r2));
        self.anchor = None;
        self.touch();
    }
}
