//! Lofi music that zendo makes itself (docs/zendo/lofi.md, ADR 0006). The synth runs in the audio
//! thread; the rest of zendo sees only commands (start, stop, chime) and one state (LOFI-4).
pub mod synth;
mod output;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};

/// The audio output that the music would use, or why there is none (LOFI-3, LOFI-14).
pub fn probe() -> Result<String, String> {
    output::probe()
}

/// What the user sees of the music.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Plays,
    Stopped,
    /// The machine has no audio output (LOFI-3).
    Unavailable,
}

/// Where the samples go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Output {
    /// The audio output of this system: CoreAudio on macOS, a player on Linux (LOFI-14).
    System,
    /// An output that takes the commands and plays nothing, for the tests.
    Test,
    /// No output, as over SSH, for the tests (LOFI-3).
    Missing,
}

/// The output opens on its own thread, so the screen does not wait for it (LOFI-4).
const OPENING: u8 = 0;
/// The output plays the samples of the synth.
const OPEN: u8 = 1;
/// The output did not open, or it died, as a sound server that restarts (LOFI-3).
const FAILED: u8 = 2;

/// The commands from zendo to the audio thread, and the state of the output back: lock-free, so
/// no side waits (LOFI-4).
pub struct Control {
    playing: AtomicBool,
    chimes: AtomicU32,
    status: AtomicU8,
}

impl Default for Control {
    fn default() -> Self {
        Control { playing: AtomicBool::new(false), chimes: AtomicU32::new(0), status: AtomicU8::new(OPENING) }
    }
}

impl Control {
    /// The output plays.
    pub fn open(&self) -> bool {
        self.status.load(Ordering::Relaxed) == OPEN
    }

    /// The output died or did not open.
    pub fn failed(&self) -> bool {
        self.status.load(Ordering::Relaxed) == FAILED
    }

    /// The output thread reports that the output plays, or that it has none. Only an opening
    /// output can become open: a failure that came first stays.
    pub(crate) fn report(&self, open: bool) {
        if open {
            let _ = self.status.compare_exchange(OPENING, OPEN, Ordering::Relaxed, Ordering::Relaxed);
        } else {
            self.status.store(FAILED, Ordering::Relaxed);
        }
    }

    /// The music shall sound now.
    pub fn playing(&self) -> bool {
        self.playing.load(Ordering::Relaxed)
    }

    /// The count of chimes asked for so far; the audio thread plays one for each new count.
    pub fn chimes(&self) -> u32 {
        self.chimes.load(Ordering::Relaxed)
    }
}

/// Apply the commands of `control` to `synth` before each block of samples.
pub(crate) fn follow(control: &Control, synth: &mut synth::Synth, chimes: &mut u32) {
    synth.set_playing(control.playing());
    let now = control.chimes();
    if now != *chimes {
        *chimes = now;
        synth.chime();
    }
}

/// The music setting, the question at the first start, the focus, and the output.
pub struct Music {
    /// The user wants the music (LOFI-8, LOFI-11).
    pub on: bool,
    /// The config has the setting `music`: zendo asked once (LOFI-12).
    pub asked: bool,
    /// The question waits for the output to open: without one, zendo does not ask (LOFI-12).
    pub ask_pending: bool,
    /// The question of the first start is open (LOFI-12).
    pub asking: bool,
    /// The terminal has the focus (LOFI-15).
    pub focused: bool,
    /// The config file that the button and the question write.
    pub config: PathBuf,
    control: Arc<Control>,
}


impl Default for Music {
    fn default() -> Self {
        // No config file: `App::new` names it (`Config::user_config`).
        let config = PathBuf::new();
        let control = Arc::new(Control::default());
        // Not started: no output yet.
        control.report(false);
        Music { on: true, asked: true, ask_pending: false, asking: false, focused: true, config, control }
    }
}

impl Music {
    /// The setting of the config: on without it (LOFI-11).
    pub fn from_config(user: &crate::setup::config::UserConfig) -> Self {
        Music { on: user.music.unwrap_or(true), asked: user.music.is_some(), ..Default::default() }
    }

    /// Open `output`: the system output opens on its own thread and does not keep the screen
    /// waiting (LOFI-4); the music plays if the user wants it.
    pub fn start(&mut self, output: Output) {
        self.control = Arc::new(Control::default());
        match output {
            Output::System => output::start(self.control.clone()),
            Output::Test => self.control.report(true),
            Output::Missing => self.control.report(false),
        }
        self.sync();
    }

    pub fn state(&self) -> State {
        match (self.control.open(), self.on) {
            (false, _) => State::Unavailable,
            (true, true) => State::Plays,
            (true, false) => State::Stopped,
        }
    }

    /// The music sounds now: on, with an open output, and with the focus (LOFI-15).
    pub fn sounds(&self) -> bool {
        self.control.open() && self.control.playing()
    }

    /// Turn the music on or off (LOFI-8).
    pub fn set(&mut self, on: bool) {
        self.on = on;
        self.sync();
    }

    /// The terminal got or lost the focus (LOFI-15).
    pub fn focus(&mut self, focused: bool) {
        self.focused = focused;
        self.sync();
    }

    /// A chime in the music, if the music sounds; false otherwise, so the system sound rings
    /// (LOFI-13).
    pub fn chime(&self) -> bool {
        let sounds = self.sounds();
        if sounds {
            self.control.chimes.fetch_add(1, Ordering::Relaxed);
        }
        sounds
    }

    /// The chimes asked for so far, for the tests.
    pub fn chimes(&self) -> u32 {
        self.control.chimes()
    }

    /// The output died, as the audio thread reports it; the tests call it too (LOFI-3).
    pub fn lose_output(&self) {
        self.control.report(false);
    }

    fn sync(&self) {
        self.control.playing.store(self.on && self.focused, Ordering::Relaxed);
    }
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
