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
    /// `[+]` and `[-]` in the head of the tree: true opens every folder (FILES-57).
    pub(super) tree_fold: Vec<(Rect, bool)>,
    /// The letters of the view strip of the tree, or the names of its hint (FILES-73).
    pub(super) view_buttons: Vec<(Rect, crate::docs::tree::View)>,
    pub(super) session_buttons: Vec<(Rect, usize)>,
    pub(super) play_buttons: Vec<(Rect, String)>,
    /// The names and the `×` of the background line, and their process groups (BG-3, BG-8).
    pub(super) bg_buttons: Vec<(Rect, u32, bool)>,
    /// `⏵ 2 running` of the background line: it opens and closes the list (BG-11).
    pub(super) bg_toggle: Option<Rect>,
    /// `[+]` under the terminal buttons: a new `[>]` terminal (TERM-53).
    pub(super) new_terminal_button: Option<Rect>,
    pub(super) read_text: Vec<String>,
    pub(super) read_rows: Vec<usize>,
    pub(super) write_rows: Vec<RowSpan>,
    pub(super) view_height: usize,
    pub(super) text_width: usize,
    pub(super) word_limit: usize,
    pub(super) word_count: usize,
    pub(super) over_limit: Option<usize>,
    pub(super) input_width: usize,
    /// The chat input, its chips, the line of the open document, and the chip window.
    pub(super) input_rect: Option<Rect>,
    pub(super) chip_rects: Vec<(Rect, String)>,
    pub(super) doc_line: Option<Rect>,
    pub(super) chip_view_rect: Option<Rect>,
    pub(super) transcript_chips: Vec<(Rect, String)>,
    pub(super) sound_button: Option<Rect>,
    pub(super) music_button: Option<Rect>,
    pub(super) spec_button: Option<Rect>,
    pub(super) upgrade_button: Option<Rect>,
    /// `[.*]` of the find panel or of the filter of the tree (FIND-44, FILES-67).
    pub(super) find_regex: Option<Rect>,
    pub(super) music_answers: Option<(Rect, Rect)>,
    /// The font list of the theme picker, and the fonts of the open list (FONT-36).
    pub(super) font_button: Option<Rect>,
    pub(super) font_items: Vec<Rect>,
    /// The two answers of the question before the undo of an external change (UNDO-12).
    pub(super) external_answers: Option<(Rect, Rect)>,
    /// The help or the find panel over the screen, for a click off it (PANES-47).
    pub(super) overlay: Option<Rect>,
    /// The option and `[OK]` of the tip menu (TIPS-3, TIPS-5).
    pub(super) tip_buttons: Option<(Rect, Rect)>,
    /// The keys of the status bar and of the help, for a click that changes one (REKEY-1).
    pub(super) key_targets: Vec<(Rect, crate::app::rekey::KeyTarget)>,
    /// The choices, `[remove]` and the suggested keys of the key float (REKEY-12, REKEY-25).
    pub(super) float_buttons: Vec<(Rect, crate::app::rekey::FloatButton)>,
}

