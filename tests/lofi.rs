//! The lofi music in the app (docs/zendo/lofi.md): the music button, the key, the question at the
//! first start, the pause without focus, and the chime of a turn. A test output plays nothing.

mod common;
use common::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use zendo::agent::Event;
use zendo::app::{App, session_key};
use zendo::music::{Music, Output, State};
use zendo::setup::config::UserConfig;

/// An app with its config file in the repository, not in the home folder, and the music of that
/// config on `output`.
fn app_with(name: &str, music: Option<bool>, output: Output) -> (std::path::PathBuf, App) {
    let root = repo(name, &[("docs/notes/2026-11-14-a.md", "# A\n\n- one\n")]);
    let config = root.join("config.toml");
    UserConfig { music, ..Default::default() }.save_to(&config).unwrap();
    let mut a = app(&root);
    a.attention.config = config.clone();
    a.attention.sound = true;
    a.music = Music::from_config(&UserConfig::load_from(&config));
    a.music.config = config.clone();
    a.start_music(output);
    a.files.set_branch(Some("main".into()));
    (root, a)
}

fn frame(a: &mut App) -> Buffer {
    let mut term = Terminal::new(TestBackend::new(160, 40)).unwrap();
    term.draw(|f| a.draw(f)).unwrap();
    term.backend().buffer().clone()
}

/// The first cell of `text` in the buffer: column and row.
fn find(buf: &Buffer, text: &str) -> Option<(u16, u16)> {
    (0..buf.area.height).find_map(|y| {
        let row: String = (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_string()).collect();
        row.find(text).map(|i| (row[..i].chars().count() as u16, y))
    })
}

fn screen(buf: &Buffer) -> String {
    (0..buf.area.height).map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>() + "\n").collect()
}

fn click(a: &mut App, (column, row): (u16, u16)) {
    a.on_mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column, row, modifiers: KeyModifiers::NONE });
    a.on_mouse(MouseEvent { kind: MouseEventKind::Up(MouseButton::Left), column, row, modifiers: KeyModifiers::NONE });
    a.settle_saves();
}

fn saved(root: &std::path::Path) -> String {
    std::fs::read_to_string(root.join("config.toml")).unwrap()
}

// LOFI-8, LOFI-10: the music button between the branch and the sound button stops and starts the
// music, blue while it plays and gray while it is stopped, and saves the setting.
#[test]
fn lofi_toggles() {
    let (root, mut a) = app_with("lofi-toggles", Some(true), Output::Test);
    let (blue, gray) = (a.theme().ext.doc, a.theme().dim);
    let buf = frame(&mut a);
    let at = find(&buf, "[music]").expect("the music button");
    let sound = find(&buf, "[sound]").expect("the sound button");
    assert!(at.1 == sound.1 && at.0 < sound.0, "left of the sound button");
    let branch = find(&buf, "main").expect("the branch");
    assert!(branch.1 == at.1 && branch.0 < at.0, "right of the branch");
    assert_eq!(buf[at].fg, blue);
    let mut seen = vec![];
    for _ in 0..3 {
        click(&mut a, at);
        seen.push((a.music.state(), frame(&mut a)[at].fg));
    }
    assert_eq!(seen, [(State::Stopped, gray), (State::Plays, blue), (State::Stopped, gray)]);
    assert!(saved(&root).contains("music = false"), "{}", saved(&root));
    let _ = std::fs::remove_dir_all(&root);
}

// LOFI-8: the music key, `alt-m` by default, does what the button does.
#[test]
fn alt_m_stops_and_starts_the_music() {
    let (root, mut a) = app_with("lofi-key", Some(true), Output::Test);
    a.on_key(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::ALT));
    assert_eq!(a.music.state(), State::Stopped);
    a.on_key(KeyEvent::new(KeyCode::Char('m'), KeyModifiers::ALT));
    assert_eq!(a.music.state(), State::Plays);
    assert!(saved(&root).contains("music = true"));
    let _ = std::fs::remove_dir_all(&root);
}

// LOFI-3: without an audio output zendo opens as usual; the button is gray and says why.
#[test]
fn lofi_without_audio() {
    let (root, mut a) = app_with("lofi-no-audio", Some(true), Output::Missing);
    let buf = frame(&mut a);
    let at = find(&buf, "[music]").expect("the music button");
    assert_eq!(buf[at].fg, a.theme().dim);
    click(&mut a, at);
    assert_eq!(a.music.state(), State::Unavailable);
    let text = screen(&frame(&mut a));
    assert!(text.contains("no audio output"), "{text}");
    let _ = std::fs::remove_dir_all(&root);
}

