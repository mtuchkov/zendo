//! A mock `claude` CLI for the questions of zendo to the light model (NAMES-21): `auth status`
//! reports a login; `-p` keeps its arguments and its input line, waits, and prints a result line.

use std::path::{Path, PathBuf};
use zendo::agent::api::{Api, api_model};

/// The mock `claude` at `path`.
pub struct MockClaude {
    pub path: PathBuf,
}

/// One question that the mock got: its arguments, and the user message of its input.
#[derive(Debug)]
pub struct Asked {
    pub args: String,
    pub message: serde_json::Value,
}

impl Asked {
    /// The text block of the message: the prompt.
    pub fn prompt(&self) -> String {
        let blocks = self.message["message"]["content"].as_array().cloned().unwrap_or_default();
        blocks.iter().filter_map(|b| b["text"].as_str()).collect()
    }
}

impl MockClaude {
    /// The light model at the mock, with the model of the short name `short`.
    pub fn api(&self, short: &str) -> Api {
        Api { path: self.path.clone(), model: api_model(short), effort: None, env: Default::default() }
    }

    /// The questions so far, in their order.
    pub fn asked(&self) -> Vec<Asked> {
        let read = |ext: &str| std::fs::read_to_string(self.path.with_extension(ext)).unwrap_or_default();
        let (args, input) = (read("args"), read("in"));
        args.lines().zip(input.lines()).map(|(a, i)| Asked { args: a.to_string(), message: serde_json::from_str(i).unwrap_or_default() }).collect()
    }
}

/// A mock `claude` in `dir` that answers `text` to each question.
pub fn mock_claude(dir: &Path, text: &str) -> MockClaude {
    mock_claude_with(dir, Ok(text), 0.0)
}

/// A mock `claude` in `dir` that answers `answer` after `secs` seconds: `Ok` a result, `Err` an
/// error result with that text (NAMES-19, REKEY-13).
pub fn mock_claude_with(dir: &Path, answer: Result<&str, &str>, secs: f32) -> MockClaude {
    std::fs::create_dir_all(dir).unwrap();
    let path = dir.join("mock-claude");
    for ext in ["args", "in"] {
        let _ = std::fs::remove_file(path.with_extension(ext));
    }
    let (text, error) = match answer {
        Ok(t) => (t, false),
        Err(e) => (e, true),
    };
    let line = serde_json::json!({"type": "result", "subtype": "success", "is_error": error, "result": text});
    let script = format!(
        "#!/bin/sh\n[ \"$1 $2\" = \"auth status\" ] && {{ echo '{{\"loggedIn\": true}}'; exit 0; }}\n\
         printf '%s\\n' \"$*\" >> \"$0.args\"\nread -r line\nprintf '%s\\n' \"$line\" >> \"$0.in\"\nsleep {secs}\n\
         cat <<'ZENDO_EOF'\n{line}\nZENDO_EOF\n"
    );
    std::fs::write(&path, script).unwrap();
    let mut p = std::fs::metadata(&path).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut p, 0o755);
    std::fs::set_permissions(&path, p).unwrap();
    MockClaude { path }
}
