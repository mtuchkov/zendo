//! The steps of a turn from the transcript of Claude Code (docs/zendo/cli-messages.md): the text
//! between the tool calls, and the tool calls. The hooks name the file; zendo reads its new whole
//! lines. The format of the file is not a public interface of Claude Code: a line that zendo does
//! not understand gives no message.

use super::messages::{Message, new_lines};
use std::path::{Path, PathBuf};

/// The transcript of a conversation, and how far zendo read it.
#[derive(Debug)]
pub struct Tail {
    path: PathBuf,
    read: u64,
}

impl Tail {
    /// The transcript at `path`: from its start for a new conversation, else from its end, so that
    /// the history of a resumed session does not show again.
    pub fn new(path: PathBuf, from_start: bool) -> Self {
        let read = if from_start { 0 } else { std::fs::metadata(&path).map_or(0, |m| m.len()) };
        Tail { path, read }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The steps in the whole lines that came since the last call.
    pub fn take(&mut self) -> Vec<Message> {
        new_lines(&self.path, &mut self.read).lines().flat_map(steps).collect()
    }
}

/// The steps in one line of the transcript: the text blocks and the tool calls of an answer of the
/// assistant. The thoughts, the tool results and the work of a subagent give none (CLIMSG-8, CLIMSG-9).
fn steps(line: &str) -> Vec<Message> {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else { return Vec::new() };
    if v["type"] != "assistant" || v["isSidechain"] == true {
        return Vec::new();
    }
    v["message"]["content"].as_array().into_iter().flatten().filter_map(step).collect()
}

fn step(block: &serde_json::Value) -> Option<Message> {
    match block["type"].as_str()? {
        "text" => block["text"].as_str().map(str::trim).filter(|s| !s.is_empty()).map(|s| Message::Text(s.to_string())),
        "tool_use" => {
            let name = block["name"].as_str()?;
            let title = detail(&block["input"]).map_or(name.to_string(), |d| format!("{name}: {d}"));
            Some(Message::Tool { id: block["id"].as_str().unwrap_or_default().to_string(), title, kind: kind(name).to_string() })
        }
        _ => None,
    }
}

/// What a tool call is about, in one line: its description, its file, or what it looks for.
fn detail(input: &serde_json::Value) -> Option<String> {
    let field = ["description", "file_path", "pattern", "url", "query", "command"].into_iter().find_map(|f| input[f].as_str())?;
    let line = field.lines().next().unwrap_or_default().trim();
    (!line.is_empty()).then(|| line.chars().take(100).collect())
}

/// The kind of a tool of Claude Code, as the chat names the kinds of the tools of the agent.
fn kind(name: &str) -> &'static str {
    match name {
        "Read" | "NotebookRead" => "read",
        "Edit" | "Write" | "MultiEdit" | "NotebookEdit" => "edit",
        "Grep" | "Glob" => "search",
        "WebFetch" | "WebSearch" => "fetch",
        "Bash" => "execute",
        _ => "other",
    }
}
