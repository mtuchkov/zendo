//! The suggestions of the light model in the key float (docs/zendo/panes/key-hints.md, ADR 0012):
//! `haiku` with low effort through a mock API, without a wait for the user.

mod common;
use common::http::{mock_api, mock_api_after, mock_api_status};
use common::rekey::*;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use std::time::{Duration, Instant};
use zendo::agent::api::{Api, Credential, Vendor};
use zendo::keys::{Key, Pane};

fn answer(text: &str) -> serde_json::Value {
    serde_json::json!({"content": [{"type": "text", "text": text}]})
}

fn api(url: &str) -> Api {
    Api { vendor: Vendor::Anthropic, base: url.into(), model: "claude-haiku-5-5".into(), cred: Credential::Key("test-key".into()), effort: Some("low".into()) }
}

fn key(s: &str) -> Key {
    Key::parse(s).unwrap()
}

/// REKEY-6, REKEY-10, REKEY-11: a clash asks `haiku` with low effort and the keymap in effect; its
/// checked keys take the place of the keys of the rules.
#[test]
fn a_clash_asks_the_model_with_low_effort_and_the_keymap() {
    let server = mock_api(answer("alt-y\nalt-u"));
    let (_root, mut a) = app_with("hints-ask");
    a.rekey.api = Some(api(&server.url));
    open_float(&mut a);
    press(&mut a, KeyCode::Tab, KeyModifiers::NONE);
    assert!(!a.rekey.float().unwrap().from_model, "the rules first");
    assert!(tick_until(&mut a, 10, |a| a.rekey.float().is_some_and(|f| f.from_model)), "the model answers");
    assert_eq!(a.rekey.float().unwrap().suggestions(), &[key("alt-y"), key("alt-u")]);
    let reqs = server.requests.lock().unwrap();
    assert_eq!(reqs.len(), 1);
    let body = &reqs[0].body;
    assert_eq!(body["model"], "claude-haiku-5-5");
    assert_eq!(body["output_config"]["effort"], "low");
    let prompt = body["messages"][0]["content"][0]["text"].as_str().unwrap();
    assert!(prompt.contains("toggle_tree") && prompt.contains("[read]") && prompt.contains("next_pane = tab"), "{prompt}");
}

/// REKEY-4: the user sets a key while the model thinks, without a wait.
#[test]
fn the_user_presses_keys_while_the_model_thinks() {
    let server = mock_api_after(200, answer("alt-y"), Duration::from_secs(2));
    let (_root, mut a) = app_with("hints-wait");
    a.rekey.api = Some(api(&server.url));
    open_float(&mut a);
    let start = Instant::now();
    press(&mut a, KeyCode::Tab, KeyModifiers::NONE);
    press(&mut a, KeyCode::Char('t'), KeyModifiers::ALT);
    assert!(start.elapsed() < Duration::from_millis(500), "{:?}", start.elapsed());
    assert!(a.rekey.float().is_none());
    assert_eq!(a.keys.first(Pane::Global, "toggle_tree"), "alt-t");
}

/// REKEY-5: an answer that comes after the float closed changes nothing, also in the next float.
#[test]
fn a_late_answer_does_not_change_the_float() {
    let server = mock_api_after(200, answer("alt-y"), Duration::from_millis(300));
    let (_root, mut a) = app_with("hints-late");
    a.rekey.api = Some(api(&server.url));
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
    let server = mock_api(answer("x, tab, `alt-y`"));
    let (_root, mut a) = app_with("hints-check");
    a.rekey.api = Some(api(&server.url));
    open_float(&mut a);
    press(&mut a, KeyCode::Tab, KeyModifiers::NONE);
    assert!(tick_until(&mut a, 10, |a| a.rekey.float().is_some_and(|f| f.from_model)));
    assert_eq!(a.rekey.float().unwrap().suggestions(), &[key("alt-y")]);
}

/// REKEY-13: without the model, or when it fails, the keys of the rules stay and no error shows.
#[test]
fn without_the_model_the_rules_stay() {
    let server = mock_api_status(500, serde_json::json!({"error": {"message": "down"}}));
    let (_root, mut a) = app_with("hints-down");
    a.rekey.api = Some(api(&server.url));
    open_float(&mut a);
    press(&mut a, KeyCode::Tab, KeyModifiers::NONE);
    let rules = a.rekey.float().unwrap().suggestions().to_vec();
    assert!(!rules.is_empty());
    tick_until(&mut a, 1, |_| !server.requests.lock().unwrap().is_empty());
    tick_until(&mut a, 1, |_| false);
    assert_eq!(a.rekey.float().unwrap().suggestions(), rules.as_slice());
    assert!(a.flash.is_none(), "{:?}", a.flash);
    a.rekey.api = None;
    press(&mut a, KeyCode::Char('j'), KeyModifiers::NONE);
    assert!(!a.rekey.float().unwrap().suggestions().is_empty(), "the rules without an API");
}
