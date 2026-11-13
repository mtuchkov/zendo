//! The messages of a Claude Code session from its hooks (docs/zendo/cli-messages.md): the hooks of
//! "prompt sent" and "stop" append their input to a file of the session, one line each; zendo
//! takes the new whole lines on each tick.

use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

/// A message of Claude Code for the chat.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    /// A prompt that Claude Code got.
    Prompt(String),
    /// The final answer of a turn.
    Answer(String),
}

/// The file of the messages of a session, and how far zendo read it.
#[derive(Debug)]
pub struct Inbox {
    path: PathBuf,
    read: u64,
}

impl Inbox {
    pub fn new(path: PathBuf) -> Self {
        Inbox { path, read: 0 }
    }

    /// The messages of the whole lines that came since the last call. A line that a hook still
    /// writes waits for the next call.
    pub fn take(&mut self) -> Vec<Message> {
        let mut new = Vec::new();
        let read = std::fs::File::open(&self.path).and_then(|mut f| {
            f.seek(SeekFrom::Start(self.read))?;
            f.read_to_end(&mut new)
        });
        let Some(end) = read.ok().and_then(|_| new.iter().rposition(|b| *b == b'\n')) else { return Vec::new() };
        self.read += end as u64 + 1;
        String::from_utf8_lossy(&new[..end]).lines().filter_map(parse).collect()
    }
}

/// The message in the input of a hook; none for another event or an empty text (CLIMSG-6).
fn parse(line: &str) -> Option<Message> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    let text = |field: &str| v[field].as_str().filter(|s| !s.trim().is_empty()).map(str::to_string);
    match v["hook_event_name"].as_str()? {
        "UserPromptSubmit" => text("prompt").map(Message::Prompt),
        "Stop" => text("last_assistant_message").map(Message::Answer),
        _ => None,
    }
}

/// A prompt as the user typed it: without the marks of pasted content, and trimmed (CLIMSG-4).
pub fn plain(prompt: &str) -> String {
    let marks = |l: &&str| l.starts_with("<pasted_content") || l.starts_with("</pasted_content");
    prompt.lines().filter(|l| !marks(l)).collect::<Vec<_>>().join("\n").trim().to_string()
}

#[cfg(test)]
#[path = "messages_tests.rs"]
mod tests;
