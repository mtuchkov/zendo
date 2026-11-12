//! The chat transcript as text to read: `tab` from the chat input moves the keys here. The cursor
//! and the selection keys of read mode work on the rows of the thread; `y` copies, and `a` adds
//! the text to the chat input as a chip of its own color.
use super::*;

/// The mark at the start of the label of a chip from the transcript.
pub const CHAT_MARK: char = '↩';

/// The cursor in the transcript, over the rows of the last frame.
#[derive(Clone, Debug, Default)]
pub struct Transcript {
    /// The transcript has the keys of the chat pane, not the input.
    pub on: bool,
    pub rows: Vec<String>,
    pub row: usize,
    pub col: usize,
    want: usize,
    pub anchor: Option<(usize, usize)>,
    /// The shell blocks of the last frame: their first row, the row after them, their command.
    pub blocks: Vec<(usize, usize, String)>,
}

impl Transcript {
    fn len(&self, r: usize) -> usize {
        self.rows.get(r).map_or(0, |l| l.trim_end().chars().count())
    }

    /// The cursor on the last row with text.
    pub fn to_end(&mut self) {
        self.row = self.rows.iter().rposition(|r| !r.trim().is_empty()).unwrap_or(0);
        self.col = 0;
        self.want = 0;
        self.anchor = None;
    }

    pub fn move_rows(&mut self, by: isize) {
        let last = self.rows.len().saturating_sub(1) as isize;
        self.row = (self.row as isize + by).clamp(0, last.max(0)) as usize;
        self.col = self.want.min(self.len(self.row).saturating_sub(1));
    }

    /// By chars, across rows.
    pub fn move_col(&mut self, by: isize) {
        let len = self.len(self.row);
        match by < 0 {
            true if self.col > 0 => self.col = self.col.min(len).saturating_sub(1),
            true if self.row > 0 => {
                self.row -= 1;
                self.col = self.len(self.row).saturating_sub(1);
            }
            false if self.col + 1 < len => self.col += 1,
            false if self.row + 1 < self.rows.len() => (self.row, self.col) = (self.row + 1, 0),
            _ => {}
        }
        self.want = self.col;
    }

    fn space(&self) -> bool {
        self.rows.get(self.row).and_then(|r| r.chars().nth(self.col)).is_none_or(char::is_whitespace)
    }

    /// To the start of the next word, or of this or the previous word.
    pub fn move_word(&mut self, right: bool) {
        let step = |t: &mut Self| {
            let before = (t.row, t.col);
            t.move_col(if right { 1 } else { -1 });
            (t.row, t.col) != before
        };
        if right {
            while !self.space() && step(self) {}
            while self.space() && step(self) {}
            return;
        }
        while step(self) && self.space() {}
        while self.col > 0 && !self.rows[self.row].chars().nth(self.col - 1).is_none_or(char::is_whitespace) {
            self.move_col(-1);
        }
    }

    /// The command of the shell block under the cursor.
    pub fn block(&self) -> Option<&str> {
        self.blocks.iter().find(|(a, b, _)| (*a..*b).contains(&self.row)).map(|(.., c)| c.as_str())
    }

    /// The selection, start before end, both ends in it.
    pub fn selection(&self) -> Option<((usize, usize), (usize, usize))> {
        let a = self.anchor?;
        let c = (self.row, self.col);
        Some(if a <= c { (a, c) } else { (c, a) })
    }

    /// The selected text; without a selection, the message under the cursor (the rows between
    /// blank rows).
    pub fn text(&self) -> String {
        let ((r1, c1), (r2, c2)) = self.selection().unwrap_or_else(|| {
            let blank = |i: usize| self.rows.get(i).is_none_or(|l| l.trim().is_empty());
            let a = (0..=self.row).rev().take_while(|i| !blank(*i)).last().unwrap_or(self.row);
            let b = (self.row..self.rows.len()).take_while(|i| !blank(*i)).last().unwrap_or(self.row);
            ((a, 0), (b, usize::MAX - 1))
        });
        let rows = (r1..=r2.min(self.rows.len().saturating_sub(1))).map(|r| {
            let chars: Vec<char> = self.rows[r].trim_end().chars().collect();
            let from = if r == r1 { c1.min(chars.len()) } else { 0 };
            let to = if r == r2 { (c2 + 1).min(chars.len()) } else { chars.len() };
            chars[from..to.max(from)].iter().collect::<String>()
        });
        rows.map(|r| r.trim_start_matches(" › ").to_string()).collect::<Vec<_>>().join("\n").trim().to_string()
    }
}

impl App {
    /// The keys of the transcript.
    pub(crate) fn key_transcript(&mut self, act: Option<&str>) {
        let page = self.view_height.saturating_sub(3).max(1) as isize;
        let Some(act) = act else { return };
        let t = &mut self.transcript;
        if act.starts_with("select_") && t.anchor.is_none() {
            t.anchor = Some((t.row, t.col));
        }
        match act {
            "down" | "select_down" => t.move_rows(1),
            "up" | "select_up" => t.move_rows(-1),
            "left" | "select_left" => t.move_col(-1),
            "right" | "select_right" => t.move_col(1),
            "select_word_left" => t.move_word(false),
            "select_word_right" => t.move_word(true),
            "page_down" => t.move_rows(page),
            "page_up" => t.move_rows(-page),
            "top" => t.move_rows(-(isize::MAX / 4)),
            "bottom" => t.move_rows(isize::MAX / 4),
            "select" => t.anchor = if t.anchor.is_some() { None } else { Some((t.row, t.col)) },
            "back" if t.anchor.is_some() => t.anchor = None,
            "back" | "input" => t.on = false,
            "copy" => self.copy_transcript(),
            "add_to_chat" => self.transcript_to_chat(),
            _ => {}
        }
    }

    fn copy_transcript(&mut self) {
        let text = self.transcript.text();
        if !text.is_empty() {
            self.clip = Some(text);
            self.transcript.anchor = None;
        }
    }

    /// `a`: the selected text of the chat goes to the input as a chip of its own color.
    fn transcript_to_chat(&mut self) {
        let text = self.transcript.text();
        if text.is_empty() {
            return self.flash("nothing to add here");
        }
        let quoted: String = text.lines().map(|l| format!("> {l}\n")).collect();
        let ctx = format!("Text from this chat:\n{quoted}");
        let label = self.add_chip(format!("{CHAT_MARK} {}", chips::selection_label(&text)), Some(ctx), None, true);
        self.transcript.anchor = None;
        self.flash(format!("{} is in the chat", label.trim_matches([chips::OPEN, chips::CLOSE])));
    }
}

#[cfg(test)]
#[path = "transcript_tests.rs"]
mod tests;
