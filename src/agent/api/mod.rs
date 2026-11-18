//! The model API of the configured agent, for the questions that need one word back: the names of
//! the agent sessions, of the pasted images and the titles of new notes
//! (docs/zendo/session-names/session-names-api.md, NAMES-15 to NAMES-21, ADR 0010). Claude asks the Anthropic Messages API, Codex the OpenAI
//! Responses API; a Claude user with a subscription only asks through `claude -p` (NAMES-21).
//! Without any of these there is no API, and zendo asks the agent in a throwaway session
//! (NAMES-16).

mod ask;
mod cli;

pub use ask::{Answer, Asker, Question};

use crate::setup::config::Kind;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The vendor of the API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vendor {
    Anthropic,
    OpenAi,
}

/// How zendo proves itself to the API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Credential {
    /// An API key from the environment or from `[env]` of the config.
    Key(String),
    /// The `ant` CLI at this path: its sign-in gives a token for each question (NAMES-20).
    Ant(PathBuf),
    /// The `claude` CLI at this path, with the login of the user, also a subscription (NAMES-21).
    Claude(PathBuf),
}

/// The API of the configured agent: where it is, the model and the credential.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Api {
    pub vendor: Vendor,
    /// The base URL: without `/v1` for Anthropic, with it for OpenAI, as the agents take it.
    pub base: String,
    pub model: String,
    pub cred: Credential,
    /// The effort of the model, as `low`: the key hints ask with low effort (REKEY-6).
    pub effort: Option<String>,
}

/// An image for the question, in base64.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    pub media: &'static str,
    pub data: String,
}

/// The longest answer, in tokens. A reasoning model of OpenAI counts its thinking too.
const ANTHROPIC_TOKENS: u32 = 128;
const OPENAI_TOKENS: u32 = 2048;

impl Api {
    /// The API of an agent of `kind` with the name model `model`, if it has a credential: for Claude
    /// `ANTHROPIC_API_KEY`, else the `ant` CLI, else the `claude` CLI; for Codex `OPENAI_API_KEY`,
    /// else `CODEX_API_KEY` (NAMES-15, NAMES-16, NAMES-20, NAMES-21). `env` reads a variable, and
    /// `which` finds a program on the `PATH`.
    pub fn find(kind: Kind, model: Option<String>, env: &dyn Fn(&str) -> Option<String>, which: &dyn Fn(&str) -> Option<PathBuf>) -> Option<Self> {
        let var = |k: &str| env(k).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        let (vendor, base, cred) = match kind {
            Kind::Claude => {
                let cli = || which("ant").map(Credential::Ant).or_else(|| which("claude").map(Credential::Claude));
                let cred = var("ANTHROPIC_API_KEY").map(Credential::Key).or_else(cli)?;
                (Vendor::Anthropic, var("ANTHROPIC_BASE_URL").unwrap_or_else(|| "https://api.anthropic.com".into()), cred)
            }
            Kind::Codex => {
                let key = var("OPENAI_API_KEY").or_else(|| var("CODEX_API_KEY"))?;
                (Vendor::OpenAi, var("OPENAI_BASE_URL").unwrap_or_else(|| "https://api.openai.com/v1".into()), Credential::Key(key))
            }
            Kind::Other => return None,
        };
        let model = api_model(vendor, &model?);
        Some(Self { vendor, base: base.trim_end_matches('/').to_string(), model, cred, effort: None })
    }

    /// The URL of a question.
    pub fn url(&self) -> String {
        match self.vendor {
            Vendor::Anthropic => format!("{}/v1/messages", self.base),
            Vendor::OpenAi => format!("{}/responses", self.base),
        }
    }

    /// The secret for the headers: the key, or a fresh token of `ant` (NAMES-20). The `claude` CLI
    /// keeps its login to itself.
    pub fn token(&self) -> Result<String, String> {
        match &self.cred {
            Credential::Key(k) => Ok(k.clone()),
            Credential::Ant(path) => ant_token(path),
            Credential::Claude(_) => Err("the claude CLI has no token for zendo".into()),
        }
    }

    /// True when the credential works now: a key always, `ant` and `claude` when they are signed
    /// in (NAMES-20, NAMES-21).
    pub fn usable(&self) -> bool {
        match &self.cred {
            Credential::Claude(path) => cli::logged_in(path),
            _ => self.token().is_ok(),
        }
    }

