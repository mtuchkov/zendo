//! The font tip of the theme picker (docs/zendo/fonts-tip.md): the tip while a recommended font
//! is missing, the font button, the question, and the request to the agent.
use super::*;
use crate::setup::fonts;

/// The state of the font tip.
pub struct FontTip {
    /// The folders where fonts are on the machine; a test sets its own.
    pub dirs: Vec<PathBuf>,
    /// A recommended font is not on the machine; zendo looks when the picker opens.
    pub missing: bool,
    /// The question whether zendo may ask the agent is open (FONT-14).
    pub asking: bool,
}

impl Default for FontTip {
    fn default() -> Self {
        Self { dirs: fonts::search_dirs(), missing: false, asking: false }
    }
}

/// The font button while a font is missing, and while all are on the machine.
pub const ASK_BUTTON: &str = "[ ask the agent · a ]";
pub const CHANGE_BUTTON: &str = "[ change the font · a ]";

impl App {
    /// The picker opens: zendo looks whether a recommended font is missing (FONT-13).
    pub(crate) fn look_at_fonts(&mut self) {
        self.font_tip.missing = !fonts::missing(fonts::FONTS, &self.font_tip.dirs).is_empty();
    }

    /// The font button of the picker; none without a connected agent (FONT-16, FONT-21).
    pub fn font_button(&self) -> Option<&'static str> {
        let agent = self.link.ready() && !self.cfg.offline;
        agent.then_some(if self.font_tip.missing { ASK_BUTTON } else { CHANGE_BUTTON })
    }

    /// `a` or a click on the font button: the question opens (FONT-14).
    pub(super) fn press_font_button(&mut self) {
        self.font_tip.asking = self.font_button().is_some();
    }

    /// A key while the question is open: `y` or enter asks the agent, `n` or esc does not.
    pub(super) fn key_font_question(&mut self, k: KeyEvent) {
        match k.code {
            KeyCode::Char('y') | KeyCode::Enter => self.answer_fonts(true),
            KeyCode::Char('n') | KeyCode::Esc => self.answer_fonts(false),
            _ => {}
        }
    }

    /// A click while the question is open: on one of its answers.
    pub(super) fn click_font_question(&mut self, x: u16, y: u16) {
        let Some((yes, no)) = self.frame.font_answers() else { return };
        let hit = |r: Rect| r.contains(ratatui::layout::Position::new(x, y));
        if hit(yes) || hit(no) {
            self.answer_fonts(hit(yes));
        }
    }

    /// The answer: only on yes, the picker closes and the request goes to the agent of the
    /// current session (FONT-5, FONT-15).
    pub fn answer_fonts(&mut self, yes: bool) {
        self.font_tip.asking = false;
        if !yes {
            return;
        }
        self.theme = self.theme_before;
        self.mode = Mode::Read;
        self.open_panel();
        self.focus = Focus::Claude;
        // No file goes with the request: it is about the machine, not about a note.
        self.ask(request(self.font_tip.missing), NEW_KEY.to_string());
    }
}

/// The request to the agent: get the fonts when one is missing, then set the font of the
/// terminal, and change the config only after the user agrees (FONT-6, FONT-15).
pub fn request(missing: bool) -> String {
    let names: Vec<&str> = fonts::FONTS.iter().map(|f| f.name).collect();
    let get = match missing {
        true => "Run `zendo setup fonts`: it downloads the fonts into my font folder and asks nothing.",
        false => "The fonts are already on this machine.",
    };
    format!(
        "Help me set the font of my terminal to one of the fonts that zendo recommends: {}.\n\
         1. {get}\n\
         2. Find out which terminal I use and where it keeps its font setting.\n\
         3. Ask me which of these fonts to use.\n\
         4. Show me the change of the terminal config, and change it only after I agree. Change nothing else.",
        names.join(", ")
    )
}
