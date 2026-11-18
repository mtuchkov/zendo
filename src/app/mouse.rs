//! The mouse: clicks, drags, the wheel, and pane widths.
use super::*;

impl App {
    pub(crate) fn region_at(&self, x: u16, y: u16) -> Option<(Region, Rect)> {
        self.frame.regions().iter().copied().find(|(_, r)| r.contains(ratatui::layout::Position::new(x, y)))
    }

    /// Mouse: drag selects inside one panel, release copies, the wheel scrolls the panel under it.
    /// A program in a terminal that takes the mouse gets it first (TERM-43, TERM-44).
    pub(super) fn mouse_in(&mut self, m: MouseEvent) {
        if self.hold.is_some() {
            return;
        }
        if !matches!(m.kind, MouseEventKind::Moved) {
            self.work.input_now();
            self.tab_run.end();
        }
        // The tip menu and the key float take the mouse: only their buttons count (TIPS-3,
        // REKEY-12, REKEY-25, REKEY-29).
        if self.tips.open || self.rekey.float().is_some() {
            if m.kind == MouseEventKind::Down(MouseButton::Left) {
                if self.tips.open { self.click_tips(m.column, m.row) } else { self.click_key_float(m.column, m.row) }
            }
            return;
        }
        if m.kind == MouseEventKind::Down(MouseButton::Left) && self.click_off_panel(m.column, m.row) {
            return;
        }
        // The float takes the mouse: only what happens over it counts.
        if let Some(fl) = self.writer.float() {
            if !fl.rect.contains(ratatui::layout::Position::new(m.column, m.row)) {
                return;
            }
        } else if self.mouse_to_program(m) {
            return;
        } else if let Some(n) = self.cli.view {
            return self.mouse_cli(n, m);
        }
        let (x, y) = (m.column, m.row);
        match m.kind {
            MouseEventKind::Down(MouseButton::Left) if self.start_drag(x, y) => {}
            MouseEventKind::Drag(MouseButton::Left) if self.pointer.grabbed().is_some() => self.drag_to(x),
            MouseEventKind::Up(MouseButton::Left) if self.pointer.grabbed().is_some() => {
                self.pointer.let_go();
                self.state.save(&self.cfg.root);
            }
            MouseEventKind::Down(MouseButton::Left) => {
                self.click(x, y);
                self.pointer.hold(m.modifiers);
            }
            MouseEventKind::Drag(MouseButton::Left) => self.drag_selection(x, y, m.modifiers),
            MouseEventKind::Up(MouseButton::Left) => self.end_selection(m.modifiers),
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => self.wheel(x, y, m.kind == MouseEventKind::ScrollUp),
            _ => {}
        }
    }

    /// A drag moves the end of the selection, until the button is up.
    pub(crate) fn drag_selection(&mut self, x: u16, y: u16, m: KeyModifiers) {
        self.pointer.drag(x, y, m);
    }

    /// The button is up: the selection is final and goes to the clipboard, or with alt to the chat;
    /// an empty one goes away.
    pub(crate) fn end_selection(&mut self, m: KeyModifiers) {
        let Some((region, to_chat)) = self.pointer.release(m) else { return };
        self.count_selection(region, to_chat);
        if region != Region::Panel {
            self.interact(2);
        }
    }

