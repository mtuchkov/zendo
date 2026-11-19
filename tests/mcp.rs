//! The MCP server of a zendo session (SESSIONS-21, docs/zendo/sessions.md): `zendo mcp --session N`,
//! and the terminal of a Claude Code session that zendo starts with it.

mod common;
use common::*;
use serde_json::{Value, json};
use std::io::Write;
use std::process::{Command as Process, Stdio};
use std::time::{Duration, Instant};
use zendo::app::{App, Focus, session_key};
use zendo::session::Command;

/// The answers of the server that `program args` starts, to an initialize and a call of `session`.
fn ask(program: &str, args: &[String]) -> Vec<Value> {
    let mut child = Process::new(program).args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let msgs = [
        json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "t", "version": "1" } } }),
        json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
        json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": { "name": "session", "arguments": {} } }),
    ];
    let mut stdin = child.stdin.take().unwrap();
    for m in msgs {
        writeln!(stdin, "{m}").unwrap();
    }
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect()
}

fn session_id(answers: &[Value]) -> u64 {
    let text = answers[1]["result"]["content"][0]["text"].as_str().unwrap();
    serde_json::from_str::<Value>(text).unwrap()["zendo_session_id"].as_u64().unwrap()
}

// SESSIONS-21: `zendo mcp --session 2` answers as the server of zendo session 2.
#[test]
fn zendo_mcp_answers_with_its_session() {
    let r = ask(env!("CARGO_BIN_EXE_zendo"), &["mcp".into(), "--session".into(), "2".into()]);
    assert_eq!(r[0]["result"]["serverInfo"]["name"], "zendo");
    assert_eq!(session_id(&r), 2);
}

// SESSIONS-21: zendo starts Claude Code of session 2 with `--mcp-config`; the server in that config
// answers with session 2.
#[tokio::test(flavor = "multi_thread")]
async fn a_claude_code_session_gets_the_server_of_its_session() {
    let root = repo("mcp-cli", &[("docs/notes/2026-11-13-a.md", "# A\n")]);
    let mut a = app(&root);
    a.cli.command = Some(Command { program: env!("CARGO_BIN_EXE_zendo-mock-cli").into(), args: vec![] });
    a.thread(&session_key(1)).session = Some("s2".into());
    a.open_panel();
    a.switch_session(1);
    a.focus = Focus::Claude;
    ctrl(&mut a, 'o');
    let args = wait_args(&mut a, 1);
    let mut after = args.split_whitespace().skip_while(|w| *w != "--mcp-config").skip(1);
    let file = after.next().unwrap_or_else(|| panic!("no --mcp-config: {args}"));
    // `--mcp-config` takes several files: an option must end the list, not a prompt.
    assert!(after.next().is_some_and(|w| w.starts_with("--")), "an option follows the config: {args}");
    let cfg: Value = serde_json::from_str(&std::fs::read_to_string(file).unwrap()).unwrap();
    let server = &cfg["mcpServers"]["zendo"];
    let server_args: Vec<String> = serde_json::from_value(server["args"].clone()).unwrap();
    assert_eq!(session_id(&ask(env!("CARGO_BIN_EXE_zendo"), &server_args)), 2);
    assert!(std::path::Path::new(server["command"].as_str().unwrap()).is_absolute());
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

/// The arguments that the mock command line of session `n` printed.
fn wait_args(a: &mut App, n: usize) -> String {
    let end = Instant::now() + Duration::from_secs(10);
    loop {
        let text: String = a.cli.text(n).lines().collect();
        if let Some((_, rest)) = text.split_once("mock cli:") {
            return rest.split("tick ").next().unwrap_or(rest).to_string();
        }
        assert!(Instant::now() < end, "timeout");
        a.tick();
        std::thread::sleep(Duration::from_millis(20));
    }
}

// SESSIONS-21: the chat of agent session 2 gets the server of zendo session 2; a thread that is not
// an agent session gets none.
#[test]
fn the_chat_gets_the_server_of_its_session() {
    let servers = zendo::mcp::acp_servers(&session_key(2));
    let v = serde_json::to_value(&servers).unwrap();
    assert_eq!(v[0]["name"], "zendo");
    assert_eq!(v[0]["args"], json!(["mcp", "--session", "2"]));
    assert!(zendo::mcp::acp_servers("docs/notes/2026-11-13-a.md").is_empty());
}

// SESSIONS-21: the chat of agent session 1 opens its ACP session with the server of zendo session 1;
// the mock agent writes the MCP servers that it got to `.zendo/mock-mcp.json`.
#[tokio::test(flavor = "multi_thread")]
async fn the_chat_opens_its_session_with_the_server() {
    use ratatui::crossterm::event::KeyCode;
    let root = repo("mcp-chat", &[("README.txt", "no notes\n")]);
    let mut a = app(&root);
    a.switch_session(1);
    a.focus = Focus::Claude;
    typ(&mut a, "hello");
    key(&mut a, KeyCode::Enter);
    until(&mut a, |a| a.threads.get(&session_key(1)).is_some_and(|t| t.session.is_some())).await;
    let got: Value = serde_json::from_str(&std::fs::read_to_string(root.join(".zendo/mock-mcp.json")).unwrap()).unwrap();
    assert_eq!(got[0]["name"], "zendo");
    assert_eq!(got[0]["args"], json!(["mcp", "--session", "1"]));
    let _ = std::fs::remove_dir_all(&root);
}
