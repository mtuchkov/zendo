//! A mock of the `claude` command line for the tests of the sessions (docs/zendo/sessions.md).
//!
//! It prints its arguments, then `tick N` each 100 ms. In raw mode it reads keys: `r` reports the
//! state "works", `w` reports "waits", `i` reports "idle", `q` exits, and `enter` prints `got: <the line>`. A state goes to the file in `$ZENDO_STATE`, as the
//! hooks of Claude Code do. On `enter`, the inputs of the "prompt sent" and "stop" hooks go to the
//! file in `$ZENDO_MESSAGES`: the line, and `answer: <the line>`.

use ratatui::crossterm::terminal;
use std::io::{Read, Write};
use std::time::Duration;

fn report(state: &str) {
    if let Ok(path) = std::env::var("ZENDO_STATE") {
        let _ = std::fs::write(path, state);
    }
}

/// The inputs of the "prompt sent" and "stop" hooks for a prompt, one line each, as the hooks of
/// zendo append them.
fn hooks(prompt: &str) {
    let Ok(path) = std::env::var("ZENDO_MESSAGES") else { return };
    let sent = serde_json::json!({ "hook_event_name": "UserPromptSubmit", "prompt": prompt });
    let stop = serde_json::json!({ "hook_event_name": "Stop", "last_assistant_message": format!("answer: {prompt}") });
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{sent}\n{stop}");
    }
}

fn main() {
    let _ = terminal::enable_raw_mode();
    let args: Vec<String> = std::env::args().skip(1).collect();
    print!("mock cli: {}\r\n", args.join(" "));
    let _ = std::io::stdout().flush();
    std::thread::spawn(|| {
        let mut n = 0u64;
        loop {
            n += 1;
            print!("tick {n}\r\n");
            let _ = std::io::stdout().flush();
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
            b'\r' | b'\n' => {
                print!("got: {}\r\n", String::from_utf8_lossy(&line));
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
