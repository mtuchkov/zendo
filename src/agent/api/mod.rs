//! The light model for the questions that need a few words back: the names of the agent sessions,
//! of the pasted images, the titles of new notes and the key hints
//! (docs/zendo/session-names/session-names-api.md, NAMES-15, NAMES-16, NAMES-21, NAMES-22,
//! ADR 0010). zendo asks through `claude -p` with the login of the user. It reads no credential of
//! another program, uses no API key, and sends no request of its own to a model API (NAMES-22). Without `claude`,
//! zendo asks the agent in a throwaway session (NAMES-16).

mod ask;
mod cli;

pub use ask::{Answer, Asker, Question};

use crate::setup::config::Kind;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The `claude` CLI that answers, and the model and the effort of its questions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Api {
    /// The `claude` CLI, with the login of the user, also a subscription (NAMES-21).
    pub path: PathBuf,
    pub model: String,
    /// The effort of the model, as `low`: the key hints ask with low effort (REKEY-6).
    pub effort: Option<String>,
    /// The variables of the agents over the environment of zendo (ENV-1).
    pub env: std::collections::BTreeMap<String, String>,
}

/// An image for the question, in base64.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub media: &'static str,
    pub data: String,
}

impl Api {
    /// The light model of an agent of `kind` with the name model `model`: the `claude` CLI on the
    /// `PATH` for Claude, none for other agents (NAMES-15, NAMES-16). `which` finds a program.
    pub fn find(kind: Kind, model: Option<String>, which: &dyn Fn(&str) -> Option<PathBuf>) -> Option<Self> {
        if kind != Kind::Claude {
            return None;
        }
        Some(Self { path: which("claude")?, model: api_model(&model?), effort: None, env: Default::default() })
    }

    /// True when `claude auth status` reports a login (NAMES-21).
    pub fn usable(&self) -> bool {
        cli::logged_in(&self.path, &self.env)
    }
}

/// The content blocks of a question: the image first, then the prompt.
pub fn content(prompt: &str, image: Option<&Image>) -> Value {
    let img = image.map(|i| json!({"type": "image", "source": {"type": "base64", "media_type": i.media, "data": i.data}}));
    Value::Array(img.into_iter().chain([json!({"type": "text", "text": prompt})]).collect())
}

/// The model identifier of the model `name` of `[name_models]` (NAMES-18).
pub fn api_model(name: &str) -> String {
    let full = match name.trim().to_lowercase().as_str() {
        "haiku" => "claude-haiku-5-5",
        "sonnet" => "claude-sonnet-5-5",
        "opus" => "claude-opus-5-5",
        _ => return name.trim().to_string(),
    };
    full.to_string()
}

/// The media type of the image at `path`, from its extension; PNG by default, as a paste writes.
pub fn media_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).map(str::to_lowercase).as_deref() {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "image/png",
    }
}

/// The program `name` on the `PATH`, if there is one.
pub fn on_path(name: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?).map(|d| d.join(name)).find(|p| p.is_file())
}

#[cfg(test)]
#[path = "api_tests.rs"]
mod tests;
