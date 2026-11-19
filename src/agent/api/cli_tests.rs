use super::*;
use std::path::PathBuf;

/// A mock `claude` in a folder of its own: `auth status` prints `status`; `-p` with the flags of
/// `args` prints the stream-json lines of an answer `answer`, or fails when the input has no `ask`.
fn mock_claude(name: &str, status: &str, answer: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zendo-claude-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("claude");
    let script = format!(
        "#!/bin/sh\n[ \"$1 $2\" = \"auth status\" ] && {{ echo '{status}'; exit 0; }}\n\
         case \"$*\" in *'--tools  --strict-mcp-config --setting-sources  '*) ;; *) exit 7 ;; esac\n\
         read line\necho \"$line\" > \"$0.in\"\n\
         echo '{{\"type\":\"system\",\"subtype\":\"init\"}}'\n\
         echo '{{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"result\":\"{answer}\"}}'\n"
    );
    std::fs::write(&path, script).unwrap();
    let mut p = std::fs::metadata(&path).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut p, 0o755);
    std::fs::set_permissions(&path, p).unwrap();
    path
}

fn api(path: &std::path::Path) -> Api {
    Api { path: path.into(), model: "claude-haiku-5-5".into(), effort: None }
}

// NAMES-21: `claude -p` gets one user message with the prompt and the image, and no tools, no
// settings, no MCP servers, no slash commands, no saved session; the result line is the answer.
#[cfg(unix)]
#[test]
fn claude_p_answers_with_no_tools_and_no_settings() {
    let path = mock_claude("ask", "{\"loggedIn\": true}", "parser, lexer");
    let img = Image { media: "image/png", data: "AAA=".into() };
    assert_eq!(ask(&api(&path), "name it", Some(&img)).as_deref(), Ok("parser, lexer"));
    let sent: Value = serde_json::from_str(&std::fs::read_to_string(path.with_extension("in")).unwrap()).unwrap();
    assert_eq!(sent["type"], "user");
    assert_eq!(sent["message"]["content"][0]["source"]["data"], "AAA=");
    assert_eq!(sent["message"]["content"][1]["text"], "name it");
    let a = args("claude-haiku-5-5", None);
    for flag in ["-p", "--strict-mcp-config", "--disable-slash-commands", "--no-session-persistence", "--system-prompt"] {
        assert!(a.iter().any(|x| x == flag), "{flag}: {a:?}");
    }
    assert!(a.windows(2).any(|w| w == ["--model", "claude-haiku-5-5"]) && a.windows(2).any(|w| w == ["--tools", ""]));
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

// REKEY-6: a question with an effort asks `claude -p` for it; without one, no `--effort`.
#[test]
fn an_effort_goes_into_the_arguments() {
    assert!(args("claude-haiku-5-5", Some("low")).windows(2).any(|w| w == ["--effort", "low"]));
    assert!(!args("claude-haiku-5-5", None).iter().any(|a| a == "--effort"));
}

// NAMES-21: `claude auth status` decides whether the CLI is a credential.
#[cfg(unix)]
#[test]
fn claude_auth_status_tells_the_login() {
    let yes = mock_claude("in", "{\"loggedIn\": true, \"authMethod\": \"claude.ai\"}", "x");
    let no = mock_claude("out", "{\"loggedIn\": false, \"authMethod\": \"none\"}", "x");
    assert!(api(&yes).usable());
    assert!(!api(&no).usable());
    assert!(!api(std::path::Path::new("/no/such/claude")).usable());
    [yes, no].iter().for_each(|p| drop(std::fs::remove_dir_all(p.parent().unwrap())));
}

// NAMES-19, NAMES-21: an error result is an error with its text.
#[test]
fn an_error_result_is_an_error() {
    let ok = "{\"type\":\"system\"}\n{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":false,\"result\":\"cache\"}\n";
    assert_eq!(result(ok).as_deref(), Ok("cache"));
    let auth = "{\"type\":\"result\",\"subtype\":\"success\",\"is_error\":true,\"result\":\"Authentication error\"}";
    assert_eq!(result(auth), Err("claude: Authentication error".into()));
    assert_eq!(result("{\"type\":\"result\",\"subtype\":\"error_max_turns\"}"), Err("claude: error_max_turns".into()));
    assert_eq!(result(""), Err("claude: no result".into()));
}
