//! The tools of the board as the MCP server lists them (BOARD-1).

use serde_json::{Value, json};

fn tool(name: &str, description: &str, props: Value, required: &[&str]) -> Value {
    json!({ "name": name, "description": description, "inputSchema": { "type": "object", "properties": props, "required": required } })
}

fn s(description: &str) -> Value {
    json!({ "type": "string", "description": description })
}

/// The board tools, after the tool `session`.
pub fn tools() -> Vec<Value> {
    let to = s("The channel: `session:<id>` (the inbox of a session), `all`, or the name of a channel.");
    let reply_to = s("Optional. The channel of the responses. A channel of yours here takes the recipients into it.");
    let text = s("The text. Self-contained: the other session has none of your context.");
    let in_reply_to = s("The id of the message that you respond to.");
    vec![
        tool("sessions", "The zendo sessions of this repository: id, inbox, name, state, and the branch of each worktree.", json!({}), &[]),
        tool("ask", "Emergency only: ask another session a question about cross-session state.", json!({ "to": to, "text": text, "reply_to": reply_to }), &["to", "text"]),
        tool("assign", "Emergency only: give another session a task, for work that belongs to it. Without `to`, the user chooses the session.", json!({ "to": s("Optional. `session:<id>`: a task goes to one session."), "text": text, "reply_to": reply_to }), &["text"]),
        tool("answer", "Answer a question that you got, once.", json!({ "in_reply_to": in_reply_to, "text": text }), &["in_reply_to", "text"]),
        tool("status", "The status of a task that you got: `accepted` or `rejected`, then `done` or `failed`, each once.", json!({ "in_reply_to": in_reply_to, "status": json!({ "type": "string", "enum": ["accepted", "rejected", "done", "failed"] }), "text": s("Optional.") }), &["in_reply_to", "status"]),
        tool("invite", "Add a session to a channel of yours.", json!({ "channel": s("The name of your channel."), "session": json!({ "type": "integer" }) }), &["channel", "session"]),
        tool("leave", "Leave a channel that you belong to.", json!({ "channel": s("The channel.") }), &["channel"]),
        tool("post", "Tell all sessions of the machine that you use a shared resource, such as `port:5220` or `docker:foo`. No response comes.", json!({ "resource": s("The resource, in free text."), "text": s("What you do with it."), "pid": json!({ "type": "integer", "description": "Optional. The process that holds the resource: the lease of the post ends with it." }) }), &["resource", "text"]),
        tool("release", "You no longer use the resource of your post.", json!({ "resource": s("The resource of your post.") }), &["resource"]),
        tool("posts", "The posts on the board with their leases, and the history.", json!({}), &[]),
        tool("tombstone", "Clear a post whose lease is expired: its owner is dead.", json!({ "post": s("The id of the post.") }), &["post"]),
    ]
}
