//! The lofi music in the app (docs/zendo/lofi.md): the start, the music button and key, the
//! question at the first start, and the focus.
use super::*;
use crate::music::{Output, State};
use crate::setup::config::UserConfig;

impl App {
    /// Open the audio output; the music plays where the user wants it (LOFI-11). Without the
    /// setting in the config, the float asks once, when the output plays (LOFI-12); a config that
    /// does not parse is no first start.
    pub fn start_music(&mut self, output: Output) {
        self.music.start(output);
        let broken = UserConfig::read_from(&self.music.config).is_err();
        self.music.ask_pending = !self.music.asked && !broken;
        self.music_tick();
    }

    /// Each tick: the question of the first start opens once the output plays (LOFI-12).
    pub(crate) fn music_tick(&mut self) {
        if self.music.ask_pending && self.music.state() != State::Unavailable {
            self.music.ask_pending = false;
            self.music.asking = true;
        }
    }

    /// The music button and key: stop or start the music, and save it in the config (LOFI-8).
    /// Without an audio output, say so (LOFI-3).
    pub fn toggle_music(&mut self) {
        if self.music.state() == State::Unavailable {
            return self.flash("music: no audio output");
        }
        let on = !self.music.on;
        self.music.set(on);
        self.save_music(if on { "music on" } else { "music off" });
    }

    /// The answer to the question of the first start: keep the music on, or turn it off (LOFI-12).
    pub fn answer_music(&mut self, keep: bool) {
        self.music.asking = false;
        self.music.asked = true;
        self.music.set(keep);
        self.save_music(if keep { "music on at each start" } else { "music off; alt-m or [music] plays it" });
    }

    /// A key while the question is open: `y` or enter keeps the music, `n` or esc turns it off.
    pub(super) fn key_music_question(&mut self, k: KeyEvent) {
        match k.code {
            KeyCode::Char('y') | KeyCode::Enter => self.answer_music(true),
            KeyCode::Char('n') | KeyCode::Esc => self.answer_music(false),
            _ => {}
        }
    }

    /// A click while the question is open: on one of its answers.
    pub(super) fn click_music_question(&mut self, x: u16, y: u16) {
        let Some((keep, off)) = self.music.answers else { return };
        let hit = |r: Rect| r.contains(ratatui::layout::Position::new(x, y));
        if hit(keep) || hit(off) {
            self.answer_music(hit(keep));
        }
    }

    /// Save the setting in the config; a config that does not parse stays as the user wrote it
    /// (CMD-17).
    fn save_music(&mut self, word: &str) {
        let on = self.music.on;
        match UserConfig::update(&self.music.config, |user| user.music = Some(on)) {
            Ok(()) => self.flash(word.to_string()),
            Err(e) => self.flash(format!("{word}; not saved: {e}")),
        }
    }
}
