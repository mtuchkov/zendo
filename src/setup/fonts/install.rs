//! `zendo setup fonts`: download the files of each font, check them, and write them in the font
//! folder of the user (FONT-1 to FONT-4, FONT-7 to FONT-11).
use super::*;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

/// What happened to one font.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Installed,
    /// The font was already on the machine (FONT-9).
    Present,
    /// Nothing of the font was installed (FONT-3, FONT-10).
    Failed(String),
}

/// Install the fonts that are not in the folders `search` into `dir`. `report` hears each font
/// when it is done.
pub fn install<'a>(fonts: &'a [Font<'a>], net: &mut dyn Net, dir: &Path, search: &[PathBuf], report: &mut dyn FnMut(&str, &Outcome)) -> Vec<(&'a str, Outcome)> {
    let names = names_in(search);
    let mut zips = HashMap::new();
    let mut one = |font: &'a Font<'a>| {
        let outcome = match on_machine(font, &names) {
            true => Outcome::Present,
            false => fetch_all(font, net, &mut zips).and_then(|files| write_all(dir, &files)).map_or_else(Outcome::Failed, |()| Outcome::Installed),
        };
        report(font.name, &outcome);
        (font.name, outcome)
    };
    fonts.iter().map(&mut one).collect()
}

/// The archives that are open, by URL: the central directory of each comes once.
type Zips<'a> = HashMap<&'a str, Zip>;

/// All files of a font, each with its pinned checksum; the first failure stops it (FONT-3).
fn fetch_all<'a>(font: &Font<'a>, net: &mut dyn Net, zips: &mut Zips<'a>) -> Result<Vec<(&'a str, Vec<u8>)>, String> {
    font.parts.iter().map(|p| fetch(p, net, zips).and_then(|bytes| checked(p, bytes))).collect()
}

fn fetch<'a>(p: &Part<'a>, net: &mut dyn Net, zips: &mut Zips<'a>) -> Result<Vec<u8>, String> {
    match p.origin {
        Origin::Raw(url) => net.open(url).and_then(|mut s| s.size().and_then(|n| s.range(0, n))).map_err(|e| format!("{url}: {e}")),
        Origin::Zip { url, member } => {
            let zip = match zips.entry(url) {
                Entry::Occupied(e) => e.into_mut(),
                Entry::Vacant(e) => e.insert(net.open(url).and_then(Zip::open).map_err(|e| format!("{url}: {e}"))?),
            };
            zip.read(member).map_err(|e| format!("{url}: {e}"))
        }
    }
}

fn checked<'a>(p: &Part<'a>, bytes: Vec<u8>) -> Result<(&'a str, Vec<u8>), String> {
    let sum = sha256(&bytes);
    if sum != p.sha256 {
        return Err(format!("{}: the checksum is {sum}, not the pinned {}", p.file, p.sha256));
    }
    Ok((p.file, bytes))
}

/// Write the files: each first under a temporary name, so that no half file stays.
fn write_all(dir: &Path, files: &[(&str, Vec<u8>)]) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    files.iter().try_for_each(|(name, bytes)| {
        let (part, file) = (dir.join(format!(".{name}.part")), dir.join(name));
        std::fs::write(&part, bytes).and_then(|()| std::fs::rename(&part, &file)).map_err(|e| format!("{}: {e}", file.display()))
    })
}

/// The SHA-256 of the bytes, in lower-case hex.
pub(super) fn sha256(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
#[path = "install_tests.rs"]
mod tests;
