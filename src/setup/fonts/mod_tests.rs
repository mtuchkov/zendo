use super::*;

fn folder(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-fonts-mod-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

// FONT-1, FONT-7, FONT-8, FONT-11: the eight fonts, the four styles of each, Regular and Bold of
// Fira Code, a license for each, all pinned.
#[test]
fn the_catalog_has_the_eight_fonts_and_their_styles() {
    let names: Vec<&str> = FONTS.iter().map(|f| f.name).collect();
    assert_eq!(names, ["JetBrains Mono", "Atkinson Hyperlegible Mono", "IBM Plex Mono", "Iosevka", "Iosevka Term", "Fira Code", "Cascadia Code", "iA Writer Mono"]);
    for f in FONTS {
        let files: Vec<&str> = f.parts.iter().map(|p| p.file).collect();
        let styles: &[&str] = if f.prefix == "FiraCode" { &["Regular", "Bold"] } else { &["Regular", "Bold", "Italic", "BoldItalic"] };
        let want: Vec<String> = styles.iter().map(|s| format!("{}-{s}.ttf", f.prefix)).collect();
        assert_eq!(files[..files.len() - 1], want, "{}", f.name);
        assert!(files.last().unwrap().starts_with(&format!("{}-LICENSE.", f.prefix)), "{}", f.name);
        for p in f.parts {
            assert!(p.sha256.len() == 64 && p.sha256.bytes().all(|b| b.is_ascii_hexdigit()), "{}", p.file);
            let url = match p.origin {
                Origin::Zip { url, .. } | Origin::Raw(url) => url,
            };
            let pinned = url.starts_with("https://github.com/") && url.contains("/releases/download/") || url.starts_with("https://raw.githubusercontent.com/");
            assert!(pinned && !url.contains("/latest/") && !url.contains("/main/") && !url.contains("/master/"), "{url}");
        }
    }
}

// FONT-13: a font without its Regular style in the folders is missing; a font with it is not.
#[test]
fn a_font_is_missing_without_its_regular_style() {
    let d = folder("missing");
    std::fs::create_dir_all(d.join("a/b")).unwrap();
    std::fs::write(d.join("a/b/JetBrainsMono-Regular.ttf"), b"x").unwrap();
    std::fs::write(d.join("FiraCode-Bold.ttf"), b"x").unwrap();
    std::fs::write(d.join("cascadiacode-regular.otf"), b"x").unwrap();
    let missing: Vec<&str> = super::missing(FONTS, &[d.clone(), d.join("no such folder")]).iter().map(|f| f.name).collect();
    assert_eq!(missing, ["Atkinson Hyperlegible Mono", "IBM Plex Mono", "Iosevka", "Iosevka Term", "Fira Code", "iA Writer Mono"]);
    let _ = std::fs::remove_dir_all(&d);
}

// FONT-20: the tip names the key of the theme picker from the keymap, and the font button.
#[test]
fn the_tip_names_the_key_of_the_theme_picker() {
    let t = tip(Some("T"));
    assert!(t.contains("`T` in read mode") && t.contains("theme picker") && t.contains("font button"), "{t}");
    assert!(tip(None).contains("action `theme`"), "an action without a key: {}", tip(None));
}

// FONT-18, FONT-19: setup names the fonts at the first run and when the list changes, and says
// nothing when the list is the same as the last time.
#[test]
fn setup_names_the_fonts_when_the_list_changed() {
    let d = folder("named");
    let named = d.join("data/fonts-named.txt");
    let first = note(&FONTS[..6], &named, Some("t")).expect("the first run names the fonts");
    assert!(first.contains("zendo setup fonts") && first.contains("`t` in read mode"), "{first}");
    assert_eq!(note(&FONTS[..6], &named, Some("t")), None, "the same list");
    assert!(note(FONTS, &named, Some("t")).is_some(), "one more font");
    assert_eq!(note(FONTS, &named, Some("t")), None);
    let _ = std::fs::remove_dir_all(&d);
}