// LOFI-11, LOFI-12: at the first start the music plays and a float asks; the answer goes into the
// config, and the next start does not ask.
#[test]
fn lofi_asks_at_the_first_start() {
    let (root, mut a) = app_with("lofi-ask", None, Output::Test);
    assert!(a.music.sounds(), "the music plays while the float asks");
    let text = screen(&frame(&mut a));
    assert!(text.contains("lofi music") && text.contains("turn off"), "{text}");
    a.on_key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE));
    a.settle_saves();
    assert!(!a.music.sounds() && !a.music.asking);
    assert!(saved(&root).contains("music = false"), "{}", saved(&root));
    a.music = Music::from_config(&UserConfig::load_from(&root.join("config.toml")));
    a.music.config = root.join("config.toml");
    a.start_music(Output::Test);
    assert!(!a.music.asking && !screen(&frame(&mut a)).contains("lofi music"), "the next start does not ask");
    let _ = std::fs::remove_dir_all(&root);
}

// LOFI-12: a click on the answer to keep the music saves `music = true`.
#[test]
fn a_click_keeps_the_music_on() {
    let (root, mut a) = app_with("lofi-ask-click", None, Output::Test);
    let buf = frame(&mut a);
    let keep = find(&buf, "keep on").expect("the answer to keep it on");
    click(&mut a, keep);
    assert!(a.music.sounds() && !a.music.asking);
    assert!(saved(&root).contains("music = true"), "{}", saved(&root));
    let _ = std::fs::remove_dir_all(&root);
}

// LOFI-15: the music pauses while the terminal has no focus, and plays again with it.
#[test]
fn lofi_pauses_without_focus() {
    let (root, mut a) = app_with("lofi-focus", Some(true), Output::Test);
    a.on_focus(false);
    assert!(!a.music.sounds());
    assert_eq!(a.music.state(), State::Plays, "the button stays blue: the music is on");
    a.on_focus(true);
    assert!(a.music.sounds());
    let _ = std::fs::remove_dir_all(&root);
}

// LOFI-13, ATTN-1: while the music plays, the end of a turn is a chime in the music, not the
// system sound; without the music, the system sound rings.
#[test]
fn lofi_chime_in_place_of_the_bell() {
    let (root, mut a) = app_with("lofi-chime", Some(true), Output::Test);
    a.on_agent(Event::TurnEnd { key: session_key(3), stop: "end_turn".into() });
    assert_eq!((a.attention.bells, a.music.chimes()), (0, 1));
    a.toggle_music();
    a.on_agent(Event::TurnEnd { key: session_key(2), stop: "end_turn".into() });
    assert_eq!((a.attention.bells, a.music.chimes()), (1, 1));
    let _ = std::fs::remove_dir_all(&root);
}

// LOFI-3, ATTN-1, LOFI-13: when the audio output dies, as a sound server that restarts, the button
// turns gray and the end of a turn rings the system sound again.
#[test]
fn a_dead_output_rings_the_bell_again() {
    let (root, mut a) = app_with("lofi-dead", Some(true), Output::Test);
    a.music.lose_output();
    assert_eq!(a.music.state(), State::Unavailable);
    a.on_agent(Event::TurnEnd { key: session_key(3), stop: "end_turn".into() });
    assert_eq!((a.attention.bells, a.music.chimes()), (1, 0));
    let _ = std::fs::remove_dir_all(&root);
}

// LOFI-8, CMD-17, CMD-19: a config with a line with an error: zendo asks as at each first start,
// and the answer adds one line; the line with the error stays.
#[test]
fn a_line_with_an_error_stays_when_the_music_is_saved() {
    let (root, mut a) = app_with("lofi-broken", Some(true), Output::Test);
    let config = root.join("config.toml");
    std::fs::write(&config, "width = \"wide\"\nsound = true\n").unwrap();
    a.music = Music::from_config(&UserConfig::load_from(&config));
    a.music.config = config.clone();
    a.start_music(Output::Test);
    assert!(a.music.asking, "no setting `music`: the first start");
    a.answer_music(false);
    a.settle_saves();
    assert_eq!(std::fs::read_to_string(&config).unwrap(), "width = \"wide\"\nsound = true\nmusic = false\n");
    let _ = std::fs::remove_dir_all(&root);
}
