//! The questions to the light model, each on a thread of its own: the tick of the app takes the
//! answers, so the UI never waits for `claude -p` (ADR 0009, ADR 0010).
use super::*;
use chrono::{DateTime, Local};
use std::sync::mpsc::{Receiver, Sender, channel};

/// What a question asks for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Question {
    /// The name of agent session `n`, whose Claude session is `id` (NAMES-1).
    Name { n: usize, id: String },
    /// The name of the pasted image at this path (CLAUDE-37).
    Image(PathBuf),
    /// The title of the note of the float that opened at `opened`; `old` is the title of the agent
    /// that the answer replaces (FLOAT-18, FLOAT-19).
    Title { opened: DateTime<Local>, old: Option<String> },
    /// Free keys for the key float, for the question with this number (REKEY-6, REKEY-5).
    Keys(u64),
}

/// A question and the text of its answer, or the error.
pub type Answer = (Question, Result<String, String>);

/// The questions on their threads, and the answers that came.
pub struct Asker {
    tx: Sender<Answer>,
    rx: Receiver<Answer>,
    /// The questions without an answer yet.
    open: Vec<Question>,
}

impl Default for Asker {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self { tx, rx, open: Vec::new() }
    }
}

impl Asker {
    /// Ask `api` with `prompt` and the image at `image`, on a thread.
    pub fn ask(&mut self, api: &Api, q: Question, prompt: String, image: Option<PathBuf>) {
        let (api, tx) = (api.clone(), self.tx.clone());
        self.open.push(q.clone());
        std::thread::spawn(move || {
            let _ = tx.send((q, ask(&api, &prompt, image.as_deref())));
        });
    }

    /// The answers that came since the last call.
    pub fn answers(&mut self) -> Vec<Answer> {
        let answers: Vec<Answer> = self.rx.try_iter().collect();
        self.open.retain(|q| !answers.iter().any(|(a, _)| a == q));
        answers
    }

    /// The questions without an answer yet.
    pub fn open(&self) -> &[Question] {
        &self.open
    }
}

/// One question to `api` through `claude -p`: the text of the answer, or the error (NAMES-21).
pub fn ask(api: &Api, prompt: &str, image: Option<&Path>) -> Result<String, String> {
    let image = image.map(read_image).transpose()?;
    cli::ask(api, prompt, image.as_ref())
}

/// The image at `path` in base64.
fn read_image(path: &Path) -> Result<Image, String> {
    use base64::Engine;
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(Image { media: media_type(path), data: base64::engine::general_purpose::STANDARD.encode(bytes) })
}
