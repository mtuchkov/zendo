//! The font list of the theme picker (docs/zendo/fonts-tip.md): the recommended fonts on the
//! machine, set in the terminal by the recipe of the terminal, without the agent.
use super::*;
use crate::setup::fonts::{self, Font, recipe};

/// The state of the font list.
pub struct FontTip {
    /// The folders where fonts are on the machine; a test sets its own.
    pub dirs: Vec<PathBuf>,
    /// The file of the recipes; a test sets its own.
    pub recipes: PathBuf,
    /// The name of the current terminal (FONT-42).
    pub terminal: String,
    /// The recommended fonts on the machine; zendo looks when the picker opens.
    pub installed: Vec<&'static Font<'static>>,
    /// A recommended font is not on the machine.
    pub missing: bool,
    /// The recipe of the terminal is there and passes its check (FONT-34).
    pub works: bool,
    /// The font that zendo set last, from the recipe.
    pub current: Option<String>,
    /// The open list, and its selected font (FONT-36).
    pub list: Option<usize>,
}

impl Default for FontTip {
    fn default() -> Self {
        let (dirs, recipes, terminal) = (fonts::search_dirs(), recipe::path(), recipe::current_terminal());
        Self { dirs, recipes, terminal, installed: Vec::new(), missing: false, works: false, current: None, list: None }
    }
}

impl FontTip {
    /// The font list shows: a working recipe and a recommended font on the machine (FONT-34).
    pub fn shows_list(&self) -> bool {
        self.works && !self.installed.is_empty()
    }
}

impl App {
    /// The picker opens: the fonts on the machine and the recipe of the terminal.
    pub(crate) fn look_at_fonts(&mut self) {
        let tip = &mut self.font_tip;
        let missing = fonts::missing(fonts::FONTS, &tip.dirs);
        tip.missing = !missing.is_empty();
        tip.installed = fonts::FONTS.iter().filter(|f| !missing.iter().any(|m| m.name == f.name)).collect();
        let recipes = recipe::Recipes::load(&tip.recipes).unwrap_or_default();
        let own = recipes.get(&tip.terminal);
        tip.works = own.is_some_and(|r| r.check(&recipe::on_path).is_ok());
        tip.current = own.and_then(|r| r.font.clone());
        tip.list = None;
    }

    /// `f` or a click on the font list: the list opens at the current font (FONT-36).
    pub(super) fn open_font_list(&mut self) {
        let tip = &mut self.font_tip;
        if tip.shows_list() {
            let at = tip.installed.iter().position(|f| Some(f.family) == tip.current.as_deref());
            tip.list = Some(at.unwrap_or(0));
        }
    }

    /// A key while the font list is open: the keys of the theme picker move, keep and close it.
    pub(super) fn key_font_list(&mut self, k: KeyEvent, i: usize) {
        let n = self.font_tip.installed.len().max(1);
        match (k.code, self.keys.local(crate::keys::Pane::Theme, &k)) {
            (_, Some("down")) => self.font_tip.list = Some((i + 1) % n),
            (_, Some("up")) => self.font_tip.list = Some((i + n - 1) % n),
            (_, Some("keep")) => self.set_font(i),
            (KeyCode::Esc, _) | (_, Some("cancel")) | (_, Some("font")) => self.font_tip.list = None,
            _ => {}
        }
    }

    /// A click on a font of the open list.
    pub(super) fn click_font_list(&mut self, x: u16, y: u16) -> bool {
        let hit = self.frame.font_items().iter().position(|r| r.contains(ratatui::layout::Position::new(x, y)));
        hit.inspect(|i| self.set_font(*i)).is_some()
    }

    /// Set the font in the terminal by the recipe, without the agent (FONT-35); a failure goes to
    /// the flash line with the repair (FONT-38).
    pub fn set_font(&mut self, i: usize) {
        self.font_tip.list = None;
        let Some(font) = self.font_tip.installed.get(i).copied() else { return };
        let tip = &self.font_tip;
        let done = recipe::Recipes::load(&tip.recipes)
            .and_then(|r| r.get(&tip.terminal).ok_or_else(|| format!("no font recipe for {}", tip.terminal)).and_then(|r| r.set(font.family, &recipe::System)))
            .and_then(|escape| {
                if let Some(e) = escape {
                    write_escape(&e);
                }
                recipe::remember(&tip.recipes, &tip.terminal, font.family)
            });
        match done {
            Ok(()) => {
                self.font_tip.current = Some(font.family.to_string());
                self.flash(format!("Font: {}", font.name));
            }
            Err(e) => self.flash(format!("The font recipe failed: {e}. Run `zendo doctor --fix --agent`.")),
        }
    }
}

/// An escape sequence of a recipe goes to the terminal at once.
fn write_escape(text: &str) {
    use std::io::Write;
    let mut out = std::io::stdout();
    let _ = out.write_all(text.as_bytes()).and_then(|()| out.flush());
}
