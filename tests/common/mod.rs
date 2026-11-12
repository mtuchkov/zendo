//! A zendo app in a test: a repository in a temp folder, the mock agent, a test terminal, keys.
#![allow(dead_code)]

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use zendo::app::{App, Config};

/// A repository with the given files.
pub fn repo(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-ui-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join(".git")).unwrap();
    for (path, text) in files {
        let p = d.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    d
}

pub fn app(root: &Path) -> App {
    App::new(Config {
        root: root.to_path_buf(),
        notes_dir: root.join("docs/notes"),
        agent: env!("CARGO_BIN_EXE_zendo-mock-agent").to_string(),
        agent_env: Default::default(),
        offline: false,
        notes_repo: None,
    })
    .unwrap()
}

/// Draw one frame on a test terminal; the screen as text, one line for each row.
pub fn draw(app: &mut App, w: u16, h: u16) -> Vec<String> {
    let mut term = Terminal::new(TestBackend::new(w, h)).unwrap();
    term.draw(|f| zendo::ui::draw(f, app)).unwrap();
    let buf = term.backend().buffer().clone();
    (0..h).map(|y| (0..w).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>()).collect()
}

pub fn key(app: &mut App, code: KeyCode) {
    app.on_key(KeyEvent::new(code, KeyModifiers::NONE));
}

pub fn ctrl(app: &mut App, c: char) {
    app.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL));
}

/// Type text, one key for each char.
pub fn typ(app: &mut App, text: &str) {
    for c in text.chars() {
        key(app, KeyCode::Char(c));
    }
}

/// Feed agent events and ticks until `done` holds.
pub async fn until(app: &mut App, done: impl Fn(&App) -> bool) {
    let end = Instant::now() + Duration::from_secs(20);
    while !done(app) {
        assert!(Instant::now() < end, "timeout");
        app.tick();
        match app.agent_events.as_mut() {
            Some(rx) => {
                if let Ok(Some(e)) = tokio::time::timeout(Duration::from_millis(50), rx.recv()).await {
                    app.on_agent(e);
                }
            }
            None => tokio::time::sleep(Duration::from_millis(50)).await,
        }
    }
}
