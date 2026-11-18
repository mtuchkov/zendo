//! The recommended fonts (docs/zendo/fonts.md, docs/zendo/fonts-tip.md): the catalog, where the
//! fonts are on the machine, the install of `zendo setup fonts`, and the tip.
mod catalog;
mod install;
mod remote;

pub use catalog::FONTS;
pub use install::{Outcome, install};
pub use remote::{Bytes, Curl, Net, Source, Zip};

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// A recommended font: the files that `zendo setup fonts` installs for it.
pub struct Font<'a> {
    pub name: &'a str,
    /// The start of the file names: `JetBrainsMono` for `JetBrainsMono-Regular.ttf`.
    pub prefix: &'a str,
    pub parts: &'a [Part<'a>],
}

/// One file of a font: where it comes from, its name in the font folder, and its pinned checksum.
pub struct Part<'a> {
    pub origin: Origin<'a>,
    pub file: &'a str,
    pub sha256: &'a str,
}

/// Where a file comes from: a member of a release archive, or a file at a pinned commit or tag.
pub enum Origin<'a> {
    Zip { url: &'a str, member: &'a str },
    Raw(&'a str),
}

/// How deep the search goes below a font folder.
const DEPTH: usize = 5;

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
}

/// The font folder of the user: `~/Library/Fonts` on macOS, `~/.local/share/fonts` elsewhere.
pub fn user_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        return home().join("Library/Fonts");
    }
    // An empty XDG_DATA_HOME counts as not set: it would name a folder in the current one.
    std::env::var_os("XDG_DATA_HOME").filter(|d| !d.is_empty()).map(PathBuf::from).unwrap_or_else(|| home().join(".local/share")).join("fonts")
}

/// The folders where a font can be on this machine: the folder of the user first.
pub fn search_dirs() -> Vec<PathBuf> {
    let system: &[&str] = if cfg!(target_os = "macos") { &["/Library/Fonts", "/System/Library/Fonts"] } else { &["/usr/share/fonts", "/usr/local/share/fonts"] };
    let mut dirs = vec![user_dir(), home().join(".fonts")];
    dirs.extend(system.iter().map(PathBuf::from));
    dirs
}

/// The file names in the folders and below them, in lower case.
pub fn names_in(dirs: &[PathBuf]) -> HashSet<String> {
    let mut names = HashSet::new();
    let mut todo: Vec<(PathBuf, usize)> = dirs.iter().map(|d| (d.clone(), 0)).collect();
    while let Some((dir, depth)) = todo.pop() {
        for e in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = e.path();
            match path.is_dir() {
                true if depth < DEPTH => todo.push((path, depth + 1)),
                true => {}
                false => drop(names.insert(e.file_name().to_string_lossy().to_lowercase())),
            }
        }
    }
    names
}

/// The font is on the machine: a folder has its Regular style, from zendo or from elsewhere.
pub fn on_machine(font: &Font, names: &HashSet<String>) -> bool {
    let regular = format!("{}-regular", font.prefix.to_lowercase());
    ["ttf", "otf"].iter().any(|ext| names.contains(&format!("{regular}.{ext}")))
}

/// The recommended fonts that are not on the machine.
pub fn missing<'a>(fonts: &'a [Font<'a>], dirs: &[PathBuf]) -> Vec<&'a Font<'a>> {
    let names = names_in(dirs);
    fonts.iter().filter(|f| !on_machine(f, &names)).collect()
}

/// The list of the recommended fonts, one name on each line.
pub fn list(fonts: &[Font]) -> String {
    fonts.iter().map(|f| format!("{}\n", f.name)).collect()
}

/// The tip: how to open the theme picker, with the key of the keymap of the user, and what the
/// font button there does (FONT-20).
pub fn tip(theme_key: Option<&str>) -> String {
    let open = match theme_key {
        Some(k) => format!("press `{k}` in read mode"),
        None => "give the action `theme` of read mode a key".to_string(),
    };
    format!("To change the font, open the theme picker in zendo: {open}. The font button there asks the agent to set the font of your terminal.")
}

/// The file where `zendo setup` keeps the list of fonts that it named the last time (FONT-18).
pub fn named_path() -> PathBuf {
    crate::setup::config::data_dir().join("fonts-named.txt")
}

/// What `zendo setup` says about fonts at its end: the command and the tip when the list changed
/// since the last time that setup named it, else nothing (FONT-18, FONT-19). Keeps the list.
pub fn note(fonts: &[Font], named: &Path, theme_key: Option<&str>) -> Option<String> {
    let now = list(fonts);
    if std::fs::read_to_string(named).is_ok_and(|was| was == now) {
        return None;
    }
    if let Some(dir) = named.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(named, &now);
    Some(format!("Recommended fonts: run `zendo setup fonts` to get them.\n{}", tip(theme_key)))
}

#[cfg(test)]
mod test_zip;
#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
