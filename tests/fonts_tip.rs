//! The font list of the theme picker (docs/zendo/fonts-tip.md): the list, the font that it sets by
//! the recipe of the terminal, and the tip without a recipe.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::path::{Path, PathBuf};
use zendo::app::{App, CHAT_KEY, Item, Mode};

/// A font folder with the Regular style of the fonts `with`.
fn fonts(root: &Path, with: &[&str]) -> Vec<PathBuf> {
    let d = root.join("fonts");
    std::fs::create_dir_all(&d).unwrap();
    for f in with {
        std::fs::write(d.join(format!("{f}-Regular.ttf")), b"x").unwrap();
    }
    vec![d]
}

const ALL: [&str; 8] = ["JetBrainsMono", "AtkinsonHyperlegibleMono", "IBMPlexMono", "Iosevka", "IosevkaTerm", "FiraCode", "CascadiaCode", "iAWriterMonoS"];

/// The app on a repository with a note, the font folder of the test, and the recipe file of the
/// test for the terminal `test-term`.
fn app_with(name: &str, with: &[&str], recipe: Option<&str>) -> (PathBuf, App) {
    let root = repo(name, &[("docs/notes/2026-11-18-a.md", "# A\n\n- one\n")]);
    let mut a = app(&root);
    a.font_tip.dirs = fonts(&root, with);
    a.font_tip.recipes = root.join("terminal-fonts.toml");
    a.font_tip.terminal = "test-term".into();
    if let Some(r) = recipe {
        std::fs::write(&a.font_tip.recipes, format!("[test-term]\n{}\n", r.replace("$ROOT", &root.display().to_string()))).unwrap();
    }
    (root, a)
}

/// A recipe that sets the line `font = …` of the file `term.conf`.
const LINE: &str = "file = \"$ROOT/term.conf\"\nprefix = \"font\"\nline = \"font = {font}\"";

fn picker(a: &mut App) -> String {
    key(a, KeyCode::Char('t'));
    assert_eq!(a.mode, Mode::Theme);
    draw(a, 160, 40).join("\n")
}

fn users(a: &App) -> usize {
    a.threads.get(CHAT_KEY).map_or(0, |t| t.items.iter().filter(|i| matches!(i, Item::User(_))).count())
}

/// The cell of the text on the screen.
fn at(a: &mut App, text: &str) -> (u16, u16) {
    let rows = draw(a, 160, 40);
    rows.iter().enumerate().find_map(|(y, r)| r.find(text).map(|i| (r[..i].chars().count() as u16 + 1, y as u16))).unwrap_or_else(|| panic!("{text}: {rows:#?}"))
}

fn click(a: &mut App, (column, row): (u16, u16)) {
    a.on_mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column, row, modifiers: KeyModifiers::NONE });
    a.on_mouse(MouseEvent { kind: MouseEventKind::Up(MouseButton::Left), column, row, modifiers: KeyModifiers::NONE });
    a.settle_saves();
}

// FONT-34, FONT-35, FONT-36: the font list with the fonts on the machine; `f`, `j` and enter set
// the second font by the recipe, without the agent.
#[test]
fn the_font_list_sets_the_font_by_the_recipe() {
    let (root, mut a) = app_with("fonts-list", &["JetBrainsMono", "IosevkaTerm"], Some(LINE));
    std::fs::write(root.join("term.conf"), "theme = dark\nfont = Menlo\n").unwrap();
    let s = picker(&mut a);
    assert!(s.contains("Font  [ choose ▾ ]"), "{s}");
    key(&mut a, KeyCode::Char('f'));
    let s = draw(&mut a, 160, 40).join("\n");
    assert!(s.contains("JetBrains Mono") && s.contains("Iosevka Term") && !s.contains("Fira Code"), "only the fonts on the machine: {s}");
    key(&mut a, KeyCode::Char('j'));
    key(&mut a, KeyCode::Enter);
    assert_eq!(std::fs::read_to_string(root.join("term.conf")).unwrap(), "theme = dark\nfont = Iosevka Term\n");
    assert_eq!(a.mode, Mode::Theme, "the picker stays open");
    assert!(draw(&mut a, 160, 40).join("\n").contains("Font  [ Iosevka Term ▾ ]"));
    assert!(std::fs::read_to_string(&a.font_tip.recipes).unwrap().contains("font = \"Iosevka Term\""), "the recipe keeps the font");
    assert_eq!(users(&a), 0, "no request to the agent");
    key(&mut a, KeyCode::Esc);
    assert!(picker(&mut a).contains("Font  [ Iosevka Term ▾ ]"), "the font that zendo set last");
    let _ = std::fs::remove_dir_all(&root);
}

// FONT-36: a click opens the font list, and a click on a font sets it.
#[test]
fn a_click_opens_the_font_list_and_sets_a_font() {
    let (root, mut a) = app_with("fonts-click", &ALL, Some(LINE));
    std::fs::write(root.join("term.conf"), "").unwrap();
    picker(&mut a);
    let list = at(&mut a, "Font  [ choose");
    click(&mut a, list);
    let font = at(&mut a, "Cascadia Code");
    click(&mut a, font);
    assert_eq!(std::fs::read_to_string(root.join("term.conf")).unwrap(), "font = Cascadia Code\n");
    let _ = std::fs::remove_dir_all(&root);
}

// FONT-2, FONT-37: without a recipe, the picker shows the tip in place of the font list, offline.
#[test]
fn the_picker_shows_the_tip_without_a_recipe() {
    let (root, mut a) = app_with("fonts-tip", &ALL, None);
    a.cfg.offline = true;
    let s = picker(&mut a);
    assert!(s.contains("zendo setup fonts") && !s.contains("Font  ["), "{s}");
    key(&mut a, KeyCode::Char('f'));
    assert!(!draw(&mut a, 160, 40).join("\n").contains("JetBrains Mono"), "no list to open");
    let _ = std::fs::remove_dir_all(&root);
}

// FONT-38: a recipe that fails names the failure and `zendo doctor --fix --agent`.
#[test]
fn a_failed_recipe_names_doctor_in_the_flash_line() {
    // The file of the recipe is a folder: the check passes, the write fails.
    let (root, mut a) = app_with("fonts-fail", &ALL, Some(LINE));
    std::fs::create_dir_all(root.join("term.conf")).unwrap();
    picker(&mut a);
    key(&mut a, KeyCode::Char('f'));
    key(&mut a, KeyCode::Enter);
    let flash = a.flash.as_ref().map(|f| f.0.clone()).unwrap_or_default();
    assert!(flash.contains("term.conf") && flash.contains("zendo doctor --fix --agent"), "{flash}");
    let _ = std::fs::remove_dir_all(&root);
}

// FONT-2: without the recommended fonts and without the network, zendo shows the note and the
// themes as before.
#[test]
fn zendo_works_without_the_fonts() {
    let (root, mut a) = app_with("fonts-none", &[], None);
    a.cfg.offline = true;
    let note = draw(&mut a, 160, 40).join("\n");
    assert!(note.contains("one"), "{note}");
    let s = picker(&mut a);
    assert!(["Ink", "Daylight", "Moss", "Mono"].iter().all(|t| s.contains(t)) && s.contains("zendo setup fonts"), "{s}");
    key(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Read);
    let _ = std::fs::remove_dir_all(&root);
}
