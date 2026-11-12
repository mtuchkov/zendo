//! "Here is what I want to say": `ctrl-shift-s` marks a place in the note with an arrow, and the chat asks
//! the agent to turn the idea of the user into text that connects with the paragraphs around it.
//! With a selection, the agent writes the selection again.
use super::*;

/// The mark of `ctrl-shift-s`: where the arrow is, and the thread that answers.
#[derive(Clone, Debug)]
pub struct Say {
    pub path: PathBuf,
    /// The source line and char of the mark: the arrow shows left of its row.
    pub line: usize,
    pub col: usize,
    pub key: String,
    pub rewrite: bool,
}

/// The question when the user sends the chip of say here alone.
pub const SAY_HERE: &str = "Write here what fits between the paragraph above and the paragraph below.";
pub const SAY_DIFFERENTLY: &str = "Write the selected text again, clearer and shorter.";

/// The chip of say here: what the agent does, and the line of the mark, as `write here :386`.
pub fn say_label(rewrite: bool, line: usize) -> String {
    format!("{} :{}", if rewrite { "rewrite this" } else { "write here" }, line + 1)
}

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

/// Where the cursor is, for the agent: the line and the column, from 1.
pub fn cursor_place(rel: &str, (line, col): (usize, usize)) -> String {
    format!("The cursor is at line {}, column {} of {rel}.", line + 1, col + 1)
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

/// The mark (line, char), the first and last selected line, the selected text, and the source lines.
type Place = ((usize, usize), (usize, usize), Option<String>, Vec<String>);

impl App {
    /// `ctrl-shift-s` in read mode or write mode: the arrow, the context, and the chat with the
    /// start of the sentence.
    pub(crate) fn start_say(&mut self) {
        let Some(path) = self.current.clone() else { return self.flash("open a note first") };
        let (mark, range, picked, lines) = if self.mode == Mode::Write { self.say_in_editor() } else { self.say_in_read() };
        let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let rel = self.rel(&path);
        let ctx = format!("{}\n{}", cursor_place(&rel, mark), say_context(&rel, &refs, range, picked.as_deref()));
        let rewrite = picked.is_some();
        self.read_anchor = None;
        self.open_panel();
        self.say = Some(Say { path, line: mark.0, col: mark.1, key: self.key(), rewrite });
        self.ask_about = if rewrite { "the selection" } else { "the mark" };
        self.ask_default = Some((if rewrite { SAY_DIFFERENTLY } else { SAY_HERE }).to_string());
        self.input.take();
        self.chips.clear();
        // The chip brings the place and the task; the user writes the idea after it.
        self.add_chip(say_label(rewrite, mark.0), Some(format!("{ctx}\nThe idea of the user is the question.")), None, false);
        self.focus = Focus::Claude;
    }

    /// The mark, the source lines around it, the selected text, and the source: from the read cursor.
    fn say_in_read(&self) -> Place {
        let lines: Vec<String> = self.note_text().unwrap_or_default().lines().map(str::to_string).collect();
        let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let to_src = |p| cursor_map::shown_to_source(&self.read_text, &self.read_rows, &refs, p);
        let (start, end) = match self.read_selection() {
            Some((a, b)) => (to_src(a), to_src(b)),
            None => {
                let p = to_src((self.read_cursor, self.read_col));
                (p, p)
            }
        };
        let picked = self.read_selection().map(|_| self.selection_text());
        (start, (start.0, end.0), picked, lines)
    }

    /// The same from the editor in write mode.
    fn say_in_editor(&self) -> Place {
        let e = &self.editor;
        match e.selection() {
            Some((a, b)) => (a, (a.0, b.0), e.selected_text(), e.lines.clone()),
            None => ((e.row, e.col), (e.row, e.row), None, e.lines.clone()),
        }
    }
}

#[cfg(test)]
#[path = "say_tests.rs"]
mod tests;
