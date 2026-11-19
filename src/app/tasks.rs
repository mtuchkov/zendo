//! A click on the box of a task checks it, or opens it again, in read mode and in write mode
//! (TEXT-14).
use super::*;
use crate::text::markdown::{task_box, toggle_task};

impl App {
    /// A click at `dx`, `dy` of the note: true when it hit the box of a task and changed it.
    pub(crate) fn click_task(&mut self, dy: usize, dx: usize) -> bool {
        let at = self.reader.scroll() + dy;
        if self.mode == Mode::Write {
            return self.click_task_in_write(at, dx);
        }
        let row = self.frame.read_text().get(at).cloned().unwrap_or_default();
        let under = row.chars().nth(crate::text::select::col_at(&row, dx));
        if !under.is_some_and(|c| c == crate::text::markdown::TASK_OPEN || c == crate::text::markdown::TASK_DONE) {
            return false;
        }
        let (Some(path), Some(text)) = (self.files.current().cloned(), self.note_text()) else { return false };
        let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
        let hit = (0..lines.len()).find(|i| self.frame.read_rows().get(*i) == Some(&at) && task_box(&lines[*i]).is_some());
        let Some(i) = hit else { return false };
        lines[i] = toggle_task(&lines[i]).unwrap_or_default();
        // The box goes to the file on a worker (IO-1); the notes read it again after.
        let body = format!("{}\n", lines.join("\n").trim_end());
        let file = path.clone();
        self.in_background_with(
            move || crate::docs::notes::save(&file, &body).map_err(|e| e.to_string()),
            move |a, done| match done {
                Ok(()) => a.files.saved(path),
                Err(e) => a.flash(format!("could not save: {e}")),
            },
        );
        true
    }

    /// Write mode: the box is where the row of the task starts; on the cursor row the source shows,
    /// and the box is the `[ ]` after the bullet.
    fn click_task_in_write(&mut self, at: usize, dx: usize) -> bool {
        let spans = self.frame.write_rows();
        let Some(k) = spans.iter().position(|s| s.row == at) else { return false };
        let line = spans[k].chars.first().map_or(spans[k].end.0, |c| c.0.0);
        let first = !spans[..k].iter().any(|s| s.chars.first().map_or(s.end.0, |c| c.0.0) == line);
        let e = self.writer.editor();
        let Some(text) = e.lines.get(line).filter(|l| first && task_box(l).is_some()) else { return false };
        let indent = text.len() - text.trim_start().len();
        let shown = indent.min(8);
        let on_box = if line == e.row { (indent + 2..indent + 5).contains(&dx) } else { (shown..shown + 2).contains(&dx) };
        if !on_box {
            return false;
        }
        let e = self.writer.editor_mut();
        e.lines[line] = toggle_task(&e.lines[line]).unwrap_or_default();
        e.dirty = true;
        self.writer.touch();
        true
    }
}
