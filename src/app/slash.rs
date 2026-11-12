//! `/` at the start of the chat input: the slash commands of the agent. zendo passes them through:
//! the list comes from the agent, and the command goes to the agent as the user typed it.
use super::*;
use crate::agent::Command;

/// The commands of the agent, and the open suggestions.
#[derive(Clone, Debug, Default)]
pub struct Slash {
    pub commands: Vec<Command>,
    /// The commands that match what follows `/`, best first, and the selected one.
    pub hits: Vec<Command>,
    pub sel: usize,
    pub open: bool,
}

/// The commands whose name starts with `q`, then the ones whose name has it.
pub fn matches(commands: &[Command], q: &str) -> Vec<Command> {
    let q = q.to_lowercase();
    let starts = commands.iter().filter(|c| c.name.to_lowercase().starts_with(&q));
    let has = commands.iter().filter(|c| !c.name.to_lowercase().starts_with(&q) && c.name.to_lowercase().contains(&q));
    starts.chain(has).cloned().collect()
}

impl App {
    /// The name after `/` when the cursor is in the first word of the input and it starts with `/`.
    fn slash_token(&self) -> Option<String> {
        let e = &self.input;
        let before: String = e.lines.first()?.chars().take(e.col).collect();
        let name = before.strip_prefix('/')?;
        (e.row == 0 && !name.contains(char::is_whitespace)).then(|| name.to_string())
    }

    /// After a key in the chat input: open, update or close the suggestions.
    pub(crate) fn update_slash(&mut self) {
        let token = self.slash_token().filter(|_| !self.slash.commands.is_empty());
        let Some(q) = token else {
            self.slash.open = false;
            return;
        };
        let hits = matches(&self.slash.commands, &q);
        if !self.slash.open || hits != self.slash.hits {
            self.slash.sel = 0;
        }
        self.slash.hits = hits;
        self.slash.open = true;
    }

    /// The keys of the open suggestions: the arrows choose, `tab` takes one, `esc` closes. `enter`
    /// takes one when the name is not complete yet; else it sends. True when the key was used.
    pub(crate) fn key_slash(&mut self, k: KeyEvent) -> bool {
        let n = self.slash.hits.len();
        let exact = self.slash_token().is_some_and(|q| self.slash.hits.iter().any(|c| c.name == q));
        match k.code {
            KeyCode::Down if n > 0 => self.slash.sel = (self.slash.sel + 1).min(n - 1),
            KeyCode::Up => self.slash.sel = self.slash.sel.saturating_sub(1),
            KeyCode::Esc => self.slash.open = false,
            KeyCode::Tab if n > 0 => self.take_slash(),
            KeyCode::Enter if n > 0 && !exact && k.modifiers.is_empty() => self.take_slash(),
            _ => return false,
        }
        true
    }

    /// Put the selected command in place of the first word, and a space after it.
    fn take_slash(&mut self) {
        let Some(c) = self.slash.hits.get(self.slash.sel).cloned() else { return };
        let line = &mut self.input.lines[0];
        let rest: String = line.chars().skip_while(|ch| !ch.is_whitespace()).collect();
        let rest = rest.trim_start();
        *line = format!("/{} {rest}", c.name);
        self.input.col = c.name.chars().count() + 2;
        self.input.dirty = true;
        self.slash.open = false;
    }
}

#[cfg(test)]
#[path = "slash_tests.rs"]
mod tests;