    /// The headers of a question with the secret `token`.
    pub fn headers(&self, token: &str) -> Vec<(String, String)> {
        let mut h = vec![("content-type".to_string(), "application/json".to_string())];
        match (self.vendor, &self.cred) {
            (Vendor::Anthropic, Credential::Key(_)) => h.push(("x-api-key".into(), token.into())),
            _ => h.push(("authorization".into(), format!("Bearer {token}"))),
        }
        if self.vendor == Vendor::Anthropic {
            h.push(("anthropic-version".into(), "2023-06-01".into()));
        }
        h
    }

    /// The body of a question: the prompt, and the image before it for Anthropic, after it for
    /// OpenAI, as their docs show.
    pub fn body(&self, prompt: &str, image: Option<&Image>) -> Value {
        let mut body = self.plain_body(prompt, image);
        if let Some(e) = &self.effort {
            let key = if self.vendor == Vendor::Anthropic { "output_config" } else { "reasoning" };
            body[key] = json!({"effort": e});
        }
        body
    }

    fn plain_body(&self, prompt: &str, image: Option<&Image>) -> Value {
        match self.vendor {
            Vendor::Anthropic => {
                let img = image.map(|i| json!({"type": "image", "source": {"type": "base64", "media_type": i.media, "data": i.data}}));
                let content: Vec<Value> = img.into_iter().chain([json!({"type": "text", "text": prompt})]).collect();
                json!({"model": self.model, "max_tokens": ANTHROPIC_TOKENS, "messages": [{"role": "user", "content": content}]})
            }
            Vendor::OpenAi => {
                let img = image.map(|i| json!({"type": "input_image", "image_url": format!("data:{};base64,{}", i.media, i.data)}));
                let content: Vec<Value> = [json!({"type": "input_text", "text": prompt})].into_iter().chain(img).collect();
                json!({"model": self.model, "max_output_tokens": OPENAI_TOKENS, "store": false, "input": [{"role": "user", "content": content}]})
            }
        }
    }
}

/// The API identifier of the model `name` of `[name_models]` (NAMES-18).
pub fn api_model(vendor: Vendor, name: &str) -> String {
    let short = name.trim().to_lowercase();
    let full = match (vendor, short.as_str()) {
        (Vendor::Anthropic, "haiku") => "claude-haiku-5-5",
        (Vendor::Anthropic, "sonnet") => "claude-sonnet-5-5",
        (Vendor::Anthropic, "opus") => "claude-opus-5-5",
        (Vendor::OpenAi, "luna") => "gpt-6-luna",
        (Vendor::OpenAi, "sol") => "gpt-6-sol",
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

/// The text of an answer: the text blocks of Anthropic, the output text of OpenAI.
pub fn answer_text(vendor: Vendor, v: &Value) -> Option<String> {
    let texts = |items: &Value, kind: &str| -> Vec<String> {
        items.as_array().into_iter().flatten().filter(|b| b["type"] == kind).filter_map(|b| b["text"].as_str().map(String::from)).collect()
    };
    let parts = match vendor {
        Vendor::Anthropic => texts(&v["content"], "text"),
        Vendor::OpenAi => v["output"].as_array().into_iter().flatten().filter(|o| o["type"] == "message").flat_map(|o| texts(&o["content"], "output_text")).collect(),
    };
    (!parts.is_empty()).then(|| parts.concat())
}

/// The error of an HTTP answer with `status`: the message of the API, when the body has one.
pub fn error_text(status: u16, v: &Value) -> String {
    match v["error"]["message"].as_str() {
        Some(m) => format!("HTTP {status}: {m}"),
        None => format!("HTTP {status}"),
    }
}

/// The access token of the active profile of `ant`; `ant` refreshes it when it is old (NAMES-20).
fn ant_token(path: &Path) -> Result<String, String> {
    let out = Command::new(path).args(["auth", "print-credentials", "--access-token"]).output().map_err(|e| format!("ant: {e}"))?;
    let token = String::from_utf8_lossy(&out.stdout).trim().to_string();
    match out.status.success() && !token.is_empty() && !token.contains(char::is_whitespace) {
        true => Ok(token),
        false => Err(format!("ant: {}", String::from_utf8_lossy(&out.stderr).lines().next().unwrap_or("no sign-in"))),
    }
}

/// The program `name` on the `PATH`, if there is one.
pub fn on_path(name: &str) -> Option<PathBuf> {
    std::env::split_paths(&std::env::var_os("PATH")?).map(|d| d.join(name)).find(|p| p.is_file())
}

#[cfg(test)]
#[path = "api_tests.rs"]
mod tests;
