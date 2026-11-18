//! The modes of the screen; the delta, the notes list, search and the theme picker.
use super::*;

/// What the screen shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Read,
    Write,
    Find,
    Theme,
    Help,
}

impl App {
    pub(crate) fn key_delta(&mut self, act: Option<&str>) {
        let page = self.frame.view_height().saturating_sub(3).max(1);
        match act {
            Some("close") => {
                self.work.close_delta();
                self.focus = Focus::Note;
            }
            Some("base") => self.switch_base(),
            Some("ask") => self.ask_about_delta(),
            Some("recap") => self.recap_now(),
            Some("layout") => self.toggle_layout(),
            Some("write") if self.mode == Mode::Read => self.start_write(false),
            Some("down") => self.work.scroll_delta(false, 1),
            Some("up") => self.work.scroll_delta(true, 1),
            Some("page_down") => self.work.scroll_delta(false, page),
            Some("page_up") => self.work.scroll_delta(true, page),
            Some("top") => self.work.delta_top(),
            Some("bottom") => self.work.delta_bottom(),
            _ => return,
        }
        if matches!(act, Some("down" | "up" | "page_down" | "page_up")) {
            self.interact(1);
        }
    }

    /// `o`: the tree in the Notes view, with the focus in it (FILES-76).
    pub fn open_notes(&mut self) {
        if !self.state.tree {
            self.toggle_tree();
        }
        self.show_tree_view(crate::docs::tree::View::Notes);
        self.focus = Focus::Tree;
    }

    /// A key in the theme list: the keys of `[theme]`; `esc` is fixed (PANES-48).
    pub(crate) fn key_theme(&mut self, k: KeyEvent) {
        if self.font_tip.asking {
            return self.key_font_question(k);
        }
        let n = THEMES.len();
        match (k.code, self.keys.local(crate::keys::Pane::Theme, &k)) {
            (_, Some("font")) => self.press_font_button(),
            (_, Some("down")) => self.theme = (self.theme + 1) % n,
            (_, Some("up")) => self.theme = (self.theme + n - 1) % n,
            (_, Some("keep")) => {
                self.state.theme = THEMES[self.theme].name.to_string();
                self.state.save(&self.cfg.root);
                self.mode = Mode::Read;
            }
            (KeyCode::Esc, _) | (_, Some("cancel")) => {
                self.theme = self.theme_before;
                self.mode = Mode::Read;
            }
            _ => {}
        }
    }
}