    /// A click: a row of the tree opens, a click in a text starts a selection and moves the focus.
    fn click(&mut self, x: u16, y: u16) {
        self.pointer.clear();
        let hit = |r: &Rect| r.contains(ratatui::layout::Position::new(x, y));
        if self.music.asking {
            return self.click_music_question(x, y);
        }
        if self.font_tip.asking {
            return self.click_font_question(x, y);
        }
        if self.writer.asks_external() {
            return self.click_external_question(x, y);
        }
        if self.mode == Mode::Theme && self.frame.font_button().is_some_and(|r| hit(&r)) {
            return self.press_font_button();
        }
        if self.click_key(x, y) {
            return;
        }
        self.focus_pane_at(x, y);
        if self.frame.music_button().is_some_and(|r| hit(&r)) {
            return self.toggle_music();
        }
        if self.frame.sound_button().is_some_and(|r| hit(&r)) {
            return self.toggle_sound();
        }
        if self.frame.spec_button().is_some_and(|r| hit(&r)) {
            self.did_by_mouse(crate::keys::Pane::Read, "to_spec");
            return self.note_to_spec();
        }
        if self.frame.upgrade_button().is_some_and(|r| hit(&r)) {
            self.did_by_mouse(crate::keys::Pane::Read, "upgrade");
            return self.upgrade();
        }
        if self.click_panel_button(&hit) {
            return;
        }
        if let Some(open) = self.frame.tree_fold().iter().find(|(r, _)| hit(r)).map(|(_, o)| *o) {
            return self.fold_all(open);
        }
        let row = self.frame.tree_rows().iter().find(|(r, _)| hit(r)).map(|(_, i)| *i);
        if let Some(i) = row {
            self.files.select_row(i);
            self.did_by_mouse(crate::keys::Pane::Tree, "preview");
            self.tree_activate(false);
            return;
        }
        let Some((region, rect)) = self.region_at(x, y) else { return };
        self.pointer.press(Some(Selection::start(region, rect, x, y)));
        let (dx, dy) = ((x - rect.x) as usize, (y - rect.y) as usize);
        if region == Region::Note && self.click_task(dy, dx) {
            self.pointer.clear();
            return self.interact(1);
        }
        match region {
            Region::Panel if self.focus == Focus::Claude && self.cli.pane.is_none() => {
                self.threads.click(dy, rect.height as usize, dx);
                self.click_step();
            }
            Region::Panel => {}
            Region::Note if self.mode == Mode::Read => self.place_reader(dy, dx),
            _ => self.interact(1),
        }
    }

    /// A click off a panel that stands over the screen closes it, as `esc` does: the help, the
    /// find panel, or the float under them (PANES-47). True when the click closed a panel.
    /// A button of the chat panel: a session, a new terminal, the background line (BG-3, BG-8), or
    /// the play button of a shell block. True when the click hit one.
    fn click_panel_button(&mut self, hit: &impl Fn(&Rect) -> bool) -> bool {
        if let Some(n) = self.frame.session_buttons().iter().find(|(r, _)| hit(r)).map(|(_, n)| *n) {
            self.count_session_button(n);
            if n < SESSIONS { self.switch_session(n) } else { self.open_terminal(n - SESSIONS) }
        } else if self.frame.new_terminal_button().is_some_and(|r| hit(&r)) {
            self.new_terminal();
        } else if let Some((_, pgid, stop)) = self.frame.bg_buttons().iter().find(|(r, ..)| hit(r)).copied() {
            if stop { self.stop_background(pgid) } else { self.show_background(pgid) }
        } else if let Some(cmd) = self.frame.play_buttons().iter().find(|(r, _)| hit(r)).map(|(_, c)| c.to_string()) {
            self.did_by_mouse(crate::keys::Pane::Global, "terminal");
            self.put_into_terminal(&cmd, false);
        } else {
            return false;
        }
        true
    }

    fn click_off_panel(&mut self, x: u16, y: u16) -> bool {
        let off = |r: Option<Rect>| r.is_some_and(|r| !r.contains(ratatui::layout::Position::new(x, y)));
        match self.mode {
            Mode::Help if off(self.frame.overlay()) => self.mode = self.back,
            Mode::Find if off(self.frame.overlay()) => self.close_find(),
            Mode::Help | Mode::Find => return false,
            _ if off(self.writer.float().map(|fl| fl.rect)) => self.close_float(),
            _ => return false,
        }
        true
    }

    /// A click in a pane moves the focus there, as `tab` does; in the chat pane the keys go to the
    /// input, and to the transcript only for a click in it (PANES-46).
    fn focus_pane_at(&mut self, x: u16, y: u16) {
        if !matches!(self.mode, Mode::Read | Mode::Write) {
            return;
        }
        let under = self.frame.pane_rects().iter().find(|(_, r)| r.contains(ratatui::layout::Position::new(x, y)));
        if let Some(&(pane, _)) = under {
            self.focus = pane;
            self.threads.transcript_mut().focus(false);
        }
    }

