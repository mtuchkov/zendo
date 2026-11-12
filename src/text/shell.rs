//! Shell blocks in a message of the agent (docs/zendo/terminal.md): the command that `F5` puts
//! into a terminal.

/// The languages of a fenced code block that hold shell commands (TERM-7).
const LANGS: &[&str] = &["sh", "bash", "zsh", "fish", "shell", "console"];

/// A shell block: the source lines of its fences, and its command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub fence: usize,
    /// The closing fence; the line after the last line when the fence is open.
    pub close: usize,
    pub command: String,
}

/// The text after the prompt `$ `, when the line has one.
fn after_prompt(line: &str) -> Option<&str> {
    line.trim_start().strip_prefix("$ ").or_else(|| (line.trim() == "$").then_some(""))
}

/// The command of a block of `lang`, if it is a shell block (TERM-7, TERM-9).
fn command(lang: &str, lines: &[&str]) -> Option<String> {
    let text: Vec<&str> = lines.iter().copied().filter(|l| !l.trim().is_empty()).collect();
    let prompts: Vec<&str> = text.iter().filter_map(|l| after_prompt(l)).collect();
    let shell = LANGS.contains(&lang.to_lowercase().as_str());
    let all_prompts = !text.is_empty() && prompts.len() == text.len();
    let out = match (shell, prompts.is_empty()) {
        (true, true) => lines.join("\n"),
        (true, false) => prompts.join("\n"),
        (false, _) if all_prompts => prompts.join("\n"),
        _ => return None,
    };
    let out = out.trim_matches('\n').to_string();
    (!out.trim().is_empty()).then_some(out)
}

/// The shell blocks of a markdown text, in their order.
pub fn blocks(src: &str) -> Vec<Block> {
    let lines: Vec<&str> = src.lines().collect();
    let fence = |l: &str| l.trim_start().starts_with("```");
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if !fence(lines[i]) {
            i += 1;
            continue;
        }
        let indent = lines[i].len() - lines[i].trim_start().len();
        let lang = lines[i].trim_start().trim_start_matches('`').split_whitespace().next().unwrap_or("");
        let close = (i + 1..lines.len()).find(|j| fence(lines[*j])).unwrap_or(lines.len());
        let body: Vec<&str> = lines[i + 1..close].iter().map(|l| l.get(indent..).filter(|_| l.starts_with(&lines[i][..indent])).unwrap_or(l)).collect();
        if let Some(command) = command(lang, &body) {
            out.push(Block { fence: i, close, command });
        }
        i = close + 1;
    }
    out
}

/// The command as the shell gets it. With bracketed paste the shell keeps the lines until `enter`;
/// without it each new line would run, so the lines join into one: a line that ends with `\`
/// goes on in the next, the other lines join with `&&`.
pub fn for_shell(command: &str, bracketed: bool) -> String {
    if bracketed {
        return command.to_string();
    }
    let mut out = String::new();
    let mut go_on = false;
    for line in command.lines().map(str::trim).filter(|l| !l.is_empty()) {
        if !out.is_empty() {
            out.push_str(if go_on { " " } else { " && " });
        }
        go_on = line.ends_with('\\');
        out.push_str(line.trim_end_matches('\\').trim_end());
    }
    out
}

#[cfg(test)]
#[path = "shell_tests.rs"]
mod tests;
