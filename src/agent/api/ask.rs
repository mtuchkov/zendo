//! The questions to the API, each on a thread of its own: the tick of the app takes the answers,
//! so the UI never waits for the network (ADR 0009, ADR 0010).
use super::*;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

/// The longest time of one question.
const LIMIT: Duration = Duration::from_secs(20);

/// What a question asks for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Question {
    /// The name of agent session `n`, whose Claude session is `id` (NAMES-1).
    Name { n: usize, id: String },
    /// The name of the pasted image at this path (CLAUDE-37).
    Image(PathBuf),
}

/// A question and the text of its answer, or the error.
pub type Answer = (Question, Result<String, String>);

/// The questions on their threads, and the answers that came.
pub struct Asker {
    tx: Sender<Answer>,
    rx: Receiver<Answer>,
}

impl Default for Asker {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self { tx, rx }
    }
}

impl Asker {
    /// Ask `api` with `prompt` and the image at `image`, on a thread.
    pub fn ask(&self, api: &Api, q: Question, prompt: String, image: Option<PathBuf>) {
        let (api, tx) = (api.clone(), self.tx.clone());
        std::thread::spawn(move || {
            let _ = tx.send((q, ask(&api, &prompt, image.as_deref())));
        });
    }

    /// The answers that came since the last call.
    pub fn answers(&self) -> Vec<Answer> {
        self.rx.try_iter().collect()
    }
}

/// One question to `api`: the text of the answer, or the error. The proxy of the environment
/// applies.
pub fn ask(api: &Api, prompt: &str, image: Option<&Path>) -> Result<String, String> {
    let image = image.map(read_image).transpose()?;
    let token = api.token()?;
    let agent: ureq::Agent = ureq::Agent::config_builder().timeout_global(Some(LIMIT)).http_status_as_error(false).build().into();
    let req = api.headers(&token).into_iter().fold(agent.post(&api.url()), |r, (k, v)| r.header(k, v));
    let mut resp = req.send_json(api.body(prompt, image.as_ref())).map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    let v: Value = resp.body_mut().read_json().unwrap_or(Value::Null);
    match status {
        200..=299 => answer_text(api.vendor, &v).ok_or_else(|| "no text in the answer".to_string()),
        _ => Err(error_text(status, &v)),
    }
}

/// The image at `path` in base64.
fn read_image(path: &Path) -> Result<Image, String> {
    use base64::Engine;
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Image { media: media_type(path), data: base64::engine::general_purpose::STANDARD.encode(bytes) })
}
