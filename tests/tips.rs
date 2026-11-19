//! The tips (docs/zendo/tips.md): the tip menu at start, after the music question at the first
//! start, its option, and a tip in the status bar.

mod common;
use common::rekey::{find, frame, screen};
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::path::{Path, PathBuf};
use zendo::app::{App, Focus};
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

/// TIPS-6, TIPS-9: the tip shows in the status bar only with room and without a message; without
/// room it does not show.
#[test]
fn the_status_bar_shows_a_tip_only_with_room_and_no_message() {
    let (root, cfg) = repo_with("tips-bar", "music = false\ntips = false\n");
    let mut a = start(&root, &cfg);
    let last = |a: &mut App, w: u16| draw(a, w, 30).last().cloned().unwrap();
    let row = last(&mut a, 260);
    assert!(row.contains(TIP), "{row}");
    let small = draw(&mut a, 80, 30);
    assert!(!small.iter().any(|r| r.contains(TIP)), "no room: the tip gives way (TIPS-9)");
    a.flash("saved");
    assert!(!last(&mut a, 260).contains(TIP), "a message comes first");
}

/// PANES-49: the status bar is one line, the last row of the screen, in every pane and mode, with a
/// message, the tip or a question of a file; the panes above it keep their height.
#[test]
fn the_status_bar_is_one_line_in_every_pane() {
    let (root, cfg) = repo_with("tips-one-line", "music = false\ntips = false\n");
    let mut a = start(&root, &cfg);
    a.toggle_tree();
    let rule = |rows: &[String]| rows.iter().rposition(|r| r.starts_with("────")).expect("the rule");
    let msg = "toggle_tree: alt-b → alt-t, saved in /home/user/.zendo/keymap.toml";
    for focus in [Focus::Note, Focus::Tree, Focus::Claude] {
        for flash in [false, true] {
            a.focus = focus;
            a.flash = None;
            if flash {
                a.flash(msg);
            }
            let rows = draw(&mut a, 100, 30);
            assert_eq!(rule(&rows), 28, "{focus:?}, message {flash}: the rule over a one-line bar");
            let bar = &rows[29];
            assert!(bar.contains("keys"), "{focus:?}: the key of the help stays: {bar}");
            assert!(!flash || bar.contains(msg), "{focus:?}: the message shows whole: {bar}");
        }
    }
    a.focus = Focus::Tree;
    a.files.expand_all();
    a.files.select_row(a.files.tree().rows().iter().position(|r| r.rel.ends_with("2026-11-18-a.md")).expect("the note in the tree"));
    key(&mut a, KeyCode::Char('m'));
    let rows = draw(&mut a, 100, 30);
    assert_eq!(rule(&rows), 28, "a question of a file takes the line of the status bar");
    assert!(rows[29].contains("MOVE"), "{}", rows[29]);
}
