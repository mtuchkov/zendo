use super::*;
use crate::setup::config::Kind;
use std::collections::HashMap;
use std::path::PathBuf;

fn env(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

/// `find` with the variables `pairs` and the programs `progs` (name, path) on the `PATH`.
fn find_with(kind: Kind, pairs: &[(&str, &str)], progs: &[(&str, &str)]) -> Option<Api> {
    let e = env(pairs);
    let which = |n: &str| progs.iter().find(|(k, _)| *k == n).map(|(_, p)| PathBuf::from(p));
    Api::find(kind, Some("haiku".into()), &|k| e.get(k).cloned(), &which)
}

fn find(kind: Kind, pairs: &[(&str, &str)], ant: Option<&str>) -> Option<Api> {
    let progs: Vec<(&str, &str)> = ant.map(|a| ("ant", a)).into_iter().collect();
    find_with(kind, pairs, &progs)
}

// NAMES-15, NAMES-16: the API follows the kind of the configured agent, and needs its credential.
#[test]
fn the_api_follows_the_configured_agent() {
    let claude = find(Kind::Claude, &[("ANTHROPIC_API_KEY", "a"), ("OPENAI_API_KEY", "o")], None).unwrap();
    assert_eq!((claude.vendor, claude.cred.clone()), (Vendor::Anthropic, Credential::Key("a".into())));
    assert_eq!(claude.base, "https://api.anthropic.com");
    let codex = find(Kind::Codex, &[("ANTHROPIC_API_KEY", "a"), ("OPENAI_API_KEY", "o")], None).unwrap();
    assert_eq!((codex.vendor, codex.cred.clone()), (Vendor::OpenAi, Credential::Key("o".into())));
    assert_eq!(codex.base, "https://api.openai.com/v1");
    let codex = find(Kind::Codex, &[("CODEX_API_KEY", "c")], None).unwrap();
    assert_eq!(codex.cred, Credential::Key("c".into()), "CODEX_API_KEY after OPENAI_API_KEY");
    assert!(find(Kind::Claude, &[("OPENAI_API_KEY", "o")], None).is_none(), "no key of Claude");
    assert!(find(Kind::Codex, &[("ANTHROPIC_API_KEY", "a")], None).is_none(), "no key of Codex");
    assert!(find(Kind::Other, &[("ANTHROPIC_API_KEY", "a"), ("OPENAI_API_KEY", "o")], None).is_none(), "another agent");
    assert!(find(Kind::Claude, &[("ANTHROPIC_API_KEY", " ")], None).is_none(), "an empty key");
    let e = env(&[("ANTHROPIC_API_KEY", "a")]);
    assert!(Api::find(Kind::Claude, None, &|k| e.get(k).cloned(), &|_| None).is_none(), "no model");
}

// NAMES-15: the agents take `ANTHROPIC_BASE_URL` and `OPENAI_BASE_URL`; so does zendo.
#[test]
fn the_base_url_comes_from_the_environment() {
    let a = find(Kind::Claude, &[("ANTHROPIC_API_KEY", "a"), ("ANTHROPIC_BASE_URL", "http://127.0.0.1:9/")], None).unwrap();
    assert_eq!(a.url(), "http://127.0.0.1:9/v1/messages");
    let o = find(Kind::Codex, &[("OPENAI_API_KEY", "o"), ("OPENAI_BASE_URL", "http://127.0.0.1:9/v1")], None).unwrap();
    assert_eq!(o.url(), "http://127.0.0.1:9/v1/responses");
}

// NAMES-20: without `ANTHROPIC_API_KEY`, the `ant` CLI is the credential of Claude; never of Codex.
#[test]
fn the_ant_cli_is_the_credential_of_claude_without_a_key() {
    let a = find(Kind::Claude, &[], Some("/bin/ant")).unwrap();
    assert_eq!(a.cred, Credential::Ant("/bin/ant".into()));
    assert_eq!(find(Kind::Claude, &[("ANTHROPIC_API_KEY", "a")], Some("/bin/ant")).unwrap().cred, Credential::Key("a".into()), "the key first");
    assert!(find(Kind::Codex, &[], Some("/bin/ant")).is_none());
}

// NAMES-21: without a key and without `ant`, the `claude` CLI is the credential of Claude; never of
// Codex.
#[test]
fn the_claude_cli_is_the_credential_of_claude_last() {
    let both = [("ant", "/bin/ant"), ("claude", "/bin/claude")];
    assert_eq!(find_with(Kind::Claude, &[], &both[1..]).unwrap().cred, Credential::Claude("/bin/claude".into()));
    assert_eq!(find_with(Kind::Claude, &[], &both).unwrap().cred, Credential::Ant("/bin/ant".into()), "ant first");
    assert_eq!(find_with(Kind::Claude, &[("ANTHROPIC_API_KEY", "a")], &both).unwrap().cred, Credential::Key("a".into()), "the key first");
    assert!(find_with(Kind::Codex, &[], &both).is_none());
}

/// A script in a folder of its own that prints `out` and ends with `code`, as a mock `ant`.
fn mock_ant(name: &str, out: &str, code: i32) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zendo-ant-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("ant");
    std::fs::write(&path, format!("#!/bin/sh\n[ \"$*\" = \"auth print-credentials --access-token\" ] || exit 9\necho '{out}'\nexit {code}\n")).unwrap();
    let mut p = std::fs::metadata(&path).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut p, 0o755);
    std::fs::set_permissions(&path, p).unwrap();
    path
}

