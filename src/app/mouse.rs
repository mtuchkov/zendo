//! The mouse: clicks, drags, the wheel, and pane widths.
use super::*;

impl App {
    pub(crate) fn region_at(&self, x: u16, y: u16) -> Option<(Region, Rect)> {
        self.regions.iter().copied().find(|(_, r)| r.contains(ratatui::layout::Position::new(x, y)))
    }

    /// Mouse: drag selects inside one panel, release copies, the wheel scrolls the panel under it.
    pub fn on_mouse(&mut self, m: MouseEvent) {
        if let Some(n) = self.cli.view {
            return self.mouse_cli(n, m);
        }
        let (x, y) = (m.column, m.row);
        if !matches!(m.kind, MouseEventKind::Moved) {
            self.last_input = Instant::now();
        }
        match m.kind {
            MouseEventKind::Down(MouseButton::Left) if self.start_drag(x, y) => {}
            MouseEventKind::Drag(MouseButton::Left) if self.drag.is_some() => self.drag_to(x),
            MouseEventKind::Up(MouseButton::Left) if self.drag.is_some() => {
                self.drag = None;
                self.state.save(&self.cfg.root);
            }
            MouseEventKind::Down(MouseButton::Left) => self.click(x, y),
            MouseEventKind::Drag(MouseButton::Left) => {
                if let Some(s) = self.sel.as_mut().filter(|s| !s.done) {
                    s.drag(x, y);
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                if let Some(s) = self.sel.as_mut() {
                    if s.is_empty() {
                        self.sel = None;
                    } else {
                        s.done = true;
                        self.copy_pending = true;
                        let note = s.region != Region::Panel;
                        if note {
                            self.interact(2);
                        }
                    }
                }
            }
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => self.wheel(x, y, m.kind == MouseEventKind::ScrollUp),
            _ => {}
        }
    }

    /// A click: a row of the tree opens, a click in a text starts a selection and moves the focus.
    fn click(&mut self, x: u16, y: u16) {
        self.sel = None;
        let hit = |r: &Rect| r.contains(ratatui::layout::Position::new(x, y));
        if let Some(n) = self.session_buttons.iter().find(|(r, _)| hit(r)).map(|(_, n)| *n) {
            return if n < SESSIONS { self.switch_session(n) } else { self.open_terminal(n - SESSIONS) };
        }
        if let Some(cmd) = self.play_buttons.iter().find(|(r, _)| hit(r)).map(|(_, c)| c.clone()) {
            return self.put_into_terminal(&cmd, false);
        }
        let row = self.tree_rows.iter().find(|(r, _)| r.contains(ratatui::layout::Position::new(x, y))).map(|(_, i)| *i);
        if let Some(i) = row {
            self.focus = Focus::Tree;
            self.tree.sel = i;
            self.tree_activate(false);
            return;
        }
        let Some((region, rect)) = self.region_at(x, y) else { return };
        self.sel = Some(Selection::start(region, rect, x, y));
        if matches!(self.mode, Mode::Read | Mode::Write) {
            self.focus = match region {
                Region::Panel => Focus::Claude,
                Region::Delta => Focus::Delta,
                Region::Note => Focus::Note,
            };
        }
        if region != Region::Panel {
            self.interact(1);
        }
        if region == Region::Note && self.mode == Mode::Read {
            self.read_cursor = self.scroll + (y - rect.y) as usize;
            let row = self.read_text.get(self.read_cursor).cloned().unwrap_or_default();
            let mut w = 0;
            let want = (x - rect.x) as usize;
            self.read_col = row.trim_end().chars().take_while(|c| {
                w += unicode_width::UnicodeWidthChar::width(*c).unwrap_or(0);
                w <= want
            }).count().min(row.trim_end().chars().count().saturating_sub(1));
            self.read_want = self.read_col;
            self.read_anchor = None;
        }
    }

    /// The wheel scrolls the pane under the pointer.
    fn wheel(&mut self, x: u16, y: u16, up: bool) {
        self.sel = None;
        let n = self.scroll_step();
        let ni = n as isize;
        if self.tree_rows.iter().any(|(r, _)| r.contains(ratatui::layout::Position::new(x, y))) {
            self.tree.scroll = if up { self.tree.scroll.saturating_sub(n) } else { self.tree.scroll + n };
            return;
        }
        match self.region_at(x, y).map(|(r, _)| r) {
            Some(Region::Panel) => {
                self.panel_scroll = if up { self.panel_scroll + n } else { self.panel_scroll.saturating_sub(n) };
            }
            Some(Region::Delta) => {
                self.delta_scroll = if up { self.delta_scroll.saturating_sub(n) } else { self.delta_scroll + n };
                self.interact(1);
            }
            Some(Region::Note) if self.mode == Mode::Write => self.editor.vertical(if up { -ni } else { ni }, self.text_width),
            Some(Region::Note) => {
                self.scroll = if up { self.scroll.saturating_sub(n) } else { self.scroll + n };
                self.read_cursor = if up { self.read_cursor.saturating_sub(n) } else { self.read_cursor + n };
                self.interact(1);
            }
            None => {}
        }
    }

    /// Lines for one scroll event: 3 for a wheel notch, 1 inside a fast run of events (a trackpad).
    pub(crate) fn scroll_step(&mut self) -> usize {
        let now = std::time::Instant::now();
        let fast = self.last_scroll.is_some_and(|t| now.duration_since(t) < std::time::Duration::from_millis(40));
        self.last_scroll = Some(now);
        if fast { 1 } else { 3 }
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
        let now = self.pane_rects.iter().find(|(p, _)| *p == pane).map(|(_, r)| r.width).unwrap_or(40);
        let w = (now as i32 + by).clamp(20, self.max_width(pane) as i32) as u16;
        self.state.widths.insert(key.into(), w);
        self.state.save(&self.cfg.root);
    }

    /// The widest a pane can get: the note keeps 50 columns, the other panes keep theirs.
    pub(crate) fn max_width(&self, pane: Focus) -> u16 {
        let total: u16 = self.pane_rects.iter().map(|(_, r)| r.width).sum();
        let others: u16 = self.pane_rects.iter().filter(|(p, _)| *p != pane && *p != Focus::Note).map(|(_, r)| r.width).sum();
        let note = if self.pane_rects.iter().any(|(p, _)| *p == Focus::Note) { 50 } else { 0 };
        total.saturating_sub(others + note).max(20)
    }

    /// A press on the edge between two panes starts a drag; returns true when it did.
    pub(crate) fn start_drag(&mut self, x: u16, y: u16) -> bool {
        for i in 1..self.pane_rects.len() {
            let (left, lr) = self.pane_rects[i - 1];
            let (right, rr) = self.pane_rects[i];
            if x == rr.x && y >= rr.y && y < rr.bottom() {
                // The note fills the rest: resize the other pane of the edge.
                self.drag = Some(if right == Focus::Note { (left, lr.x, true) } else { (right, rr.right(), false) });
                return true;
            }
        }
        false
    }

    pub(crate) fn drag_to(&mut self, x: u16) {
        let Some((pane, anchor, grows_right)) = self.drag else { return };
        let Some(key) = App::pane_key(pane) else { return };
        let w = if grows_right { x.saturating_sub(anchor) } else { anchor.saturating_sub(x) };
        let max = self.max_width(pane);
        self.state.widths.insert(key.into(), w.clamp(20, max));
    }
}
