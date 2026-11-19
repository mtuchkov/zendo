//! The suggestions of the light model in the key float (docs/zendo/panes/key-hints.md, ADR 0013):
//! `haiku` with low effort through a mock `claude -p`, without a wait for the user.

use crate::common;
use common::claude::{MockClaude, mock_claude, mock_claude_with};
use common::rekey::*;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use std::time::{Duration, Instant};
use zendo::agent::api::Api;
use zendo::keys::{Key, Pane};

/// The light model of the key hints at the mock: `haiku` with low effort (REKEY-6).
fn api(cli: &MockClaude) -> Api {
    Api { effort: Some("low".into()), ..cli.api("haiku") }
}

fn key(s: &str) -> Key {
    Key::parse(s).unwrap()
}

/// REKEY-6, REKEY-10, REKEY-11: a clash asks `haiku` with low effort and the keymap in effect; its
/// checked keys take the place of the keys of the rules.
#[test]
fn a_clash_asks_the_model_with_low_effort_and_the_keymap() {
    let (root, mut a) = app_with("hints-ask");
    let cli = mock_claude(&root.join(".claude-mock"), "alt-y\nalt-u");
    a.rekey.api = Some(api(&cli));
    open_float(&mut a);
    press(&mut a, KeyCode::Tab, KeyModifiers::NONE);
    assert!(!a.rekey.float().unwrap().from_model, "the rules first");
    assert!(tick_until(&mut a, 10, |a| a.rekey.float().is_some_and(|f| f.from_model)), "the model answers");
    assert_eq!(a.rekey.float().unwrap().suggestions(), &[key("alt-y"), key("alt-u")]);
    let asked = cli.asked();
    assert_eq!(asked.len(), 1);
    assert!(asked[0].args.contains("--model claude-haiku-5-5") && asked[0].args.contains("--effort low"), "{}", asked[0].args);
    let prompt = asked[0].prompt();
    assert!(prompt.contains("toggle_tree") && prompt.contains("[read]") && prompt.contains("next_pane = tab"), "{prompt}");
}

/// REKEY-4: the user sets a key while the model thinks, without a wait. The model answers after
/// `THINKS`; the keys take far less even on a busy machine, so a wait for the model would show.
#[test]
fn the_user_presses_keys_while_the_model_thinks() {
    const THINKS: f32 = 20.0;
    let (root, mut a) = app_with("hints-wait");
    let cli = mock_claude_with(&root.join(".claude-mock"), Ok("alt-y"), THINKS);
    a.rekey.api = Some(api(&cli));
    open_float(&mut a);
    let start = Instant::now();
    press(&mut a, KeyCode::Tab, KeyModifiers::NONE);
    press(&mut a, KeyCode::Char('t'), KeyModifiers::ALT);
    assert!(start.elapsed() < Duration::from_secs_f32(THINKS / 2.0), "{:?}", start.elapsed());
    assert!(a.rekey.float().is_none());
    assert_eq!(a.keys.first(Pane::Global, "toggle_tree"), "alt-t");
}

/// REKEY-5: an answer that comes after the float closed changes nothing, also in the next float.
#[test]
fn a_late_answer_does_not_change_the_float() {
    let (root, mut a) = app_with("hints-late");
    let cli = mock_claude_with(&root.join(".claude-mock"), Ok("alt-y"), 0.3);
    a.rekey.api = Some(api(&cli));
    open_float(&mut a);
    press(&mut a, KeyCode::Tab, KeyModifiers::NONE);
    press(&mut a, KeyCode::Esc, KeyModifiers::NONE);
    open_float(&mut a);
    assert!(!tick_until(&mut a, 1, |a| a.rekey.float().is_some_and(|f| f.from_model)), "the old answer went");
    assert!(a.rekey.float().unwrap().suggestions().is_empty());
}

/// REKEY-7: a key of the model that clashes is not shown.
#[test]
fn a_suggestion_that_clashes_is_not_shown() {
    let (root, mut a) = app_with("hints-check");
    let cli = mock_claude(&root.join(".claude-mock"), "x, tab, `alt-y`");
    a.rekey.api = Some(api(&cli));
    open_float(&mut a);
    press(&mut a, KeyCode::Tab, KeyModifiers::NONE);
    assert!(tick_until(&mut a, 10, |a| a.rekey.float().is_some_and(|f| f.from_model)));
    assert_eq!(a.rekey.float().unwrap().suggestions(), &[key("alt-y")]);
}

/// REKEY-13: without the model, or when it fails, the keys of the rules stay and no error shows.
#[test]
fn without_the_model_the_rules_stay() {
    let (root, mut a) = app_with("hints-down");
    let cli = mock_claude_with(&root.join(".claude-mock"), Err("down"), 0.0);
    a.rekey.api = Some(api(&cli));
    open_float(&mut a);
    press(&mut a, KeyCode::Tab, KeyModifiers::NONE);
    let rules = a.rekey.float().unwrap().suggestions().to_vec();
    assert!(!rules.is_empty());
    tick_until(&mut a, 1, |_| !cli.asked().is_empty());
    tick_until(&mut a, 1, |_| false);
    assert_eq!(a.rekey.float().unwrap().suggestions(), rules.as_slice());
    assert!(a.flash.is_none(), "{:?}", a.flash);
    a.rekey.api = None;
    press(&mut a, KeyCode::Char('j'), KeyModifiers::NONE);
    assert!(!a.rekey.float().unwrap().suggestions().is_empty(), "the rules without an API");
}
