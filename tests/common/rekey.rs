//! The key float in a test: an app with its config and keymap file in the repository, frames,
//! clicks and keys (docs/zendo/panes/key-change.md).
use super::*;
pub use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
pub use zendo::app::App;

/// An app with its config in the repository, a keymap file with a comment, and the music question
/// answered.
pub fn app_with(name: &str) -> (PathBuf, App) {
    let root = repo(name, &[("docs/notes/2026-11-18-a.md", "# A\n\n- one\n")]);
    let keymap = root.join("keymap.toml");
    std::fs::write(&keymap, "# my keys\n[global]\ntoggle_tree = [\"alt-b\", \"ctrl-shift-b\"]  # file tree\n").unwrap();
    let config = root.join("config.toml");
    std::fs::write(&config, format!("music = false\ntips = false\nkeymap = \"{}\"\n", keymap.display())).unwrap();
    let mut a = app(&root);
    a.rekey.config = config;
    a.music.asked = true;
    (root, a)
}

pub fn frame(a: &mut App) -> Buffer {
    let mut term = Terminal::new(TestBackend::new(170, 40)).unwrap();
    term.draw(|f| a.draw(f)).unwrap();
    term.backend().buffer().clone()
}

/// The first cell of `text` on the row `y`, or on any row.
pub fn find(buf: &Buffer, text: &str, y: Option<u16>) -> Option<(u16, u16)> {
    let rows = y.map_or(0..buf.area.height, |y| y..y + 1);
    rows.into_iter().find_map(|y| {
        let row: String = (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_string()).collect();
        row.find(text).map(|i| (row[..i].chars().count() as u16, y))
    })
}

pub fn screen(buf: &Buffer) -> String {
    (0..buf.area.height).map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>() + "\n").collect()
}

pub fn click(a: &mut App, (column, row): (u16, u16)) {
    a.on_mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column, row, modifiers: KeyModifiers::NONE });
    a.on_mouse(MouseEvent { kind: MouseEventKind::Up(MouseButton::Left), column, row, modifiers: KeyModifiers::NONE });
}

pub fn press(a: &mut App, code: KeyCode, m: KeyModifiers) {
    a.on_key(KeyEvent::new(code, m));
}

pub fn status_row(buf: &Buffer) -> u16 {
    let row = |y: u16| (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>();
    (0..buf.area.height).rev().find(|y| row(*y).starts_with("────")).expect("the rule") + 1
}

/// Two clicks on `alt-b` in the status bar: the key float of `toggle_tree`.
pub fn open_float(a: &mut App) -> Buffer {
    let buf = frame(a);
    let at = find(&buf, "alt-b", Some(status_row(&buf))).expect("alt-b in the status bar");
    click(a, at);
    frame(a);
    click(a, at);
    frame(a)
}


/// The keys of the event of `k`.
pub fn mods(k: zendo::keys::Key) -> KeyModifiers {
    let mut m = KeyModifiers::NONE;
    m.set(KeyModifiers::CONTROL, k.ctrl);
    m.set(KeyModifiers::ALT, k.alt);
    m.set(KeyModifiers::SHIFT, k.shift);
    m
}

/// Tick the app until `done` holds, for at most `secs` seconds.
pub fn tick_until(a: &mut App, secs: u64, done: impl Fn(&App) -> bool) -> bool {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(secs);
    while std::time::Instant::now() < end {
        a.tick();
        if done(a) {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    false
}
