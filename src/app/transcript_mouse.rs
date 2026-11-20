//! The mouse in the chat transcript: a drag selects its rows, also out of view, and a drag at the
//! edge of the pane scrolls it (PANES-50, PANES-52, ADR 0026).
use super::*;

impl App {
    /// A click in the transcript: the keys go there, with the cursor under the pointer; a drag
    /// from here selects its rows (PANES-46, PANES-50).
    pub(crate) fn click_rows(&mut self, y: usize, h: usize, x: usize) {
        self.threads.click(y, h, x);
        self.pointer.in_rows();
    }

    /// A drag that started in the transcript: the selection grows to the char under the pointer,
    /// or to the top or the bottom row of the view when the pointer is beyond it.
    pub(crate) fn drag_rows(&mut self, x: u16, y: u16) {
        let Some(s) = self.pointer.sel().filter(|_| self.pointer.rows()) else { return };
        let r = s.rect;
        let dy = y.saturating_sub(r.y).min(r.height.saturating_sub(1));
        self.threads.transcript_mut().set_free(s.free);
        self.threads.drag(dy as usize, r.height as usize, x.saturating_sub(r.x) as usize);
    }

    /// The direction of the edge scroll: up while a drag in the transcript holds the pointer on
    /// the top row of the pane or above it, down on the bottom row or below it.
    pub fn edge_drag(&self) -> Option<isize> {
        let s = self.pointer.sel().filter(|_| self.pointer.rows() && self.pointer.dragging())?;
        let (_, y) = self.pointer.at();
        match () {
            _ if y <= s.rect.y => Some(-1),
            _ if y + 1 >= s.rect.bottom() => Some(1),
            _ => None,
        }
    }

    /// A tick of the edge scroll: the selection grows by a row, and the view follows it (PANES-52).
    pub fn edge_scroll(&mut self) {
        if let Some(by) = self.edge_drag() {
            self.threads.transcript_mut().move_rows(by);
        }
    }
}
