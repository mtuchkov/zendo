use super::*;
use serde_json::{Value, json};

/// The answers of the server to `lines`, as JSON.
fn talk(session: usize, lines: &[Value]) -> Vec<Value> {
    let input: String = lines.iter().map(|l| format!("{l}\n")).collect();
    let mut out = Vec::new();
    serve(input.as_bytes(), &mut out, session).unwrap();
    String::from_utf8(out).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect()
}

fn call(id: u32, method: &str, params: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params })
}

// SESSIONS-21: the server says that zendo runs the session, and has the tool `session`.
#[test]
fn initialize_names_zendo_and_the_session_tool() {
    let r = talk(2, &[call(1, "initialize", json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "t", "version": "1" } }))]);
    assert_eq!(r[0]["id"], 1);
    assert_eq!(r[0]["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(r[0]["result"]["serverInfo"]["name"], "zendo");
    assert!(r[0]["result"]["capabilities"]["tools"].is_object());
    assert!(r[0]["result"]["instructions"].as_str().unwrap().contains("`session`"));
}

// SESSIONS-21: the instructions name the id of the zendo session and carry the rules of its
// worktree, so the agent knows them without a call of the tool.
#[test]
fn instructions_carry_the_session_id_and_the_worktree_rules() {
    let r = talk(2, &[call(1, "initialize", json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "t", "version": "1" } }))]);
    let text = r[0]["result"]["instructions"].as_str().unwrap();
    assert!(text.contains("zendo session 2"), "{text}");
    assert!(text.contains("`.worktrees/2`"), "{text}");
    assert!(text.contains("git worktree add .worktrees/2 -b"), "{text}");
    assert!(text.contains("`git stash`"), "{text}");
    assert!(text.contains("approves"), "{text}");
    assert!(!text.contains("<id>") && !text.contains("{{ID}}"), "{text}");
    // The instructions are in XML tags: the session with its id, its worktree, and the rules.
    assert!(text.starts_with("<zendo_session id=\"2\">") && text.trim_end().ends_with("</zendo_session>"), "{text}");
    assert!(text.contains("<worktree>.worktrees/2</worktree>"), "{text}");
    assert_eq!(text.matches("<rule>").count(), 5, "{text}");
    assert_eq!(text.matches("<rule>").count(), text.matches("</rule>").count(), "{text}");
    assert!(text.contains("<rules>") && text.contains("</rules>"), "{text}");
}

// SESSIONS-21: the tool `session` returns the id of the zendo session.
#[test]
fn the_session_tool_returns_the_zendo_session_id() {
    let r = talk(3, &[
        call(1, "tools/list", json!({})),
        call(2, "tools/call", json!({ "name": "session", "arguments": {} })),
    ]);
    assert_eq!(r[0]["result"]["tools"][0]["name"], "session");
    assert_eq!(r[0]["result"]["tools"][0]["inputSchema"]["type"], "object");
    let text = r[1]["result"]["content"][0]["text"].as_str().unwrap();
    let v: Value = serde_json::from_str(text).unwrap();
    assert_eq!(v["zendo_session_id"], 3);
}

// SESSIONS-21: a notification gets no answer; an unknown method and an unknown tool get an error.
#[test]
fn notifications_get_no_answer_and_unknown_requests_an_error() {
    let r = talk(1, &[
        json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
        call(7, "nope", json!({})),
        call(8, "tools/call", json!({ "name": "nope" })),
        call(9, "ping", json!({})),
    ]);
    assert_eq!(r.len(), 3, "{r:?}");
    assert_eq!(r[0]["id"], 7);
    assert_eq!(r[0]["error"]["code"], -32601);
    assert_eq!(r[1]["result"]["isError"], true);
    assert_eq!(r[2]["result"], json!({}));
}

// SESSIONS-21: the command line of the server, as Claude Code and ACP start it.
#[test]
fn the_config_starts_zendo_mcp_with_the_session() {
    let v: Value = serde_json::from_str(&config_json(std::path::Path::new("/bin/zendo"), 4)).unwrap();
    assert_eq!(v["mcpServers"]["zendo"]["command"], "/bin/zendo");
    assert_eq!(v["mcpServers"]["zendo"]["args"], json!(["mcp", "--session", "4"]));
}

// SESSIONS-21, JSON-RPC: a response from the client gets no answer; a line that is not JSON gets a
// parse error.
#[test]
fn responses_get_no_answer_and_bad_json_a_parse_error() {
    let input = "{\"jsonrpc\":\"2.0\",\"id\":5,\"result\":{}}\nnot json\n";
    let mut out = Vec::new();
    serve(input.as_bytes(), &mut out, 1).unwrap();
    let r: Vec<Value> = String::from_utf8(out).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert_eq!(r.len(), 1, "{r:?}");
    assert_eq!(r[0]["id"], Value::Null);
    assert_eq!(r[0]["error"]["code"], -32700);
}

// SESSIONS-21, MCP: initialize answers a version that the server knows, else its own.
#[test]
fn initialize_answers_a_known_protocol_version() {
    let init = |v: &str| talk(1, &[call(1, "initialize", json!({ "protocolVersion": v }))])[0]["result"]["protocolVersion"].clone();
    assert_eq!(init("2025-03-26"), "2025-03-26");
    assert_eq!(init("1999-01-01"), PROTOCOL);
}
