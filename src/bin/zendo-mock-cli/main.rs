//! A mock of the `claude` command line for the tests of the sessions (docs/zendo/sessions.md).
//!
//! It prints its arguments, then `tick N` each 100 ms. In raw mode it reads keys: `r` reports the
//! state "works", `w` reports "waits", `i` reports "idle", `q` exits, and `enter` prints `got: <the line>`;
//! the line `hush` stops the ticks, and the line `background <seconds>` starts `sleep <seconds>` in a process group of its own. A state goes to the file in `$ZENDO_STATE`, as the
//! hooks of Claude Code do. On `enter`, the inputs of the "prompt sent" and "stop" hooks go to the
//! file in `$ZENDO_MESSAGES`: the line, and `answer: <the line>`. At the start the input of the
//! "session start" hook of a resume goes there with the session ID after `--resume`, and on
//! `ctrl-l` the one of a `/clear` with a new session ID. Each input names the transcript of the
//! session, a file beside `$ZENDO_MESSAGES`; a prompt appends to it, as Claude Code does: the text
//! `step: <the line>`, a call of the tool `Bash`, and the text `answer: <the line>`.

use ratatui::crossterm::terminal;
use std::io::{Read, Write};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// The session ID of the conversation; `/clear` changes it.
static SESSION: Mutex<String> = Mutex::new(String::new());
/// The line `hush` stops the ticks: the program is quiet, as a tool at its prompt.
static QUIET: AtomicBool = AtomicBool::new(false);

/// The transcript of the conversation: a file beside the file of the messages.
fn transcript() -> String {
    format!("{}.{}.jsonl", std::env::var("ZENDO_MESSAGES").unwrap_or_default(), SESSION.lock().unwrap())
}

fn append(path: &str, lines: &[serde_json::Value]) {
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let text: String = lines.iter().map(|l| format!("{l}\n")).collect();
        let _ = f.write_all(text.as_bytes());
    }
}

/// One entry of the transcript with one block of an answer of the assistant.
fn entry(block: serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "type": "assistant", "isSidechain": false, "message": { "role": "assistant", "content": [block] } })
}

fn report(state: &str) {
    if let Ok(path) = std::env::var("ZENDO_STATE") {
        let _ = std::fs::write(path, state);
    }
}

/// The input of the "session start" hook: `source` is "resume" or "clear".
fn session_start(id: &str, source: &str) {
    let Ok(path) = std::env::var("ZENDO_MESSAGES") else { return };
    *SESSION.lock().unwrap() = id.to_string();
    append(&path, &[serde_json::json!({ "hook_event_name": "SessionStart", "session_id": id, "source": source, "transcript_path": transcript() })]);
}

/// A turn for a prompt: the inputs of the "prompt sent" and "stop" hooks, one line each, as the
/// hooks of zendo append them, and between them the steps of the turn in the transcript.
fn hooks(prompt: &str) {
    let Ok(path) = std::env::var("ZENDO_MESSAGES") else { return };
    let t = transcript();
    append(&path, &[serde_json::json!({ "hook_event_name": "UserPromptSubmit", "prompt": prompt, "transcript_path": t })]);
    let run = serde_json::json!({ "type": "tool_use", "id": format!("tool-{prompt}"), "name": "Bash", "input": { "command": "true", "description": format!("run {prompt}") } });
    let text = |s: String| entry(serde_json::json!({ "type": "text", "text": s }));
    append(&t, &[text(format!("step: {prompt}")), entry(run), text(format!("answer: {prompt}"))]);
    append(&path, &[serde_json::json!({ "hook_event_name": "Stop", "last_assistant_message": format!("answer: {prompt}"), "transcript_path": t })]);
}

/// `background 6201`: start `sleep 6201` in a shell, in a process group of its own, as Claude Code does with a
/// command in the background (BG-1).
fn background(line: &str) {
    use std::os::unix::process::CommandExt;
    if let Some(s) = line.strip_prefix("background ").filter(|s| s.trim().parse::<u32>().is_ok()) {
        let _ = std::process::Command::new("sh").args(["-c", &format!("sleep {}; :", s.trim())]).process_group(0).stdout(std::process::Stdio::null()).spawn();
    }
}

fn main() {
    let _ = terminal::enable_raw_mode();
    let args: Vec<String> = std::env::args().skip(1).collect();
    print!("mock cli: {}\r\n", args.join(" "));
    if let Some(id) = args.iter().skip_while(|a| *a != "--resume").nth(1) {
        session_start(id, "resume");
    }
    let _ = std::io::stdout().flush();
    std::thread::spawn(|| {
        let mut n = 0u64;
        loop {
            if !QUIET.load(Ordering::Relaxed) {
                n += 1;
                print!("tick {n}\r\n");
                let _ = std::io::stdout().flush();
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    });
    let mut stdin = std::io::stdin();
    let mut b = [0u8; 1];
    let mut line = Vec::new();
    while stdin.read(&mut b).is_ok_and(|n| n == 1) {
        match b[0] {
            b'r' => report("works"),
            b'w' => report("waits"),
            b'i' => report("idle"),
            b'q' => break,
            0x0c => {
                session_start(&format!("cleared-{}", std::process::id()), "clear");
                continue;
            }
            b'\r' | b'\n' => {
                print!("got: {}\r\n", String::from_utf8_lossy(&line));
                background(&String::from_utf8_lossy(&line));
                QUIET.fetch_or(line == b"hush", Ordering::Relaxed);
                let _ = std::io::stdout().flush();
                hooks(&String::from_utf8_lossy(&line));
                line.clear();
                continue;
            }
            _ => {}
        }
        line.push(b[0]);
    }
    let _ = terminal::disable_raw_mode();
}
