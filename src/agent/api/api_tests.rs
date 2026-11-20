use super::*;
use std::path::PathBuf;

/// `find` with the programs `progs` on the `PATH`, by name.
fn find_with(kind: Kind, progs: &[(&str, &str)]) -> Option<Api> {
    let which = |name: &str| progs.iter().find(|(n, _)| *n == name).map(|(_, p)| PathBuf::from(p));
    Api::find(kind, Some("haiku".into()), &which)
}

// NAMES-15, NAMES-16, NAMES-21: the light model is the `claude` CLI on the `PATH`, for a Claude
// agent only; without `claude`, or for another agent, there is none.
#[test]
fn the_light_model_is_claude_on_the_path() {
    let claude = [("claude", "/bin/claude")];
    let api = find_with(Kind::Claude, &claude).unwrap();
    assert_eq!((api.path, api.model.as_str(), api.effort), (PathBuf::from("/bin/claude"), "claude-haiku-5-5", None));
    assert_eq!(find_with(Kind::Claude, &[]), None, "no claude");
    assert_eq!(find_with(Kind::Codex, &claude), None, "Codex has no light model");
    assert_eq!(find_with(Kind::Other, &claude), None);
    assert_eq!(Api::find(Kind::Claude, None, &|_| Some("/bin/claude".into())), None, "no model");
}

// NAMES-22: zendo runs no other program for a credential: `find` looks only for `claude`, even
// when `ant` is on the `PATH`.
#[test]
fn zendo_reads_no_credential_of_another_program() {
    let asked = std::cell::RefCell::new(Vec::new());
    let which = |name: &str| {
        asked.borrow_mut().push(name.to_string());
        Some(PathBuf::from(format!("/bin/{name}")))
    };
    let api = Api::find(Kind::Claude, Some("haiku".into()), &which).unwrap();
    assert_eq!(api.path, PathBuf::from("/bin/claude"));
    assert_eq!(asked.into_inner(), ["claude"]);
}

// NAMES-18: short model names map to model identifiers; any other name passes as it is.
#[test]
fn short_model_names_map_to_model_identifiers() {
    assert_eq!(api_model("haiku"), "claude-haiku-5-5");
    assert_eq!(api_model("Sonnet"), "claude-sonnet-5-5");
    assert_eq!(api_model("opus"), "claude-opus-5-5");
    assert_eq!(api_model(" claude-3-x "), "claude-3-x");
    assert_eq!(api_model("luna"), "luna", "no OpenAI names");
}

// NAMES-21, CLAUDE-37: the content blocks of a question: the image in base64 first, then the prompt.
#[test]
fn the_content_has_the_image_and_the_prompt() {
    let img = Image { media: "image/png", data: "AAA=".into() };
    let c = content("name it", Some(&img));
    assert_eq!((c[0]["type"].as_str(), c[0]["source"]["data"].as_str()), (Some("image"), Some("AAA=")));
    assert_eq!((c[1]["type"].as_str(), c[1]["text"].as_str()), (Some("text"), Some("name it")));
    assert_eq!(content("name it", None).as_array().map(Vec::len), Some(1));
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

// FLOAT-19, FLOAT-23: the asker knows the questions without an answer, so a closed float does not
// ask again while its question runs.
#[test]
fn the_asker_keeps_the_open_questions_until_their_answers() {
    let mut asker = Asker::default();
    let api = Api { path: "/no/such/claude".into(), model: "m".into(), effort: None, env: Default::default() };
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