    /// A click in the note in read mode: the cursor goes to the char under the pointer.
    fn place_reader(&mut self, dy: usize, dx: usize) {
        self.interact(1);
        let at = self.reader.scroll() + dy;
        let row = self.frame.read_text().get(at).cloned().unwrap_or_default();
        self.reader.place(at, crate::text::select::col_at(&row, dx));
        self.reader.unselect();
    }

    /// The wheel scrolls the pane under the pointer.
    fn wheel(&mut self, x: u16, y: u16, up: bool) {
        self.pointer.clear();
        let n = self.pointer.scroll_step(Instant::now());
        let ni = n as isize;
        if self.frame.tree_rows().iter().any(|(r, _)| r.contains(ratatui::layout::Position::new(x, y))) {
            self.files.scroll_tree(up, n);
            return;
        }
        match self.region_at(x, y).map(|(r, _)| r) {
            // The wheel over a terminal in the chat pane scrolls through the lines that left it.
            Some(Region::Panel) if self.cli.pane.is_some() => self.cli.scroll(self.cli.pane.unwrap_or_default(), up),
            Some(Region::Panel) => {
                self.threads.scroll_by(up, n);
            }
            Some(Region::Delta) => {
                self.work.scroll_delta(up, n);
                self.interact(1);
            }
            Some(Region::Note) if self.mode == Mode::Write => self.writer.editor_mut().vertical(if up { -ni } else { ni }, self.frame.text_width()),
            Some(Region::Note) => {
                self.reader.wheel(up, n);
                self.interact(1);
            }
            None => {}
        }
    }

    pub fn pane_key(f: Focus) -> Option<&'static str> {
        match f {
            Focus::Tree => Some("tree"),
            Focus::Claude => Some("chat"),
            Focus::Delta => Some("delta"),
            Focus::Note => None,
        }
    }

    /// Change a pane width by `by` columns, from its width in the last frame.
    pub fn resize(&mut self, pane: Focus, by: i32) {
        let Some(key) = App::pane_key(pane) else { return };
        let now = self.frame.pane_rects().iter().find(|(p, _)| *p == pane).map(|(_, r)| r.width).unwrap_or(40);
        let w = (now as i32 + by).clamp(20, self.max_width(pane) as i32) as u16;
        self.state.widths.insert(key.into(), w);
        self.state.save(&self.cfg.root);
    }

    /// The widest a pane can get: the note keeps 50 columns, the other panes keep theirs.
    pub(crate) fn max_width(&self, pane: Focus) -> u16 {
        let total: u16 = self.frame.pane_rects().iter().map(|(_, r)| r.width).sum();
        let others: u16 = self.frame.pane_rects().iter().filter(|(p, _)| *p != pane && *p != Focus::Note).map(|(_, r)| r.width).sum();
        let note = if self.frame.pane_rects().iter().any(|(p, _)| *p == Focus::Note) { 50 } else { 0 };
        total.saturating_sub(others + note).max(20)
    }

    /// A press on the edge between two panes, or one column beside it, starts a drag; returns true
    /// when it did (PANES-10).
    pub(crate) fn start_drag(&mut self, x: u16, y: u16) -> bool {
        let hit = self.frame.pane_rects().windows(2).find(|w| x.abs_diff(w[1].1.x) <= 1 && y >= w[1].1.y && y < w[1].1.bottom());
        let Some(&[(left, lr), (right, rr)]) = hit else { return false };
        // The anchor moves with the grab, so the width changes only as far as the mouse moves.
        let shift = |a: u16| (a + x).saturating_sub(rr.x);
        // The note fills the rest: resize the other pane of the edge.
        self.pointer.grab(if right == Focus::Note { (left, shift(lr.x), true) } else { (right, shift(rr.right()), false) });
        true
    }

    pub(crate) fn drag_to(&mut self, x: u16) {
        let Some((pane, anchor, grows_right)) = self.pointer.grabbed() else { return };
        let Some(key) = App::pane_key(pane) else { return };
        let w = if grows_right { x.saturating_sub(anchor) } else { anchor.saturating_sub(x) };
        let max = self.max_width(pane);
        self.state.widths.insert(key.into(), w.clamp(20, max));
    }
}
