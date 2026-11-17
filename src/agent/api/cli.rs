//! The `claude` CLI as the way to Claude for a user who signed in with a subscription, as Anthropic
//! allows for local use: `claude -p` with no tools, no settings, no MCP servers and a system prompt
//! of its own, so a question costs little more than one request (NAMES-21, ADR 0010).
use super::*;
use std::io::{Read, Write};
use std::process::Stdio;
use std::time::{Duration, Instant};

/// The system prompt of a question in place of the one of Claude Code.
const SYSTEM: &str = "You name things. Answer with the few words that the user asks for, and nothing else.";
/// The longest time of one question; the CLI starts first.
const LIMIT: Duration = Duration::from_secs(60);

/// The arguments of `claude` for one question to `model`: the message comes on the standard input,
/// the answer in JSON lines, and nothing of the settings, hooks, tools or MCP servers of the user
/// takes part.
pub fn args(model: &str) -> Vec<String> {
    let a = ["-p", "--input-format", "stream-json", "--output-format", "stream-json", "--verbose", "--model", model, "--system-prompt", SYSTEM];
    let b = ["--tools", "", "--strict-mcp-config", "--setting-sources", "", "--disable-slash-commands", "--no-session-persistence"];
    a.iter().chain(b.iter()).map(|s| s.to_string()).collect()
}

/// The line of the standard input: one user message with the content blocks `blocks`.
pub fn input(blocks: &Value) -> String {
    format!("{}\n", json!({"type": "user", "message": {"role": "user", "content": blocks}}))
}

/// The text of the result line of the output, or its error.
pub fn result(out: &str) -> Result<String, String> {
    let r = out.lines().filter_map(|l| serde_json::from_str::<Value>(l).ok()).find(|v| v["type"] == "result").ok_or("claude: no result")?;
    let text = r["result"].as_str().unwrap_or_default().to_string();
    match r["is_error"].as_bool().unwrap_or(false) || r["subtype"] != "success" {
        true => Err(format!("claude: {}", if text.is_empty() { r["subtype"].as_str().unwrap_or("error") } else { &text })),
        false => Ok(text),
    }
}

/// True when `claude auth status` at `path` reports a login (NAMES-21).
pub fn logged_in(path: &Path) -> bool {
    let out = Command::new(path).args(["auth", "status"]).stdin(Stdio::null()).output();
    out.ok().and_then(|o| serde_json::from_slice::<Value>(&o.stdout).ok()).is_some_and(|v| v["loggedIn"] == true)
}

/// One question through `claude -p` at `path`, in the temp folder, so that no `CLAUDE.md` of a
/// project takes part.
pub fn ask(path: &Path, api: &Api, prompt: &str, image: Option<&Image>) -> Result<String, String> {
    let blocks = Api { vendor: Vendor::Anthropic, ..api.clone() }.body(prompt, image)["messages"][0]["content"].clone();
    let spawn = Command::new(path).args(args(&api.model)).current_dir(std::env::temp_dir()).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn();
    let mut child = spawn.map_err(|e| format!("claude: {e}"))?;
    let sent = child.stdin.take().map(|mut i| i.write_all(input(&blocks).as_bytes()));
    let mut out = child.stdout.take().ok_or("claude: no output")?;
    let reader = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = out.read_to_string(&mut s);
        s
    });
    let end = Instant::now() + LIMIT;
    while child.try_wait().map_err(|e| e.to_string())?.is_none() {
        if Instant::now() > end {
            let _ = child.kill();
            return Err("claude: no answer in 60 s".into());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    sent.transpose().map_err(|e| format!("claude: {e}"))?;
    result(&reader.join().unwrap_or_default())
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
