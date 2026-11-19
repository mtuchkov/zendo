use super::*;
use crate::setup::fonts::test_zip::{sha, zip};
use std::collections::HashMap;
use std::io;

/// The local copies of the release archives, by URL.
#[derive(Default)]
struct Local(HashMap<String, Vec<u8>>);

impl Net for Local {
    fn open(&mut self, url: &str) -> io::Result<Box<dyn Source>> {
        let bytes = self.0.get(url).cloned().ok_or_else(|| io::Error::other(format!("404 for {url}")))?;
        Ok(Box::new(Bytes(bytes)))
    }
}

fn s(x: String) -> &'static str {
    Box::leak(x.into_boxed_str())
}

/// A test folder of its own, empty.
fn folder(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-fonts-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

const STYLES: [&str; 4] = ["Regular", "Bold", "Italic", "BoldItalic"];

/// The bytes of a style of a font.
fn body(prefix: &str, style: &str) -> Vec<u8> {
    format!("{prefix} {style} glyphs ").repeat(40).into_bytes()
}

/// A font with the styles in a local archive; its license in the archive, or beside it as a raw
/// file. The style `bad` has a wrong pin.
fn font(net: &mut Local, prefix: &str, styles: &[&str], raw_license: bool, bad: Option<&str>) -> Font<'static> {
    let url = s(format!("https://example.org/{prefix}.zip"));
    let (mut members, mut parts) = (Vec::new(), Vec::new());
    for st in styles {
        let (member, bytes) = (s(format!("ttf/{prefix}-{st}.ttf")), body(prefix, st));
        let sum = if bad == Some(*st) { sha(b"other bytes") } else { sha(&bytes) };
        parts.push(Part { origin: Origin::Zip { url, member }, file: s(format!("{prefix}-{st}.ttf")), sha256: s(sum) });
        members.push((member, bytes));
    }
    let license = format!("SIL Open Font License of {prefix}").into_bytes();
    let file = s(format!("{prefix}-LICENSE.txt"));
    let origin = if raw_license {
        let raw = s(format!("https://example.org/{prefix}/OFL.txt"));
        net.0.insert(raw.into(), license.clone());
        Origin::Raw(raw)
    } else {
        members.push(("OFL.txt", license.clone()));
        Origin::Zip { url, member: "OFL.txt" }
    };
    parts.push(Part { origin, file, sha256: s(sha(&license)) });
    let refs: Vec<(&str, &[u8], bool)> = members.iter().map(|(m, b)| (*m, &b[..], true)).collect();
    net.0.insert(url.into(), zip(&refs));
    Font { name: s(format!("{prefix} Mono")), family: s(format!("{prefix} Mono")), prefix: s(prefix.into()), parts: Box::leak(parts.into_boxed_slice()) }
}

fn names(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    v.sort();
    v
}

fn run<'a>(fonts: &'a [Font<'a>], net: &mut Local, dir: &Path, search: &[PathBuf]) -> Vec<(&'a str, Outcome)> {
    install(fonts, net, dir, search, &mut |_, _| {})
}

// FONT-1, FONT-4, FONT-7, FONT-8, FONT-9, FONT-11: the four styles and the license of each font,
// only two styles for a font without italic, only in the font folder; a second run changes nothing.
#[test]
fn setup_fonts_installs_the_pinned_styles() {
    let root = folder("styles");
    let dir = root.join("fonts");
    let mut net = Local::default();
    let fonts = [font(&mut net, "Aaa", &STYLES, false, None), font(&mut net, "Bbb", &["Regular", "Bold"], true, None)];
    let first = run(&fonts, &mut net, &dir, std::slice::from_ref(&dir));
    assert_eq!(first, vec![("Aaa Mono", Outcome::Installed), ("Bbb Mono", Outcome::Installed)]);
    let want = ["Aaa-Bold.ttf", "Aaa-BoldItalic.ttf", "Aaa-Italic.ttf", "Aaa-LICENSE.txt", "Aaa-Regular.ttf", "Bbb-Bold.ttf", "Bbb-LICENSE.txt", "Bbb-Regular.ttf"];
    assert_eq!(names(&dir), want);
    assert_eq!(std::fs::read(dir.join("Aaa-Italic.ttf")).unwrap(), body("Aaa", "Italic"));
    assert_eq!(names(&root), ["fonts"], "nothing outside the font folder");
    let when = std::fs::metadata(dir.join("Aaa-Regular.ttf")).unwrap().modified().unwrap();
    // The second run has no network: it needs none.
    let again = run(&fonts, &mut Local::default(), &dir, std::slice::from_ref(&dir));
    assert_eq!(again, vec![("Aaa Mono", Outcome::Present), ("Bbb Mono", Outcome::Present)]);
    assert_eq!(names(&dir), want);
    assert_eq!(std::fs::metadata(dir.join("Aaa-Regular.ttf")).unwrap().modified().unwrap(), when);
    let _ = std::fs::remove_dir_all(&root);
}

// FONT-3, FONT-10: a file without its pinned checksum installs nothing of its font; the other
// fonts are installed.
#[test]
fn a_bad_checksum_installs_nothing_of_the_font() {
    let root = folder("checksum");
    let mut net = Local::default();
    let fonts = [font(&mut net, "Aaa", &STYLES, false, Some("Italic")), font(&mut net, "Bbb", &STYLES, false, None)];
    let out = run(&fonts, &mut net, &root, std::slice::from_ref(&root));
    let Outcome::Failed(why) = &out[0].1 else { panic!("{out:?}") };
    assert!(why.contains("Aaa-Italic.ttf") && why.contains("checksum"), "{why}");
    assert_eq!(out[1].1, Outcome::Installed);
    assert!(names(&root).iter().all(|n| n.starts_with("Bbb-")), "{:?}", names(&root));
    let _ = std::fs::remove_dir_all(&root);
}

// FONT-10: a failed download names the font and the failure; the other fonts are installed.
#[test]
fn a_failed_download_names_the_failure_and_installs_the_others() {
    let root = folder("download");
    let mut net = Local::default();
    let fonts = [font(&mut net, "Aaa", &STYLES, true, None), font(&mut net, "Bbb", &STYLES, false, None)];
    net.0.remove("https://example.org/Aaa/OFL.txt");
    let mut heard = Vec::new();
    let out = install(&fonts, &mut net, &root, std::slice::from_ref(&root), &mut |name, o| heard.push(format!("{name}: {o:?}")));
    assert!(matches!(&out[0].1, Outcome::Failed(why) if why.contains("404")), "{out:?}");
    assert_eq!(out[1].1, Outcome::Installed);
    assert_eq!(heard.len(), 2, "each font is reported when it is done: {heard:?}");
    assert!(names(&root).iter().all(|n| n.starts_with("Bbb-")));
    let _ = std::fs::remove_dir_all(&root);
}

// FONT-9: a font that is already on the machine, in another folder and in another case, is not
// installed again.
#[test]
fn a_font_on_the_machine_is_not_installed_again() {
    let root = folder("present");
    let (dir, system) = (root.join("user"), root.join("system/truetype/aaa"));
    std::fs::create_dir_all(&system).unwrap();
    std::fs::write(system.join("aaa-regular.TTF"), b"x").unwrap();
    let mut net = Local::default();
    let fonts = [font(&mut net, "Aaa", &STYLES, false, None)];
    let out = run(&fonts, &mut net, &dir, &[dir.clone(), root.join("system")]);
    assert_eq!(out, vec![("Aaa Mono", Outcome::Present)]);
    assert!(!dir.exists(), "nothing written");
    let _ = std::fs::remove_dir_all(&root);
}
