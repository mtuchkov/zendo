//! The tips (docs/zendo/tips.md): the tip menu at start, after the music question at the first
//! start, its option, and a tip in the status bar.

mod common;
use common::rekey::{find, frame, screen};
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::path::{Path, PathBuf};
use zendo::app::App;
use zendo::music::{Music, Output};
use zendo::setup::config::UserConfig;

const TIP: &str = "If a key does not work for you, double-click it to change it.";

/// An app of a repository with the config `config`, started as zendo starts: the music, then the
/// tips.
fn start(root: &Path, config: &Path) -> App {
    let mut a = app(root);
    a.music = Music::from_config(&UserConfig::load_from(config));
    a.music.config = config.to_path_buf();
    a.tips.config = config.to_path_buf();
    a.start_music(Output::Test);
    a.start_tips();
    a.tick();
    a
}

fn repo_with(name: &str, config: &str) -> (PathBuf, PathBuf) {
    let root = repo(name, &[("docs/notes/2026-11-18-a.md", "# A\n\n- one\n")]);
    let cfg = root.join("config.toml");
    std::fs::write(&cfg, config).unwrap();
    (root, cfg)
}

fn press(a: &mut App, code: KeyCode) {
    a.on_key(KeyEvent::new(code, KeyModifiers::NONE));
}

fn click(a: &mut App, (column, row): (u16, u16)) {
    a.on_mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column, row, modifiers: KeyModifiers::NONE });
    a.on_mouse(MouseEvent { kind: MouseEventKind::Up(MouseButton::Left), column, row, modifiers: KeyModifiers::NONE });
}

/// TIPS-1 to TIPS-4, TIPS-7: the tip menu comes after the music question; only `[OK]` closes it;
/// the option goes to the config, not to `.zendo/db.json`; the tip in the status bar stays.
#[test]
fn the_tip_menu_follows_the_music_question() {
    let (root, cfg) = repo_with("tips-first", "");
    let mut a = start(&root, &cfg);
    assert!(a.music.asking && !a.tips.open, "the music question first");
    press(&mut a, KeyCode::Char('y'));
    a.tick();
    assert!(a.tips.open, "the tip menu after the answer");
    assert!(screen(&frame(&mut a)).contains(TIP));
    press(&mut a, KeyCode::Esc);
    press(&mut a, KeyCode::Char('q'));
    assert!(a.tips.open, "only [OK] closes it");
    let db = std::fs::read_to_string(root.join(".zendo/db.json")).unwrap_or_default();
    press(&mut a, KeyCode::Char(' '));
    let buf = frame(&mut a);
    assert!(screen(&buf).contains("[x] Do not show tips at start"), "{}", screen(&buf));
    click(&mut a, find(&buf, "[ OK ]", None).expect("[ OK ]"));
    assert!(!a.tips.open);
    assert_eq!(UserConfig::load_from(&cfg).tips, Some(false));
    assert_eq!(std::fs::read_to_string(root.join(".zendo/db.json")).unwrap_or_default(), db);
    let mut b = start(&root, &cfg);
    assert!(!b.tips.open, "no tip menu at the next start");
    assert!(draw(&mut b, 260, 30).last().unwrap().contains(TIP), "the status bar still shows the tip");
}

/// TIPS-1, TIPS-3: at a later start the tip menu comes at once; `enter` closes it and writes nothing.
#[test]
fn the_tip_menu_comes_at_each_start() {
    let (root, cfg) = repo_with("tips-each", "music = false\n");
    let mut a = start(&root, &cfg);
    assert!(a.tips.open);
    press(&mut a, KeyCode::Enter);
    assert!(!a.tips.open);
    assert_eq!(UserConfig::load_from(&cfg).tips, None);
}

/// TIPS-6: the tip shows in the status bar only with room and without a message.
#[test]
fn the_status_bar_shows_a_tip_only_with_room_and_no_message() {
    let (root, cfg) = repo_with("tips-bar", "music = false\ntips = false\n");
    let mut a = start(&root, &cfg);
    let last = |a: &mut App, w: u16| draw(a, w, 30).last().cloned().unwrap();
    let row = last(&mut a, 260);
    assert!(row.contains(TIP), "{row}");
    let small = draw(&mut a, 80, 30);
    assert!(!small[28].contains(TIP) && small[29].contains(TIP), "no room: the tip goes below (TIPS-9)");
    a.flash("saved");
    assert!(!last(&mut a, 260).contains(TIP), "a message comes first");
}

/// TIPS-9, PANES-49: on a small screen, the tip, then a message, take a line below the status bar;
/// the status bar keeps the key hints.
#[test]
fn on_a_small_screen_the_tip_and_the_message_go_below_the_hints() {
    let (root, cfg) = repo_with("tips-small", "music = false\ntips = false\n");
    let mut a = start(&root, &cfg);
    let rows = draw(&mut a, 100, 30);
    let (status, below) = (&rows[28], &rows[29]);
    assert!(status.contains("READ") && status.contains("keys"), "{status}");
    assert!(!status.contains(TIP));
    assert!(below.contains(TIP), "{below}");
    let hints = status.matches("   ").count();
    let msg = "toggle_tree: alt-b → alt-t, saved in /home/user/.zendo/keymap.toml";
    a.flash(msg);
    let rows = draw(&mut a, 100, 30);
    assert!(rows[29].contains(msg) && !rows[29].contains(TIP), "{}", rows[29]);
    assert!(rows[28].contains("READ") && !rows[28].contains("saved in"), "{}", rows[28]);
    assert!(rows[28].matches("   ").count() >= hints, "the message takes no hint away: {}", rows[28]);
    let wide = draw(&mut a, 260, 30);
    assert!(wide[29].contains(msg) && wide[29].contains("READ"), "a wide screen keeps one line: {}", wide[29]);
}
