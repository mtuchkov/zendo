//! The messages of a Claude Code session (docs/zendo/cli-messages.md): the hooks of "prompt sent",
//! "stop" and "session start" append their input to a file of the session, one line each; zendo
//! takes the new whole lines on each tick. Each input names the transcript of the conversation,
//! where zendo reads the steps of the turn.

use super::transcript::Tail;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// A message of Claude Code for the chat.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    /// A prompt that Claude Code got.
    Prompt(String),
    /// The final answer of a turn, from the "stop" hook.
    Answer(String),
    /// Text of Claude in a turn, from the transcript (CLIMSG-8).
    Text(String),
    /// A tool call of a turn, from the transcript: its ID, one line about it, its kind (CLIMSG-9).
    Tool { id: String, title: String, kind: String },
    /// A Claude session started: its session ID, and whether `/clear` started a new conversation
    /// (NAMES-4, NAMES-7).
    Session { id: String, clear: bool },
}

/// The file of the messages of a session, and how far zendo read it.
#[derive(Debug)]
pub struct Inbox {
    path: PathBuf,
    read: u64,
    /// The transcript that the last hook named.
    transcript: Option<Tail>,
    /// The texts of Claude that the chat got since the last prompt, so that none shows twice.
    said: Vec<String>,
}

impl Inbox {
    pub fn new(path: PathBuf) -> Self {
        Inbox { path, read: 0, transcript: None, said: Vec::new() }
    }

    /// The messages since the last call: the prompts and the new conversations of the hooks, then
    /// the steps of the transcript, then the answers of the hooks. A line that a hook or Claude
    /// Code still writes waits for the next call.
    pub fn take(&mut self) -> Vec<Message> {
        let text = new_lines(&self.path, &mut self.read);
        let lines: Vec<serde_json::Value> = text.lines().filter_map(|l| serde_json::from_str(l).ok()).collect();
        let mut steps = self.transcript.as_mut().map(Tail::take).unwrap_or_default();
        lines.iter().for_each(|v| steps.extend(self.follow(v)));
        let hooks: Vec<Message> = lines.iter().filter_map(parse).collect();
        let split = hooks.iter().rposition(|m| matches!(m, Message::Prompt(_) | Message::Session { .. })).map_or(0, |i| i + 1);
        let (first, answers) = hooks.split_at(split);
        let all = first.iter().cloned().chain(steps).chain(answers.iter().cloned());
        all.filter(|m| self.is_new(m)).collect()
    }

    /// The hook input `v` names another transcript: follow it (CLIMSG-10), and take what it has. A
    /// new conversation is read from its start.
    fn follow(&mut self, v: &serde_json::Value) -> Vec<Message> {
        let Some(path) = v["transcript_path"].as_str().map(Path::new) else { return Vec::new() };
        if self.transcript.as_ref().is_some_and(|t| t.path() == path) {
            return Vec::new();
        }
        let new_conversation = v["hook_event_name"] == "SessionStart" && matches!(v["source"].as_str(), Some("clear" | "startup"));
        let tail = self.transcript.insert(Tail::new(path.to_path_buf(), new_conversation));
        tail.take()
    }

    /// A text of Claude shows once in a turn: the transcript and the "stop" hook both have the
    /// final answer (CLIMSG-1).
    fn is_new(&mut self, m: &Message) -> bool {
        match m {
            Message::Prompt(_) => self.said.clear(),
            Message::Answer(s) | Message::Text(s) if self.said.iter().any(|t| t == s.trim()) => return false,
            Message::Answer(s) | Message::Text(s) => self.said.push(s.trim().to_string()),
            _ => {}
        }
        true
    }
}

/// The whole lines of the file at `path` after the byte `read`, which moves on. A line without
/// its end waits.
pub(super) fn new_lines(path: &Path, read: &mut u64) -> String {
    let mut new = Vec::new();
    let got = std::fs::File::open(path).and_then(|mut f| {
        f.seek(SeekFrom::Start(*read))?;
        f.read_to_end(&mut new)
    });
    let Some(end) = got.ok().and_then(|_| new.iter().rposition(|b| *b == b'\n')) else { return String::new() };
    *read += end as u64 + 1;
    String::from_utf8_lossy(&new[..end]).into_owned()
}

/// The message in the input of a hook; none for another event or an empty text (CLIMSG-6).
fn parse(v: &serde_json::Value) -> Option<Message> {
    let text = |field: &str| v[field].as_str().filter(|s| !s.trim().is_empty()).map(str::to_string);
    match v["hook_event_name"].as_str()? {
        "UserPromptSubmit" => text("prompt").map(Message::Prompt),
        "Stop" => text("last_assistant_message").map(Message::Answer),
        "SessionStart" => text("session_id").map(|id| Message::Session { id, clear: v["source"] == "clear" }),
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
