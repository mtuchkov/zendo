//! `zendo setup fonts` (docs/zendo/fonts.md), and what `zendo setup` says about the fonts at its
//! end (docs/zendo/fonts-tip.md).
use super::*;
use crate::setup::fonts::{self, Outcome};

/// Download and install the recommended fonts in the font folder of the user; asks nothing
/// (FONT-7, FONT-17). False when a font failed (FONT-10).
pub fn setup_fonts() -> bool {
    let mut o = Out::new();
    o.head("zendo setup fonts");
    let dir = fonts::user_dir();
    println!("  {:<13}{}", "", o.paint("2", &format!("downloading into {}…", dir.display())));
    let mut installed = 0;
    let mut report = |name: &str, r: &Outcome| match r {
        Outcome::Installed => {
            installed += 1;
            o.ok("font", format!("{name}: installed"));
        }
        Outcome::Present => o.ok("font", format!("{name}: already on this machine")),
        Outcome::Failed(why) => o.fail("font", format!("{name}: {why}")),
    };
    fonts::install(fonts::FONTS, &mut fonts::Curl, &dir, &fonts::search_dirs(), &mut report);
    if installed > 0 && cfg!(target_os = "linux") {
        refresh_cache(&dir, &o);
    }
    println!();
    println!("  The terminal draws the font: set one of these fonts in your terminal.");
    println!("  {}", fonts::tip(theme_key().as_deref()));
    !o.failed
}

/// The font cache of the user knows the new files (FONT-12).
fn refresh_cache(dir: &Path, o: &Out) {
    let done = std::process::Command::new("fc-cache").arg("-f").arg(dir).stdin(std::process::Stdio::null()).output();
    match done {
        Ok(out) if out.status.success() => o.ok("font cache", "refreshed"),
        Ok(out) => o.warn("font cache", String::from_utf8_lossy(&out.stderr).trim()),
        Err(_) => o.warn("font cache", "fc-cache is not installed: the terminal may need a restart to see the fonts"),
    }
}

/// The key that opens the theme picker, from the keymap of the user (FONT-20).
fn theme_key() -> Option<String> {
    UserConfig::load().keymap().0.key(crate::keys::Pane::Read, "theme")
}

/// The end of `zendo setup`: the fonts, when their list changed since the last time (FONT-18,
/// FONT-19).
pub(super) fn fonts_note(o: &Out) {
    if let Some(text) = fonts::note(fonts::FONTS, &fonts::named_path(), theme_key().as_deref()) {
        let mut lines = text.lines();
        lines.next().into_iter().for_each(|l| o.hint(l));
        lines.for_each(|l| o.more(l));
    }
}