impl Frame {
    /// The frame before the first draw: the height of the view and the width of the text.
    pub(crate) fn sized(view_height: usize, text_width: usize, word_limit: usize) -> Self {
        Self { view_height, text_width, word_limit, input_width: 40, ..Self::default() }
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

    /// `[+]` and `[-]` in the head of the tree, for mouse clicks: true opens every folder.
    pub fn tree_fold(&self) -> &[(Rect, bool)] {
        &self.tree_fold
    }

    /// The letters of the view strip or the names of its hint, for mouse clicks.
    pub fn view_buttons(&self) -> &[(Rect, crate::docs::tree::View)] {
        &self.view_buttons
    }

    /// The buttons of the agent sessions, for mouse clicks.
    pub fn session_buttons(&self) -> &[(Rect, usize)] {
        &self.session_buttons
    }

    /// `[+]` under the terminal buttons, for mouse clicks (TERM-54).
    pub fn new_terminal_button(&self) -> Option<Rect> {
        self.new_terminal_button
    }

    /// The play buttons of the shell blocks, and their commands (TERM-10).
    pub fn play_buttons(&self) -> &[(Rect, String)] {
        &self.play_buttons
    }

    /// Forget the buttons of the panes of the last draw: the sessions, the new terminal, the play
    /// buttons, the background line, and the fold of the tree.
    pub(super) fn clear_pane_buttons(&mut self) {
        self.session_buttons.clear();
        self.play_buttons.clear();
        self.bg_buttons.clear();
        self.bg_toggle = None;
        self.new_terminal_button = None;
        self.tree_fold.clear();
    }

    /// The names and the `×` of the background line: the place, the process group, and true for `×`.
    pub fn bg_buttons(&self) -> &[(Rect, u32, bool)] {
        &self.bg_buttons
    }

    /// `⏵ 2 running` of the background line, which opens and closes the list (BG-11).
    pub fn bg_toggle(&self) -> Option<Rect> {
        self.bg_toggle
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

    /// The width of the chat input, for up and down moves in it.
    pub fn input_width(&self) -> usize {
        self.input_width
    }

    /// The places for the mouse that each draw of the panes sets again.
    pub(super) fn clear_places(&mut self) {
        self.regions.clear();
        self.clear_pane_buttons();
        (self.input_rect, self.doc_line, self.chip_view_rect) = (None, None, None);
        self.chip_rects.clear();
        self.transcript_chips.clear();
    }

    /// The chips of the messages in the transcript on the screen, with the text in their marks.
    pub fn transcript_chips(&self) -> &[(Rect, String)] {
        &self.transcript_chips
    }

    pub fn input_rect(&self) -> Option<Rect> {
        self.input_rect
    }

    /// The chips of the chat input on the screen, with their labels.
    pub fn chip_rects(&self) -> &[(Rect, String)] {
        &self.chip_rects
    }

    pub fn doc_line(&self) -> Option<Rect> {
        self.doc_line
    }

    pub fn chip_view_rect(&self) -> Option<Rect> {
        self.chip_view_rect
    }

    /// The sound button, for the mouse.
    pub fn sound_button(&self) -> Option<Rect> {
        self.sound_button
    }

    /// The music button, for the mouse.
    /// `[ → spec ]` in the head of a note, for mouse clicks (SPEC-1).
    pub fn spec_button(&self) -> Option<Rect> {
        self.spec_button
    }

    pub fn music_button(&self) -> Option<Rect> {
        self.music_button
    }

    /// The upgrade button, for the mouse (CMD-11).
    /// `[.*]` on the screen, for mouse clicks.
    pub fn find_regex(&self) -> Option<Rect> {
        self.find_regex
    }

    pub fn upgrade_button(&self) -> Option<Rect> {
        self.upgrade_button
    }

    /// The help or the find panel that stands over the screen, for a click off it (PANES-47).
    pub fn overlay(&self) -> Option<Rect> {
        self.overlay
    }

    /// The two answers of the music question: keep on, turn off.
    pub fn music_answers(&self) -> Option<(Rect, Rect)> {
        self.music_answers
    }

    /// The font list of the theme picker.
    pub fn font_button(&self) -> Option<Rect> {
        self.font_button
    }

    /// The two answers of the question before the undo of an external change: undo it, keep it.
    pub fn external_answers(&self) -> Option<(Rect, Rect)> {
        self.external_answers
    }

    /// The fonts of the open font list, in its order.
    pub fn font_items(&self) -> &[Rect] {
        &self.font_items
    }

    /// The word limit of the document on the screen.
    pub fn word_limit(&self) -> usize {
        self.word_limit
    }

    /// The words of the document on the screen.
    pub fn word_count(&self) -> usize {
        self.word_count
    }

    /// The row of the word after the limit.
    pub fn over_limit(&self) -> Option<usize> {
        self.over_limit
    }

    /// Before a draw: no button is on the screen yet.
    pub(super) fn clear_buttons(&mut self) {
        (self.sound_button, self.music_button, self.overlay, self.spec_button, self.upgrade_button) = (None, None, None, None, None);
        self.key_targets.clear();
        self.float_buttons.clear();
        self.tip_buttons = None;
        self.find_regex = None;
    }

    /// The option and `[OK]` of the tip menu.
    pub fn tip_buttons(&self) -> Option<(Rect, Rect)> {
        self.tip_buttons
    }

    /// The keys on the screen that a click can change (REKEY-1).
    pub fn key_targets(&self) -> &[(Rect, crate::app::rekey::KeyTarget)] {
        &self.key_targets
    }

    /// The buttons of the key float.
    pub fn float_buttons(&self) -> &[(Rect, crate::app::rekey::FloatButton)] {
        &self.float_buttons
    }

    /// The float and the screen behind it each keep their own rows of write mode.
    pub(crate) fn swap_write_rows(&mut self, other: &mut Vec<RowSpan>) {
        std::mem::swap(&mut self.write_rows, other);
    }
}

#[cfg(test)]
#[path = "frame_tests.rs"]
mod tests;
