//! The font tip of the theme picker (docs/zendo/fonts-tip.md): the tip, the font button, the
//! question, and the request to the agent.

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

/// The app on a repository with a note, and the font folder of the test.
fn app_with(name: &str, with: &[&str]) -> (PathBuf, App) {
    let root = repo(name, &[("docs/notes/2026-11-18-a.md", "# A\n\n- one\n")]);
    let mut a = app(&root);
    a.font_tip.dirs = fonts(&root, with);
    (root, a)
}

fn picker(a: &mut App) -> String {
    key(a, KeyCode::Char('t'));
    assert_eq!(a.mode, Mode::Theme);
    draw(a, 160, 40).join("\n")
}

fn users(a: &App) -> Vec<String> {
    a.threads.get(CHAT_KEY).map(|t| t.items.iter().filter_map(|i| if let Item::User(s) = i { Some(s.clone()) } else { None }).collect()).unwrap_or_default()
}

// FONT-2, FONT-13, FONT-16, FONT-21: the tip while a font is missing, the button only with an
// agent, and the button to change the font when all fonts are on the machine.
#[test]
fn the_theme_picker_shows_the_font_tip() {
    let (root, mut a) = app_with("fonts-tip", &ALL[..7]);
    let s = picker(&mut a);
    assert!(s.contains("THEME") && s.contains("zendo setup fonts"), "{s}");
    assert!(!s.contains("ask the agent"), "no agent, no button: {s}");
    key(&mut a, KeyCode::Char('a'));
    assert!(!a.font_tip.asking, "no button, no question");
    key(&mut a, KeyCode::Esc);
    a.link.set_name("mock".into());
    let s = picker(&mut a);
    assert!(s.contains("zendo setup fonts") && s.contains("[ ask the agent · a ]"), "{s}");
    key(&mut a, KeyCode::Esc);
    a.font_tip.dirs = fonts(&root, &ALL);
    let s = picker(&mut a);
    assert!(!s.contains("zendo setup fonts") && s.contains("[ change the font · a ]"), "{s}");
    key(&mut a, KeyCode::Esc);
    a.cfg.offline = true;
    let s = picker(&mut a);
    assert!(!s.contains("zendo setup fonts") && !s.contains("change the font"), "{s}");
    let _ = std::fs::remove_dir_all(&root);
}

// FONT-5, FONT-6, FONT-14, FONT-15: `a` asks first; no request after no; one request after yes,
// which runs `zendo setup fonts` and changes the terminal only after the user agrees.
#[tokio::test]
async fn the_agent_gets_the_font_request_after_yes() {
    let (root, mut a) = app_with("fonts-request", &[]);
    a.ensure_agent();
    until(&mut a, |a| a.link.ready()).await;
    picker(&mut a);
    key(&mut a, KeyCode::Char('a'));
    let s = draw(&mut a, 160, 40).join("\n");
    assert!(a.font_tip.asking && s.contains("May zendo ask the agent"), "{s}");
    key(&mut a, KeyCode::Char('n'));
    assert!(!a.font_tip.asking && a.mode == Mode::Theme && users(&a).is_empty());
    key(&mut a, KeyCode::Char('a'));
    key(&mut a, KeyCode::Char('y'));
    assert_eq!(a.mode, Mode::Read, "the picker closes");
    let sent = users(&a);
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert!(sent[0].contains("zendo setup fonts") && sent[0].contains("only after I agree") && sent[0].contains("iA Writer Mono"), "{}", sent[0]);
    until(&mut a, |a| a.threads.get(CHAT_KEY).is_some_and(|t| t.busy.is_none())).await;
    let _ = std::fs::remove_dir_all(&root);
}

// FONT-14, FONT-15: a click on the font button asks, a click on yes sends; with all fonts on the
// machine the request does not run `zendo setup fonts`.
#[tokio::test]
async fn a_click_on_the_font_button_asks_first() {
    let (root, mut a) = app_with("fonts-click", &ALL);
    a.ensure_agent();
    until(&mut a, |a| a.link.ready()).await;
    picker(&mut a);
    let at = |a: &mut App, text: &str| {
        let rows = draw(a, 160, 40);
        rows.iter().enumerate().find_map(|(y, r)| r.find(text).map(|i| (r[..i].chars().count() as u16 + 2, y as u16))).unwrap_or_else(|| panic!("{text}: {rows:#?}"))
    };
    let click = |a: &mut App, (column, row): (u16, u16)| {
        a.on_mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column, row, modifiers: KeyModifiers::NONE });
        a.on_mouse(MouseEvent { kind: MouseEventKind::Up(MouseButton::Left), column, row, modifiers: KeyModifiers::NONE });
    };
    let button = at(&mut a, "[ change the font · a ]");
    click(&mut a, button);
    assert!(a.font_tip.asking, "the click asks first");
    let yes = at(&mut a, "[ yes · y ]");
    click(&mut a, yes);
    let sent = users(&a);
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert!(!sent[0].contains("Run `zendo setup fonts`") && sent[0].contains("only after I agree"), "{}", sent[0]);
    until(&mut a, |a| a.threads.get(CHAT_KEY).is_some_and(|t| t.busy.is_none())).await;
    let _ = std::fs::remove_dir_all(&root);
}

// FONT-2: without the recommended fonts and without the network, zendo shows the note and the
// themes as before.
#[test]
fn zendo_works_without_the_fonts() {
    let (root, mut a) = app_with("fonts-none", &[]);
    a.cfg.offline = true;
    let note = draw(&mut a, 160, 40).join("\n");
    assert!(note.contains("one"), "{note}");
    let s = picker(&mut a);
    assert!(["Ink", "Daylight", "Moss", "Mono"].iter().all(|t| s.contains(t)) && s.contains("zendo setup fonts"), "{s}");
    key(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Read);
    let _ = std::fs::remove_dir_all(&root);
}
