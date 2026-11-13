//! The mouse over a terminal or a Claude Code session, in the chat pane or on the full screen
//! (docs/zendo/terminal/terminal-mouse.md).
use super::*;
use crate::session::{keys, mouse};
use ratatui::layout::Position;

impl App {
    /// The session on the screen, and the area of its screen in the last frame.
    fn session_area(&self) -> Option<(usize, Rect)> {
        let n = self.cli.view.or(self.cli.pane)?;
        self.regions.iter().find(|(region, _)| *region == Region::Panel).map(|(_, rect)| (n, *rect))
    }

    /// Give the event to the program of the session under the pointer: a program that reports the
    /// mouse gets the event (TERM-43), and the wheel over a program on the alternate screen is
    /// arrow keys (TERM-44). False: the program does not take the event, it is for zendo.
    pub(crate) fn mouse_to_program(&mut self, m: MouseEvent) -> bool {
        let Some((n, area)) = self.session_area() else { return false };
        // The program that got the press gets the drag and the release, also outside its screen.
        let pressed = self.cli.pressed == Some(n);
        if matches!(m.kind, MouseEventKind::Up(_)) {
            self.cli.pressed = None;
        }
        if !pressed && !area.contains(Position::new(m.column, m.row)) {
            return false;
        }
        let modes = |s: &vt100::Screen| (s.mouse_protocol_mode(), s.mouse_protocol_encoding(), s.alternate_screen(), s.application_cursor());
        let Some((mode, encoding, alternate, app_cursor)) = self.cli.with_screen(n, modes) else { return false };
        let column = m.column.clamp(area.x, area.right().saturating_sub(1)) - area.x;
        let row = m.row.clamp(area.y, area.bottom().saturating_sub(1)) - area.y;
        let bytes = mouse::encode(&MouseEvent { column, row, ..m }, mode, encoding);
        if !bytes.is_empty() {
            if matches!(m.kind, MouseEventKind::Down(_)) {
                self.press_in_session(n);
            }
            self.cli.write(n, &bytes);
            return true;
        }
        let key = match m.kind {
            MouseEventKind::ScrollUp => KeyCode::Up,
            MouseEventKind::ScrollDown => KeyCode::Down,
            _ => return false,
        };
        if !alternate || mode != vt100::MouseProtocolMode::None {
            return false;
        }
        let lines = self.scroll_step();
        self.cli.write(n, &keys::encode(&KeyEvent::new(key, KeyModifiers::NONE), app_cursor).repeat(lines));
        true
    }

    /// The program of session `n` got the press of a button: it has the mouse until the release,
    /// and a terminal in the chat pane gets the keys, as after a click in the transcript (TERM-47).
    fn press_in_session(&mut self, n: usize) {
        self.cli.pressed = Some(n);
        self.sel = None;
        if self.cli.view.is_none() && matches!(self.mode, Mode::Read | Mode::Write) {
            self.focus = Focus::Claude;
        }
    }

    /// The mouse on the full screen of a session whose program does not take it: the wheel scrolls
    /// through the lines that left the screen (TERM-45), and a drag selects text (TERM-46).
    pub(crate) fn mouse_cli(&mut self, n: usize, m: MouseEvent) {
        let (x, y) = (m.column, m.row);
        match m.kind {
            MouseEventKind::ScrollUp | MouseEventKind::ScrollDown => {
                self.sel = None;
                self.cli.scroll(n, m.kind == MouseEventKind::ScrollUp);
            }
            MouseEventKind::Down(MouseButton::Left) => self.sel = self.region_at(x, y).map(|(region, rect)| Selection::start(region, rect, x, y)),
            MouseEventKind::Drag(MouseButton::Left) => self.drag_selection(x, y),
            MouseEventKind::Up(MouseButton::Left) => self.end_selection(),
            _ => {}
        }
    }
}
