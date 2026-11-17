//! The sound of a turn that ended (ATTN-1, ATTN-12): zendo plays it itself where it can, so that it
//! does not depend on the bell setting of the terminal; elsewhere the terminal bell rings.

use std::process::{Command, Stdio};

/// The program and the sound file of a player.
pub(super) type Player = (&'static str, &'static str);

/// The player of the sound on the system `os`: `afplay` on a Mac. None over SSH, where the sound
/// would play on the other machine, and on a system without a known player.
pub(super) fn player(os: &str, ssh: bool) -> Option<Player> {
    (os == "macos" && !ssh).then_some(("afplay", "/System/Library/Sounds/Glass.aiff"))
}

/// Start `player`, and do not wait for the end of the sound. False when there is no player, or it
/// does not start: then the terminal bell rings.
pub(super) fn play(player: Option<Player>) -> bool {
    let Some((program, file)) = player else { return false };
    let child = Command::new(program).arg(file).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
    // A thread waits for the player, so that the process does not stay as a zombie.
    child.map(|mut c| std::thread::spawn(move || c.wait())).is_ok()
}

/// The sound of a turn that ended: the player of this system, else the terminal bell.
pub(super) fn ring() {
    let ssh = std::env::var_os("SSH_CONNECTION").is_some() || std::env::var_os("SSH_TTY").is_some();
    if !play(player(std::env::consts::OS, ssh)) {
        bell();
    }
}

/// The terminal bell: the terminal decides what the user hears.
fn bell() {
    use std::io::Write;
    let mut out = std::io::stdout();
    let _ = out.write_all(b"\x07").and_then(|_| out.flush());
}

#[cfg(test)]
#[path = "sound_tests.rs"]
mod tests;
