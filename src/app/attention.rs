//! The attention of the user (docs/zendo/attention.md): when the turn of an agent session or the
//! command of a terminal ends and the user does not see it, the terminal bell rings and its button
//! blinks until the user sees it. The sound button turns the bell off and on.
use super::*;
use crate::session::TERMINALS;
use crate::setup::config::UserConfig;

/// The bell, the blinks, the sound setting, and the focus of the terminal.
pub struct Attention {
    /// The bell rings (ATTN-1); the config sets it, the sound button changes it (ATTN-8, ATTN-9).
    pub sound: bool,
    /// The config file that the sound button writes.
    pub config: PathBuf,
    /// The sound button in the last frame, for the mouse.
    pub button: Option<Rect>,
    /// The terminal has the focus; without focus events, always (ATTN-11).
    pub focused: bool,
    /// The bells to ring after the next frame.
    pub bells: usize,
    /// For each session slot: since when its button blinks.
    blink: Vec<Option<Instant>>,
}

impl Default for Attention {
    fn default() -> Self {
        let config = crate::setup::config::config_path();
        Attention { sound: true, config, button: None, focused: true, bells: 0, blink: vec![None; SESSIONS + TERMINALS] }
    }
}

impl Attention {
    pub fn from_config(user: &UserConfig) -> Self {
        Attention { sound: user.sound.unwrap_or(true), ..Default::default() }
    }

    /// Start the blink of slot `n`; a blink that runs keeps its start.
    pub fn start(&mut self, n: usize) {
        if let Some(b) = self.blink.get_mut(n) {
            b.get_or_insert_with(Instant::now);
        }
    }

    /// Let slot `n` blink since `t`.
    pub fn blink_since(&mut self, n: usize, t: Instant) {
        if let Some(b) = self.blink.get_mut(n) {
            *b = Some(t);
        }
    }

    pub fn stop(&mut self, n: usize) {
        if let Some(b) = self.blink.get_mut(n) {
            *b = None;
        }
    }

    pub fn since(&self, n: usize) -> Option<Instant> {
        self.blink.get(n).copied().flatten()
    }

    pub fn blinking(&self) -> bool {
        self.blink.iter().any(Option::is_some)
    }

    /// The slots that blink.
    pub fn blinks(&self) -> impl Iterator<Item = usize> + '_ {
        self.blink.iter().enumerate().filter_map(|(n, b)| b.map(|_| n))
    }
}

impl App {
    /// The user sees session slot `n`: it is on the screen, or the chat of its agent session is,
    /// and the terminal has the focus (ATTN-11).
    pub fn sees(&self, n: usize) -> bool {
        let chat = self.panel && matches!(self.mode, Mode::Read | Mode::Write | Mode::Help);
        let shown = match self.cli.view {
            Some(v) => v == n,
            None => chat && self.cli.pane.map_or(n == self.state.session, |p| p == n),
        };
        self.attention.focused && shown
    }

    /// The pulse of the button of slot `n`, 0 to 1, while it blinks (ATTN-3).
    pub fn blink(&self, n: usize) -> Option<f32> {
        self.attention.since(n).map(|t| crate::ui::theme::pulse(t.elapsed()))
    }

    /// The turn or the command of slot `n` ended: the bell and the blink, if the user does not see
    /// it (ATTN-1, ATTN-2).
    pub(crate) fn turn_ended(&mut self, n: usize) {
        if self.sees(n) {
            return;
        }
        if self.attention.sound {
            self.attention.bells += 1;
        }
        self.attention.start(n);
    }

    /// The end of an answer in the chat of an agent session. An answer that the user stopped, or
    /// one with a question in the queue after it, does not count (ATTN-5).
    pub(crate) fn chat_turn_ended(&mut self, key: &str, stop: &str, queued: bool) {
        let n = (0..SESSIONS).find(|n| session_key(*n) == key);
        if let Some(n) = n.filter(|_| stop != "cancelled" && !queued) {
            self.turn_ended(n);
        }
    }

    /// Each tick: the turns that ended, and the end of the blink of each session that the user
    /// sees (ATTN-4).
    pub(crate) fn attention_tick(&mut self) {
        for n in self.cli.finished() {
            self.turn_ended(n);
        }
        let seen: Vec<usize> = self.attention.blinks().filter(|n| self.sees(*n)).collect();
        seen.into_iter().for_each(|n| self.attention.stop(n));
    }

    /// The terminal got or lost the focus (ATTN-11).
    pub fn on_focus(&mut self, focused: bool) {
        self.attention.focused = focused;
    }

    /// The sound button: turn the bell off or on, and save it in the config (ATTN-8).
    pub fn toggle_sound(&mut self) {
        self.attention.sound = !self.attention.sound;
        let path = self.attention.config.clone();
        let mut user = UserConfig::load_from(&path);
        user.sound = Some(self.attention.sound);
        let word = if self.attention.sound { "on" } else { "off" };
        match user.save_to(&path) {
            Ok(()) => self.flash(format!("sound {word}")),
            Err(e) => self.flash(format!("sound {word}; could not save it in {}: {e}", path.display())),
        }
    }
}

#[cfg(test)]
#[path = "attention_tests.rs"]
mod tests;
