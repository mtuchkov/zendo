//! The rows of the transcript, frozen while it has the keys or a selection (PANES-51, ADR 0026):
//! the cursor and the selection are row numbers, and frozen rows do not move under them.
use super::Transcript;
use ratatui::text::Line;

/// The rows of the transcript as the draw makes them, with what each row holds.
#[derive(Clone, Default)]
pub struct Rows {
    pub lines: Vec<Line<'static>>,
    /// The shell blocks: their first row, the row after them, their command.
    pub blocks: Vec<(usize, usize, String)>,
    /// The steps: their row, their item, and the step in it.
    pub steps: Vec<(usize, usize, Option<usize>)>,
    /// The chips of the messages of the user: the row, the first column, the width, the text.
    pub chips: Vec<(usize, u16, u16, String)>,
}

/// Why the next draw freezes the rows again.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refreeze {
    /// `bottom`: the new rows come in, and the cursor goes to the last row with text (PANES-54).
    End,
    /// `o`: a step opens or closes; the cursor stays, and the selection goes (PANES-56).
    Step,
}

#[derive(Default)]
pub(super) struct Freeze {
    rows: Option<Rows>,
    /// The thread and the width of the last draw.
    last: Option<(String, usize)>,
    again: Option<Refreeze>,
}

impl Freeze {
    pub(super) fn refreeze(&mut self, why: Refreeze) {
        self.again = Some(why);
    }

    /// The rows to draw of the thread `key` at `width`, and the number of new rows below them.
    /// `hold`: the transcript has the keys or a selection.
    pub(super) fn view(&mut self, (key, width): (&str, usize), hold: bool, live: Rows, tr: &mut Transcript) -> (Rows, usize) {
        // Another thread or a new width: the rows and their numbers are new (PANES-55, PANES-56).
        let moved = self.last.as_ref().is_some_and(|(k, w)| k != key || *w != width);
        self.last = Some((key.to_string(), width));
        let again = self.again.take();
        let (rows, new) = match self.rows.take() {
            Some(old) if hold && !moved && again.is_none() => {
                let new = live.lines.len().saturating_sub(old.lines.len());
                (old, new)
            }
            _ => (live, 0),
        };
        if hold {
            self.rows = Some(rows.clone());
        }
        tr.show_rows(rows.lines.iter().map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect()).collect());
        tr.show_blocks(rows.blocks.clone());
        tr.show_steps(rows.steps.clone());
        match (moved, again) {
            (true, _) => tr.to_end(),
            (false, Some(Refreeze::End)) => tr.to_last_text(),
            (false, Some(Refreeze::Step)) => tr.unselect(),
            (false, None) => {}
        }
        (rows, new)
    }
}

#[cfg(test)]
#[path = "frozen_tests.rs"]
mod tests;