// NAMES-20 (`the_ant_sign_in_is_the_claude_credential`): the token of `ant auth print-credentials
// --access-token` goes as a bearer token; an `ant` that fails gives no credential.
#[cfg(unix)]
#[test]
fn the_ant_sign_in_is_the_claude_credential() {
    let ok = mock_ant("ok", "sk-ant-oat01-test", 0);
    let api = Api { vendor: Vendor::Anthropic, base: "http://x".into(), model: "m".into(), cred: Credential::Ant(ok.clone()), effort: None };
    assert_eq!(api.token().as_deref(), Ok("sk-ant-oat01-test"));
    assert!(api.usable());
    let h = api.headers("sk-ant-oat01-test");
    assert!(h.contains(&("authorization".into(), "Bearer sk-ant-oat01-test".into())) && !h.iter().any(|(k, _)| k == "x-api-key"), "{h:?}");
    let bad = mock_ant("bad", "not signed in", 1);
    assert!(!Api { cred: Credential::Ant(bad), ..api }.usable());
    let _ = std::fs::remove_dir_all(ok.parent().unwrap());
}

// NAMES-15: the headers of each API: the key of Anthropic as `x-api-key`, of OpenAI as a bearer.
#[test]
fn the_headers_carry_the_credential_of_the_vendor() {
    let a = find(Kind::Claude, &[("ANTHROPIC_API_KEY", "a")], None).unwrap();
    let h = a.headers("a");
    assert!(h.contains(&("x-api-key".into(), "a".into())) && h.contains(&("anthropic-version".into(), "2023-06-01".into())), "{h:?}");
    let o = find(Kind::Codex, &[("OPENAI_API_KEY", "o")], None).unwrap();
    assert!(o.headers("o").contains(&("authorization".into(), "Bearer o".into())));
}

// NAMES-18: short model names map to API identifiers; any other name passes as it is.
#[test]
fn short_model_names_map_to_api_identifiers() {
    assert_eq!(api_model(Vendor::Anthropic, "haiku"), "claude-haiku-5-5");
    assert_eq!(api_model(Vendor::Anthropic, "Sonnet"), "claude-sonnet-5-5");
    assert_eq!(api_model(Vendor::Anthropic, "opus"), "claude-opus-5-5");
    assert_eq!(api_model(Vendor::Anthropic, "claude-haiku-4-5"), "claude-haiku-4-5");
    assert_eq!(api_model(Vendor::OpenAi, "luna"), "gpt-6-luna");
    assert_eq!(api_model(Vendor::OpenAi, "sol"), "gpt-6-sol");
    assert_eq!(api_model(Vendor::OpenAi, "gpt-5.6-luna"), "gpt-5.6-luna");
    assert_eq!(find(Kind::Claude, &[("ANTHROPIC_API_KEY", "a")], None).unwrap().model, "claude-haiku-5-5");
}

