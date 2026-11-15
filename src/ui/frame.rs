//! What a draw leaves behind for the mouse and the keys: the panes, the buttons, and the rows of
//! the text. `ui` builds a frame in each draw; the app stores it and only reads it.
use crate::app::Focus;
use crate::app::write_rows::RowSpan;
use crate::text::select::Region;
use ratatui::layout::Rect;

/// The layout of the last draw. Only `ui` writes the fields; the app reads them with the methods.
#[derive(Default)]
pub struct Frame {
    pub(super) visible: Vec<Focus>,
    pub(super) pane_rects: Vec<(Focus, Rect)>,
    pub(super) regions: Vec<(Region, Rect)>,
    pub(super) tree_rows: Vec<(Rect, usize)>,
    pub(super) session_buttons: Vec<(Rect, usize)>,
    pub(super) play_buttons: Vec<(Rect, String)>,
    pub(super) read_text: Vec<String>,
    pub(super) read_rows: Vec<usize>,
    pub(super) write_rows: Vec<RowSpan>,
    pub(super) view_height: usize,
    pub(super) text_width: usize,
}

impl Frame {
    /// The frame before the first draw: the height of the view and the width of the text.
    pub(crate) fn sized(view_height: usize, text_width: usize) -> Self {
        Self { view_height, text_width, ..Self::default() }
    }

    /// The panes, left to right.
    pub fn visible(&self) -> &[Focus] {
        &self.visible
    }

    /// The panes with their areas, for resizing.
    pub fn pane_rects(&self) -> &[(Focus, Rect)] {
        &self.pane_rects
    }

    /// The panels on the screen, for the mouse.
    pub fn regions(&self) -> &[(Region, Rect)] {
        &self.regions
    }

    /// The tree rows on the screen, for mouse clicks.
    pub fn tree_rows(&self) -> &[(Rect, usize)] {
        &self.tree_rows
    }

    /// The buttons of the agent sessions, for mouse clicks.
    pub fn session_buttons(&self) -> &[(Rect, usize)] {
        &self.session_buttons
    }

    /// The play buttons of the shell blocks, and their commands (TERM-10).
    pub fn play_buttons(&self) -> &[(Rect, String)] {
        &self.play_buttons
    }

    /// The rendered rows of the read view, as plain text.
    pub fn read_text(&self) -> &[String] {
        &self.read_text
    }

    /// The row of each source line in the read view.
    pub fn read_rows(&self) -> &[usize] {
        &self.read_rows
    }

    /// Write mode: the rows on the screen as spans of the source.
    pub fn write_rows(&self) -> &[RowSpan] {
        &self.write_rows
    }

    /// The height of the main area, for keys that move by screen.
    pub fn view_height(&self) -> usize {
        self.view_height
    }

    /// The width of the text column.
    pub fn text_width(&self) -> usize {
        self.text_width
    }

    /// The float and the screen behind it each keep their own rows of write mode.
    pub(crate) fn swap_write_rows(&mut self, other: &mut Vec<RowSpan>) {
        std::mem::swap(&mut self.write_rows, other);
    }
}

#[cfg(test)]
#[path = "frame_tests.rs"]
mod tests;
