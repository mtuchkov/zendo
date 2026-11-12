//! The keys help: f1 or alt-? in every pane and mode, and its hint in the status bar.

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use zendo::app::{Focus, Mode};
use zendo::keys::Pane;

// PANES-16, PANES-26: f1 shows the keys of the pane in focus or of the mode, and the status bar always names it.
#[tokio::test]
async fn f1_shows_the_keys_of_the_pane_from_everywhere() {
    let root = repo("help-keys", &[("docs/notes/2026-11-12-a.md", "# A\n\ntext\n")]);
    let mut a = app(&root);
    let bar = |a: &mut zendo::app::App| draw(a, 160, 30).into_iter().rev().find(|r| r.contains("   ")).unwrap_or_default();
    for (setup, pane) in [(0, Pane::Read), (1, Pane::Write), (2, Pane::Chat), (3, Pane::Notes), (4, Pane::Global)] {
        a.mode = Mode::Read;
        a.panel = false;
        a.focus = Focus::Note;
        match setup {
            1 => key(&mut a, KeyCode::Char('i')),
            2 => {
                a.open_panel();
                a.focus = Focus::Claude;
            }
            3 => a.open_notes(),
            4 => a.open_search(""),
            _ => {}
        }
        let b = bar(&mut a);
        assert!(b.contains("alt-? keys"), "{pane:?}: {b}");
        key(&mut a, KeyCode::F(1));
        assert_eq!((a.mode, a.help_pane), (Mode::Help, pane), "{pane:?}");
        key(&mut a, KeyCode::Esc);
        assert_ne!(a.mode, Mode::Help, "any key closes the help");
        if a.mode == Mode::Write {
            key(&mut a, KeyCode::Esc);
        }
    }
    a.on_key(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::ALT));
    assert_eq!(a.mode, Mode::Help, "alt-? works where f1 does not reach zendo");
    let _ = std::fs::remove_dir_all(&root);
}
