//! The help of the keys: the pane that it shows, the keymap file of the config, and what is wrong
//! in the keymap (ADR 0008).
use crate::keys::Pane;

#[derive(Default)]
pub struct KeyHelp {
    /// The pane whose keys the help shows.
    pane: Pane,
    /// What is wrong in `[keys]` of the config.
    problems: Vec<String>,
    /// The keymap file that the config names (PANES-31).
    file: Option<String>,
}

impl KeyHelp {
    pub fn new(problems: Vec<String>, file: Option<String>) -> Self {
        Self { pane: Pane::default(), problems, file }
    }

    pub fn pane(&self) -> Pane {
        self.pane
    }

    /// The help shows the keys of `pane`.
    pub fn show(&mut self, pane: Pane) {
        self.pane = pane;
    }

    pub fn problems(&self) -> &[String] {
        &self.problems
    }

    pub fn file(&self) -> Option<&String> {
        self.file.as_ref()
    }
}

#[cfg(test)]
#[path = "key_help_tests.rs"]
mod tests;
