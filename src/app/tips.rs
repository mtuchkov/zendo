//! The tips (docs/zendo/tips.md): the tip menu at start, after the music question at the first
//! start, and a tip in the status bar.
use super::*;
use crate::setup::config::UserConfig;
use std::path::PathBuf;

/// The tips of zendo (TIPS-2).
pub const TIPS: &[&str] = &["If a key does not work for you, double-click it to change it."];

/// The tip menu and the tip of the status bar.
#[derive(smart_default::SmartDefault)]
pub struct Tips {
    /// The tip menu waits for the music question (TIPS-1).
    pending: bool,
    /// The tip menu is open.
    pub open: bool,
    /// `Do not show tips at start` is on (TIPS-4).
    pub off: bool,
    /// The tip of the status bar, chosen at random at the start (TIPS-8).
    #[default(pick(TIPS.len()))]
    pub tip: usize,
    /// The config file that the option writes (TIPS-7).
    #[default(crate::setup::config::config_path())]
    pub config: PathBuf,
}

/// A number below `n`, from the clock.
fn pick(n: usize) -> usize {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.subsec_nanos() as usize);
    nanos % n.max(1)
}

impl Tips {
    /// The tip of the status bar.
    pub fn status(&self) -> &'static str {
        TIPS[self.tip % TIPS.len()]
    }
}

impl App {
    /// The start: the tip menu comes unless the config has `tips = false` (TIPS-1, TIPS-7).
    pub fn start_tips(&mut self) {
        self.tips.pending = UserConfig::load_from(&self.tips.config).tips != Some(false);
        self.tips_tick();
    }

    /// Each tick: the tip menu opens once the music question has its answer, or has none to wait
    /// for (TIPS-1).
    pub(crate) fn tips_tick(&mut self) {
        let waits = self.music.asking || (self.music.ask_pending && self.music.state() != crate::music::State::Unavailable);
        if self.tips.pending && !waits {
            self.tips.pending = false;
            self.tips.open = true;
        }
    }

    /// A key while the tip menu is open: `enter` closes it, `space` turns the option on or off,
    /// other keys do nothing (TIPS-3, TIPS-5).
    pub(super) fn key_tips(&mut self, k: KeyEvent) {
        match k.code {
            KeyCode::Enter => self.close_tips(),
            KeyCode::Char(' ') => self.tips.off = !self.tips.off,
            _ => {}
        }
    }

    /// A click while the tip menu is open: on the option or on `[OK]`; other clicks do nothing.
    pub(super) fn click_tips(&mut self, x: u16, y: u16) {
        let Some((option, ok)) = self.frame.tip_buttons() else { return };
        let hit = |r: Rect| r.contains(ratatui::layout::Position::new(x, y));
        if hit(option) {
            self.tips.off = !self.tips.off;
        } else if hit(ok) {
            self.close_tips();
        }
    }

    /// `[OK]`: the menu closes; with the option on, the config gets `tips = false` (TIPS-4).
    fn close_tips(&mut self) {
        self.tips.open = false;
        if !self.tips.off {
            return;
        }
        if let Err(e) = UserConfig::update(&self.tips.config, |u| u.tips = Some(false)) {
            self.flash(format!("tips: not saved: {e}"));
        }
    }
}
