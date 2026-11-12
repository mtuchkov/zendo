//! "Here is what I want to say": `ctrl-h` marks a place in the note with an arrow, and the chat asks
//! the agent to turn the idea of the user into text that connects with the paragraphs around it.
//! With a selection, the agent writes the selection again.
use super::*;

/// The mark of `ctrl-h`: where the arrow is, and the thread that answers.
#[derive(Clone, Debug)]
pub struct Say {
    pub path: PathBuf,
    /// The rendered row and char under which the arrow shows.
    pub row: usize,
    pub col: usize,
    pub key: String,
    pub rewrite: bool,
}

pub const SAY_HERE: &str = "Here is what I want to say here: ";
pub const SAY_DIFFERENTLY: &str = "I want to say differently here: ";

fn blank(lines: &[&str], i: usize) -> bool {
    lines.get(i).is_none_or(|l| l.trim().is_empty())
}

/// The paragraph at line `i`: its first and last line. `None` on a blank line.
pub fn paragraph(lines: &[&str], i: usize) -> Option<(usize, usize)> {
    if blank(lines, i) {
        return None;
    }
    let start = (0..=i).rev().take_while(|k| !blank(lines, *k)).last()?;
    let end = (i..lines.len()).take_while(|k| !blank(lines, *k)).last()?;
    Some((start, end))
}

/// The paragraph that ends before line `i`.
pub fn paragraph_before(lines: &[&str], i: usize) -> Option<(usize, usize)> {
    (0..i).rev().find(|k| !blank(lines, *k)).and_then(|k| paragraph(lines, k))
}

/// The paragraph that starts after line `i`.
pub fn paragraph_after(lines: &[&str], i: usize) -> Option<(usize, usize)> {
    (i + 1..lines.len()).find(|k| !blank(lines, *k)).and_then(|k| paragraph(lines, k))
}

fn quote(lines: &[&str], p: Option<(usize, usize)>) -> String {
    match p {
        Some((a, b)) => lines[a..=b].iter().map(|l| format!("> {l}\n")).collect(),
        None => "(none)\n".to_string(),
    }
}

/// What the agent gets with the idea: the place, the paragraphs around it, and the task.
pub fn say_context(rel: &str, lines: &[&str], (lo, hi): (usize, usize), selection: Option<&str>) -> String {
    match selection {
        None => {
            let here = paragraph(lines, lo);
            let end = here.map_or(lo, |(_, e)| e);
            let above = here.or_else(|| paragraph_before(lines, lo));
            format!(
                "The user marks a place in {rel}, after line {}.\nThe paragraph above the mark:\n{}The paragraph below the mark:\n{}\
                 Turn the idea of the user into text for this place. Connect it with the paragraph above and the paragraph below, and keep the style and the level of detail of the document. Propose it as one edit of {rel} that inserts the text at the mark. Change nothing else.",
                end + 1,
                quote(lines, above),
                quote(lines, paragraph_after(lines, end)),
            )
        }
        Some(sel) => {
            let start = paragraph(lines, lo).map_or(lo, |(s, _)| s);
            let end = paragraph(lines, hi).map_or(hi, |(_, e)| e);
            let selected: String = sel.lines().map(|l| format!("> {l}\n")).collect();
            format!(
                "The user selected this text in {rel}, about lines {}–{}:\n{selected}The paragraphs around it:\n{}The paragraph before them:\n{}The paragraph after them:\n{}\
                 Write the selected text again so that it says what the user wants. Connect it with the text around it, and keep the style and the level of detail of the document. Propose it as one edit of {rel} that replaces the selected text. Change nothing else.",
                lo + 1,
                hi + 1,
                quote(lines, Some((start, end))),
                quote(lines, paragraph_before(lines, start)),
                quote(lines, paragraph_after(lines, end)),
            )
        }
    }
}

impl App {
    /// `ctrl-h` in read mode: the arrow, the context, and the chat with the start of the sentence.
    pub(crate) fn start_say(&mut self) {
        let Some(path) = self.current.clone() else { return self.flash("open a note first") };
        let selection = self.read_selection();
        let (r1, r2) = self.selection_rows();
        let (row, col) = match selection {
            Some((_, (r, c))) => (r, c),
            None => (self.read_cursor, self.read_col),
        };
        let src_of = |r: usize| self.read_rows.iter().rposition(|x| *x <= r).unwrap_or(0);
        let (lo, hi) = if selection.is_some() { (src_of(r1), src_of(r2)) } else { (src_of(row), src_of(row)) };
        let text = self.note_text().unwrap_or_default();
        let lines: Vec<&str> = text.lines().collect();
        let picked = selection.map(|_| self.selection_text());
        let ctx = say_context(&self.rel(&path), &lines, (lo, hi), picked.as_deref());
        let rewrite = picked.is_some();
        self.read_anchor = None;
        self.open_panel();
        self.say = Some(Say { path, row, col, key: self.key(), rewrite });
        self.ask_context = Some(ctx);
        self.ask_about = if rewrite { "the selection" } else { "the mark" };
        self.input.take();
        self.input.type_str(if rewrite { SAY_DIFFERENTLY } else { SAY_HERE });
        self.focus = Focus::Claude;
    }
}

#[cfg(test)]
#[path = "say_tests.rs"]
mod tests;
