//! The MCP server of a zendo session (SESSIONS-21): `zendo mcp --session N`.
//!
//! zendo starts Claude Code, in a terminal and in the chat, with this server. It speaks JSON-RPC,
//! one message on each line of stdin and stdout, and has one tool, `session`, that returns the id
//! of the zendo session. Its instructions name the id and the rules of the worktree of the session.

use crate::app::{SESSIONS, session_key};
use crate::board::tools::Ctx;
use agent_client_protocol::v1::schema::{McpServer, McpServerStdio};
use serde_json::{Value, json};
use std::io::{BufRead, Write};
use std::path::Path;

/// The protocol version when the client names none, or one that the server does not know.
const PROTOCOL: &str = "2025-06-18";
const KNOWN: [&str; 3] = ["2024-11-05", "2025-03-26", PROTOCOL];

/// The rules of a zendo session, in XML tags; `{{ID}}` stands for its id.
const INSTRUCTIONS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/mcp-instructions.md"));

/// The rules of zendo session 0, which works in the main checkout (SESSIONS-23).
const MAIN_INSTRUCTIONS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/mcp-instructions-main.md"));

/// The rules of the message board; `{{EXE}}` stands for the path of zendo (BOARD-12, BOARD-25).
const BOARD_INSTRUCTIONS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/mcp-board.md"));

/// The instructions of the server of zendo session `session`, with the rules of the board when it
/// has one.
fn instructions(session: usize, ctx: Option<&Ctx>) -> String {
    let board = ctx.map_or(String::new(), |c| BOARD_INSTRUCTIONS.replace("{{EXE}}", &c.exe));
    let text = if session == 0 { MAIN_INSTRUCTIONS } else { INSTRUCTIONS };
    text.replace("{{BOARD}}", &board).replace("{{ID}}", &session.to_string())
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
    vec![McpServer::Stdio(McpServerStdio::new("zendo", exe).args(args(n)))]
}

/// The board of the server of `session` in the folder `cwd`, when the user has a home (ADR 0021).
pub fn board(session: usize, cwd: &Path) -> Option<Ctx> {
    let exe = std::env::current_exe().ok()?.display().to_string();
    let journal = crate::board::Journal::at(crate::board::default_dir()?);
    Some(Ctx { journal, repo: crate::board::repo_of(cwd), session, exe, choose_for: std::time::Duration::from_secs(600) })
}

/// Answer each request on `input` on `output`, until `input` ends.
pub fn serve(input: impl BufRead, output: impl Write, session: usize) -> std::io::Result<()> {
    let cwd = std::env::current_dir().unwrap_or_default();
    serve_with(input, output, session, board(session, &cwd).as_ref())
}

/// `serve` with the board `ctx`; without it, only the tool `session`.
pub fn serve_with(input: impl BufRead, mut output: impl Write, session: usize, ctx: Option<&Ctx>) -> std::io::Result<()> {
    for line in input.lines() {
        let reply = match serde_json::from_str::<Value>(&line?) {
            Ok(msg) => answer(&msg, session, ctx),
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
fn answer(msg: &Value, session: usize, ctx: Option<&Ctx>) -> Option<Value> {
    let id = msg.get("id")?.clone();
    let params = &msg["params"];
    let result = match msg.get("method")?.as_str().unwrap_or_default() {
        "initialize" => json!({
            "protocolVersion": params["protocolVersion"].as_str().filter(|v| KNOWN.contains(v)).unwrap_or(PROTOCOL),
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "zendo", "version": env!("CARGO_PKG_VERSION") },
            "instructions": instructions(session, ctx),
        }),
        "ping" => json!({}),
        "tools/list" => json!({ "tools": tools(ctx.is_some()) }),
        "tools/call" => call(params["name"].as_str().unwrap_or_default(), &params["arguments"], session, ctx),
        _ => return Some(json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": "method not found" } })),
    };
    Some(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

/// The tool `session`, and the tools of the board when it is there.
fn tools(board: bool) -> Vec<Value> {
    let mut v = vec![json!({
        "name": "session",
        "description": "The id of the zendo session that runs this Claude Code session.",
        "inputSchema": { "type": "object", "properties": {} },
    })];
    if board {
        v.extend(crate::board::schema::tools());
    }
    v
}

/// The result of the tool `name`.
fn call(name: &str, args: &Value, session: usize, ctx: Option<&Ctx>) -> Value {
    let board = ctx.and_then(|c| crate::board::tools::call(c, name, args));
    let (text, error) = match (name, board) {
        ("session", _) => (json!({ "zendo_session_id": session }).to_string(), false),
        (_, Some(Ok(v))) => (v.to_string(), false),
        (_, Some(Err(e))) => (e, true),
        (other, None) => (format!("unknown tool: {other}"), true),
    };
    json!({ "content": [{ "type": "text", "text": text }], "isError": error })
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
