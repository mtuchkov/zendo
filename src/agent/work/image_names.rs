//! The names of pasted images (docs/zendo/chat.md): the chip of an image says `Image #n` until the
//! name model names the image in a throwaway session; then it says `Image: <name>` (CLAUDE-37 to
//! CLAUDE-39). The question is in no thread.
use super::*;
use crate::agent::{Attachment, Event};
use std::path::PathBuf;

pub const IMAGE_PREFIX: &str = "(image) ";
/// The longest name, in chars.
const LONGEST: usize = 24;

pub const IMAGE_PROMPT: &str = "Name this image with one to three short lowercase words that tell what it \
shows, such as `login form` or `error dialog`. Answer with the name alone on one line.";

impl App {
    /// Ask the name model for a name of the pasted image at `path`, where the agent takes images
    /// (CLAUDE-37, CLAUDE-39).
    pub(crate) fn name_image(&mut self, path: PathBuf) {
        let Some(a) = self.agent.as_ref().filter(|_| self.agent_images) else { return };
        let key = format!("{IMAGE_PREFIX}{}", path.display());
        if let Some(model) = self.name_model.clone() {
            a.send(Cmd::Model { key: key.clone(), model });
        }
        a.send(Cmd::Prompt { key: key.clone(), known: None, text: IMAGE_PROMPT.into(), attach: vec![Attachment::Image(path.clone())] });
        self.image_names.insert(key, (path, String::new()));
    }

    /// An event of a throwaway session that names an image.
    pub(crate) fn on_image_event(&mut self, e: Event) {
        match e {
            Event::Text { key, text } => self.image_names.get_mut(&key).into_iter().for_each(|(_, buf)| buf.push_str(&text)),
            Event::Permission { request, .. } => self.agent.iter().for_each(|a| a.send(Cmd::Answer { request, option: None })),
            Event::TurnEnd { key, .. } => self.image_named(&key, true),
            Event::Error { key: Some(key), .. } => self.image_named(&key, false),
            _ => {}
        }
    }

    /// The naming turn ended: the name labels the chip of the image, while it is in the input
    /// (CLAUDE-38).
    fn image_named(&mut self, key: &str, ok: bool) {
        let Some((path, buf)) = self.image_names.remove(key) else { return };
        if let Some(a) = &self.agent {
            a.send(Cmd::Forget { key: key.to_string() });
        }
        let has = |c: &crate::app::Chip| matches!(&c.attach, Some(Attachment::Image(p)) if *p == path);
        if let (true, Some(name), Some(old)) = (ok, image_name(&buf), self.composer.chips().iter().find(|c| has(c)).map(|c| c.label.clone())) {
            self.composer.rename_chip(&old, &format!("Image: {name}"));
        }
    }
}

/// The name in the answer: the last line that has letters, in lowercase words, cut at a word to
/// `LONGEST` chars.
pub fn image_name(answer: &str) -> Option<String> {
    let line = answer.lines().rev().find(|l| l.chars().any(char::is_alphanumeric))?;
    let words = line.split(|c: char| !c.is_alphanumeric() && c != '-').filter(|w| !w.is_empty()).map(str::to_lowercase);
    let name = words.fold(String::new(), |acc, w| match acc.is_empty() {
        true => w.chars().take(LONGEST).collect(),
        false if acc.chars().count() + 1 + w.chars().count() <= LONGEST => format!("{acc} {w}"),
        false => acc,
    });
    (!name.is_empty()).then_some(name)
}

#[cfg(test)]
#[path = "image_names_tests.rs"]
mod tests;