// NAMES-15, CLAUDE-37: the body of each API, with the prompt and an image in base64.
#[test]
fn the_bodies_have_the_prompt_and_the_image() {
    let img = Some(Image { media: "image/png", data: "AAA=".into() });
    let a = Api { vendor: Vendor::Anthropic, base: "b".into(), model: "claude-haiku-5-5".into(), cred: Credential::Key("k".into()), effort: None };
    let b = a.body("name it", img.as_ref());
    assert_eq!(b["model"], "claude-haiku-5-5");
    assert!(b["max_tokens"].as_u64().is_some());
    assert_eq!(b["messages"][0]["content"][0]["source"]["data"], "AAA=");
    assert_eq!(b["messages"][0]["content"][0]["source"]["media_type"], "image/png");
    assert_eq!(b["messages"][0]["content"][1]["text"], "name it");
    let o = Api { vendor: Vendor::OpenAi, model: "gpt-6-luna".into(), ..a };
    let b = o.body("name it", img.as_ref());
    assert_eq!(b["model"], "gpt-6-luna");
    assert_eq!(b["input"][0]["content"][0]["text"], "name it");
    assert_eq!(b["input"][0]["content"][1]["image_url"], "data:image/png;base64,AAA=");
    assert_eq!(o.body("name it", None)["input"][0]["content"].as_array().map(Vec::len), Some(1));
}

// CLAUDE-37: the media type of an image from its extension.
#[test]
fn the_media_type_comes_from_the_extension() {
    assert_eq!(media_type(std::path::Path::new("a.PNG")), "image/png");
    assert_eq!(media_type(std::path::Path::new("a.jpg")), "image/jpeg");
    assert_eq!(media_type(std::path::Path::new("a.jpeg")), "image/jpeg");
    assert_eq!(media_type(std::path::Path::new("a.gif")), "image/gif");
    assert_eq!(media_type(std::path::Path::new("a.webp")), "image/webp");
    assert_eq!(media_type(std::path::Path::new("a")), "image/png");
}

// NAMES-15, NAMES-19: the text of an answer of each API; the message of an error.
#[test]
fn the_answers_of_both_apis_give_their_text() {
    let a = serde_json::json!({"content": [{"type": "text", "text": "parser, lexer"}]});
    assert_eq!(answer_text(Vendor::Anthropic, &a).as_deref(), Some("parser, lexer"));
    let o = serde_json::json!({"output": [{"type": "reasoning", "summary": []}, {"type": "message", "content": [{"type": "output_text", "text": "cache"}]}]});
    assert_eq!(answer_text(Vendor::OpenAi, &o).as_deref(), Some("cache"));
    assert_eq!(answer_text(Vendor::OpenAi, &serde_json::json!({"output": []})), None);
    let e = serde_json::json!({"type": "error", "error": {"type": "authentication_error", "message": "invalid x-api-key"}});
    assert_eq!(error_text(401, &e), "HTTP 401: invalid x-api-key");
    assert_eq!(error_text(500, &serde_json::Value::Null), "HTTP 500");
}

// FLOAT-19, FLOAT-23: the asker knows the questions without an answer, so a closed float does not
// ask again while its question runs.
#[test]
fn the_asker_keeps_the_open_questions_until_their_answers() {
    let mut asker = Asker::default();
    let api = Api { vendor: Vendor::Anthropic, base: "http://x".into(), model: "m".into(), cred: Credential::Claude("/no/such/claude".into()), effort: None };
    let q = Question::Image("/tmp/a.png".into());
    asker.ask(&api, q.clone(), "name it".into(), None);
    assert_eq!(asker.open(), std::slice::from_ref(&q));
    let end = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let mut got = Vec::new();
    while got.is_empty() && std::time::Instant::now() < end {
        got = asker.answers();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(matches!(&got[..], [(a, Err(_))] if *a == q), "{got:?}");
    assert!(asker.open().is_empty());
}

// REKEY-6: a question with an effort asks for it: `output_config` for Anthropic, `reasoning` for
// OpenAI; without one, neither.
#[test]
fn an_effort_goes_into_the_body() {
    let a = Api { vendor: Vendor::Anthropic, base: "b".into(), model: "claude-haiku-5-5".into(), cred: Credential::Key("k".into()), effort: Some("low".into()) };
    assert_eq!(a.body("keys", None)["output_config"]["effort"], "low");
    let o = Api { vendor: Vendor::OpenAi, ..a.clone() };
    assert_eq!(o.body("keys", None)["reasoning"]["effort"], "low");
    let none = Api { effort: None, ..a };
    assert!(none.body("keys", None).get("output_config").is_none());
}
