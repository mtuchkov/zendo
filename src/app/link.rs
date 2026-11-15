//! The connection to the agent: its handle and events, its name and what it can do, and the
//! questions for the names of sessions and images (ADR 0008). The link does not see the app.
use crate::agent::work::Asked;
use crate::agent::{AgentHandle, Cmd, Event};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Default)]
pub struct AgentLink {
    handle: Option<AgentHandle>,
    events: Option<UnboundedReceiver<Event>>,
    /// The name of the agent, from the time it is ready.
    name: Option<String>,
    /// Why the agent went down.
    down: Option<String>,
    /// The agent takes images in a prompt (from its ACP capabilities).
    images: bool,
    /// The model that names the agent sessions; none: the default model of the agent (NAMES-12).
    name_model: Option<String>,
    /// For each session ID: the last question for its name (NAMES-10, NAMES-14).
    asked: HashMap<String, Asked>,
    /// The pasted images that the name model names: by session key, the file and the answer so far.
    image_names: HashMap<String, (PathBuf, String)>,
}

impl AgentLink {
    /// A link that names sessions with `name_model`.
    pub fn with_name_model(name_model: Option<String>) -> Self {
        Self { name_model, ..Self::default() }
    }

    /// The agent process runs.
    pub fn running(&self) -> bool {
        self.handle.is_some()
    }

    pub fn handle(&self) -> Option<&AgentHandle> {
        self.handle.as_ref()
    }

    /// Send a command to the agent, when it runs.
    pub fn send(&self, cmd: Cmd) {
        if let Some(a) = &self.handle {
            a.send(cmd);
        }
    }

    /// A new agent process: its handle and its events. What made the last one go down is past.
    pub fn connect(&mut self, handle: AgentHandle, events: UnboundedReceiver<Event>) {
        (self.handle, self.events, self.down) = (Some(handle), Some(events), None);
    }

    /// The agent process is gone, for the reason `why`.
    pub fn disconnect(&mut self, why: String) {
        (self.handle, self.events, self.down) = (None, None, Some(why));
    }

    pub fn events_mut(&mut self) -> Option<&mut UnboundedReceiver<Event>> {
        self.events.as_mut()
    }

    /// The agent is ready: its name.
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    /// The name of the agent, when it is ready.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn ready(&self) -> bool {
        self.name.is_some()
    }

    pub fn down(&self) -> Option<&str> {
        self.down.as_deref()
    }

    pub fn images(&self) -> bool {
        self.images
    }

    pub fn set_images(&mut self, images: bool) {
        self.images = images;
    }

    pub fn name_model(&self) -> Option<String> {
        self.name_model.clone()
    }

    pub fn set_name_model(&mut self, model: Option<String>) {
        self.name_model = model;
    }

    /// The last question for the name of the Claude session `id`.
    pub fn asked(&self, id: &str) -> Option<&Asked> {
        self.asked.get(id)
    }

    /// zendo asks for the name of session `id` now, at a conversation of `size`.
    pub fn ask_name(&mut self, id: &str, size: usize) {
        let keeps = self.asked.get(id).map_or(0, |a| a.keeps);
        self.asked.insert(id.to_string(), Asked { size, at: Instant::now(), keeps });
    }

    /// The answer kept the name of session `id`: the thresholds double, up to `most` times.
    pub fn name_kept(&mut self, id: &str, most: u32) {
        if let Some(a) = self.asked.get_mut(id) {
            a.keeps = (a.keeps + 1).min(most);
        }
    }

    /// The name model names the image at `path` in the throwaway session `key`.
    pub fn name_image(&mut self, key: String, path: PathBuf) {
        self.image_names.insert(key, (path, String::new()));
    }

    /// A part of the answer for the image of session `key`.
    pub fn image_text(&mut self, key: &str, text: &str) {
        if let Some((_, buf)) = self.image_names.get_mut(key) {
            buf.push_str(text);
        }
    }

    /// The naming turn of `key` ended: the image and the answer.
    pub fn take_image(&mut self, key: &str) -> Option<(PathBuf, String)> {
        self.image_names.remove(key)
    }

    /// No image waits for a name.
    pub fn no_image_waits(&self) -> bool {
        self.image_names.is_empty()
    }
}

#[cfg(test)]
#[path = "link_tests.rs"]
mod tests;
