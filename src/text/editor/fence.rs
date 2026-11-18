//! Code fences: Enter after an opening fence closes it (TEXT-7), Enter in a code block keeps the
//! indent, and the code mark on lines puts fences around them (TEXT-6).
use super::*;

/// The fence at the start of a line, ``` or ~~~ or longer, with its indent: `("  ", "```")`.
fn fence(line: &str) -> Option<(&str, &str)> {
    let body = line.trim_start_matches(' ');
    let indent = &line[..line.len() - body.len()];
    let c = body.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let n = body.chars().take_while(|x| *x == c).count();
    (n >= 3).then(|| (indent, &body[..n]))
}

impl Editor {
    /// The rows of the fence lines, top to bottom.
    fn fence_rows(&self) -> Vec<usize> {
        self.lines.iter().enumerate().filter(|(_, l)| fence(l).is_some()).map(|(i, _)| i).collect()
    }

    /// The fences above the cursor row.
    fn fences_above(&self) -> usize {
        self.fence_rows().iter().take_while(|r| **r < self.row).count()
    }

    /// The cursor is at the end of a fence that opens a block, and no fence closes it: the line
    /// that closes it, with the indent of the fence.
    fn open_fence(&self) -> Option<String> {
        let (indent, marks) = fence(&self.lines[self.row])?;
        let opens = self.fences_above().is_multiple_of(2) && !self.fence_rows().len().is_multiple_of(2);
        (opens && self.col == self.len(self.row)).then(|| format!("{indent}{marks}"))
    }

    /// The cursor is at the end of a fence that opens a block that no fence closes.
    pub fn opens_fence(&self) -> bool {
        self.open_fence().is_some()
    }

    /// The cursor row is inside a code block, between its fences.
    pub fn in_fence(&self) -> bool {
        !self.fences_above().is_multiple_of(2) && fence(&self.lines[self.row]).is_none()
    }

    /// Enter in the text: the selection goes. In prose, a fence closes, a code block keeps the
    /// indent, and a list goes on (TEXT-7, TEXT-8, TEXT-9); in code, the line keeps its indent.
    pub fn enter(&mut self, prose: bool) {
        self.delete_selection();
        let indent: String = self.lines[self.row].chars().take_while(|c| c.is_whitespace()).collect();
        if let Some(close) = self.open_fence().filter(|_| prose) {
            let indent: String = close.chars().take_while(|c| *c == ' ').collect();
            self.split_line(false);
            self.insert_str(&indent);
            self.lines.insert(self.row + 1, close);
        } else if prose && !self.in_fence() {
            self.split_line(true);
        } else {
            self.split_line(false);
            self.insert_str(&indent);
        }
        self.touch();
    }

    /// The code mark: on a selection of more than one line, fences around its lines, or away when
    /// the lines start and end with fences (TEXT-6); else backticks (TEXT-5).
    pub fn wrap_code(&mut self) {
        let Some(((r1, _), (r2, c2))) = self.selection().filter(|((a, _), (b, _))| a != b) else {
            return self.wrap_marks("`", "`");
        };
        // A selection that ends at the start of a line does not take that line.
        let r2 = if c2 == 0 { r2 - 1 } else { r2 };
        let fenced = r2 > r1 && fence(&self.lines[r1]).is_some() && fence(&self.lines[r2]).is_some();
        if fenced {
            self.lines.remove(r2);
            self.lines.remove(r1);
            (self.row, self.col) = (r2 - 2, self.len(r2 - 2));
        } else {
            let indent: String = self.lines[r1].chars().take_while(|c| *c == ' ').collect();
            self.lines.insert(r2 + 1, format!("{indent}```"));
            self.lines.insert(r1, format!("{indent}```"));
            (self.row, self.col) = (r2 + 2, self.len(r2 + 2));
        }
        self.anchor = None;
        self.touch();
    }
}
