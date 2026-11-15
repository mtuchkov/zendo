//! The MCP server of a zendo session (SESSIONS-21): `zendo mcp --session N`.
//!
//! zendo starts Claude Code, in a terminal and in the chat, with this server. It speaks JSON-RPC,
//! one message on each line of stdin and stdout, and has one tool, `session`, that returns the id
//! of the zendo session. Its instructions name the id and the rules of the worktree of the session.

use crate::app::{SESSIONS, session_key};
use agent_client_protocol::v1::schema::{McpServer, McpServerStdio};
use serde_json::{Value, json};
use std::io::{BufRead, Write};
use std::path::Path;

/// The protocol version when the client names none, or one that the server does not know.
const PROTOCOL: &str = "2025-06-18";
const KNOWN: [&str; 3] = ["2024-11-05", "2025-03-26", PROTOCOL];

/// The rules of a zendo session, in XML tags; `{{ID}}` stands for its id.
const INSTRUCTIONS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/mcp-instructions.md"));

/// The instructions of the server of zendo session `session`.
fn instructions(session: usize) -> String {
    INSTRUCTIONS.replace("{{ID}}", &session.to_string())
}

/// The arguments of `zendo` that start the server of `session`.
pub fn args(session: usize) -> Vec<String> {
    vec!["mcp".into(), "--session".into(), session.to_string()]
}

/// The config of Claude Code (`--mcp-config`) with the server of `session`.
pub fn config_json(exe: &Path, session: usize) -> String {
    json!({ "mcpServers": { "zendo": { "command": exe, "args": args(session) } } }).to_string()
}

/// The MCP servers of the chat thread `key`: the server of its zendo session, or none for a thread
/// that is not an agent session.
pub fn acp_servers(key: &str) -> Vec<McpServer> {
    let session = (0..SESSIONS).find(|n| session_key(*n) == key);
    let (Some(n), Ok(exe)) = (session, std::env::current_exe()) else { return Vec::new() };
    vec![McpServer::Stdio(McpServerStdio::new("zendo", exe).args(args(n + 1)))]
}

/// Answer each request on `input` on `output`, until `input` ends.
pub fn serve(input: impl BufRead, mut output: impl Write, session: usize) -> std::io::Result<()> {
    for line in input.lines() {
        let reply = match serde_json::from_str::<Value>(&line?) {
            Ok(msg) => answer(&msg, session),
            Err(_) => Some(json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": "parse error" } })),
        };
        if let Some(reply) = reply {
            writeln!(output, "{reply}")?;
            output.flush()?;
        }
    }
    Ok(())
}

/// The answer to one message; none for a notification or a response of the client.
fn answer(msg: &Value, session: usize) -> Option<Value> {
    let id = msg.get("id")?.clone();
    let params = &msg["params"];
    let result = match msg.get("method")?.as_str().unwrap_or_default() {
        "initialize" => json!({
            "protocolVersion": params["protocolVersion"].as_str().filter(|v| KNOWN.contains(v)).unwrap_or(PROTOCOL),
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "zendo", "version": env!("CARGO_PKG_VERSION") },
            "instructions": instructions(session),
        }),
        "ping" => json!({}),
        "tools/list" => json!({ "tools": [{
            "name": "session",
            "description": "The id of the zendo session that runs this Claude Code session.",
            "inputSchema": { "type": "object", "properties": {} },
        }] }),
        "tools/call" => call(params["name"].as_str().unwrap_or_default(), session),
        _ => return Some(json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": "method not found" } })),
    };
    Some(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

/// The result of the tool `name`.
fn call(name: &str, session: usize) -> Value {
    let (text, error) = match name {
        "session" => (json!({ "zendo_session_id": session }).to_string(), false),
        other => (format!("unknown tool: {other}"), true),
    };
    json!({ "content": [{ "type": "text", "text": text }], "isError": error })
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
