//! The recipe of the terminal font (docs/zendo/fonts-recipe-format.md): one recipe for each
//! terminal in `~/.zendo/terminal-fonts.toml`, written by the agent; zendo sets a font only by it.
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// How to set a font in one terminal.
#[derive(Default, Deserialize, Debug, PartialEq)]
pub struct Recipe {
    pub file: Option<String>,
    pub prefix: Option<String>,
    #[serde(default)]
    pub whole: bool,
    pub line: Option<String>,
    pub command: Option<Vec<String>>,
    pub escape: Option<String>,
    pub reload: Option<Vec<String>>,
    pub size: Option<Vec<String>>,
    /// The font that zendo set last (FONT-34).
    pub font: Option<String>,
}

/// The recipes of all terminals, by the name of the terminal (FONT-32).
#[derive(Default, Debug)]
pub struct Recipes(BTreeMap<String, Recipe>);

/// Runs a command with its arguments, without a shell, and gives its output (FONT-30).
pub trait Run {
    fn run(&self, argv: &[String]) -> Result<String, String>;
}

impl Recipes {
    pub fn parse(text: &str) -> Result<Recipes, String> {
        toml::from_str(text).map(Recipes).map_err(|e| e.to_string())
    }

    /// The recipes in the file; none when there is no file.
    pub fn load(path: &Path) -> Result<Recipes, String> {
        match std::fs::read_to_string(path) {
            Ok(text) => Recipes::parse(&text).map_err(|e| format!("{}: {e}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Recipes::default()),
            Err(e) => Err(format!("{}: {e}", path.display())),
        }
    }

    pub fn get(&self, terminal: &str) -> Option<&Recipe> {
        self.0.get(terminal)
    }
}

impl Recipe {
    /// Set the font: the file, the command, the reload; the escape sequence comes back, for the
    /// caller to write to the terminal (FONT-29, FONT-40, FONT-41).
    pub fn set(&self, family: &str, run: &dyn Run) -> Result<Option<String>, String> {
        if !self.has_way() {
            return Err("the recipe has no way to set a font".into());
        }
        let size = if self.texts().any(|t| t.contains("{size}")) { Some(self.current_size(run)?) } else { None };
        let fill = |t: &str| t.replace("{font}", family).replace("{size}", size.as_deref().unwrap_or(""));
        if let (Some(file), Some(line)) = (&self.file, &self.line) {
            write_line(&expand(file), self.prefix.as_deref(), self.whole, &fill(line))?;
        }
        if let Some(cmd) = &self.command {
            run.run(&cmd.iter().map(|a| fill(a)).collect::<Vec<_>>())?;
        }
        if let Some(reload) = &self.reload {
            run.run(reload)?;
        }
        Ok(self.escape.as_deref().map(fill))
    }

    /// The problems of the recipe: a file that is not there, a command that is not on the `PATH`
    /// (FONT-31).
    pub fn check(&self, on_path: &dyn Fn(&str) -> bool) -> Result<(), String> {
        if !self.has_way() {
            return Err("the recipe has no way to set a font".into());
        }
        let file = self.file.as_deref().map(expand).filter(|f| !f.exists()).map(|f| format!("{} is not there", f.display()));
        let commands = [&self.command, &self.reload, &self.size].into_iter().flatten().filter_map(|c| c.first());
        let missing = commands.filter(|c| if c.contains('/') { !expand(c).exists() } else { !on_path(c) }).map(|c| format!("`{c}` is not on the PATH"));
        let problems: Vec<String> = file.into_iter().chain(missing).collect();
        if problems.is_empty() { Ok(()) } else { Err(problems.join("; ")) }
    }

    fn has_way(&self) -> bool {
        (self.file.is_some() && self.line.is_some()) || self.command.as_ref().is_some_and(|c| !c.is_empty()) || self.escape.is_some()
    }

    /// The texts that can hold `{font}` and `{size}`.
    fn texts(&self) -> impl Iterator<Item = &String> {
        self.line.iter().chain(self.command.iter().flatten()).chain(self.escape.iter())
    }

    /// The last number that the `size` command prints (FONT-41, FONT-43).
    fn current_size(&self, run: &dyn Run) -> Result<String, String> {
        let cmd = self.size.as_ref().ok_or("the recipe has `{size}` but no `size` command")?;
        let out = run.run(cmd)?;
        let mut numbers = out.split(|c: char| !(c.is_ascii_digit() || c == '.')).filter(|w| w.chars().any(|c| c.is_ascii_digit()));
        numbers.next_back().map(|n| n.trim_matches('.').to_string()).ok_or_else(|| format!("the `size` command printed no size: {}", out.trim()))
    }
}

/// `~/` at the start of a path is the home folder.
fn expand(path: &str) -> PathBuf {
    match path.strip_prefix("~/") {
        Some(rest) => std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default().join(rest),
        None => PathBuf::from(path),
    }
}

/// Replace the first line that starts with `prefix`, or add the line at the end; with `whole`,
/// the line is the whole file (FONT-40).
fn write_line(path: &Path, prefix: Option<&str>, whole: bool, line: &str) -> Result<(), String> {
    let fail = |e: std::io::Error| format!("{}: {e}", path.display());
    if whole {
        return crate::atomic::write(path, line).map_err(fail);
    }
    let prefix = prefix.ok_or("the recipe has a file without a `prefix`")?;
    let text = std::fs::read_to_string(path).map_err(fail)?;
    let mut done = false;
    let mut out: String = text
        .split_inclusive('\n')
        .map(|l| match !done && l.trim_start().starts_with(prefix) {
            true => {
                done = true;
                format!("{line}{}", if l.ends_with('\n') { "\n" } else { "" })
            }
            false => l.to_string(),
        })
        .collect();
    if !done {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(line);
        out.push('\n');
    }
    crate::atomic::write(path, out).map_err(fail)
}

/// Keep the font that zendo set in the recipe of the terminal; the rest of the file stays.
pub fn remember(path: &Path, terminal: &str, family: &str) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut doc: toml_edit::DocumentMut = text.parse().map_err(|e: toml_edit::TomlError| e.to_string())?;
    let table = doc.get_mut(terminal).and_then(|t| t.as_table_mut()).ok_or_else(|| format!("no recipe for {terminal}"))?;
    table["font"] = toml_edit::value(family);
    crate::atomic::write(path, doc.to_string()).map_err(|e| format!("{}: {e}", path.display()))
}

/// The variables that terminals set, where `TERM_PROGRAM` is not set (FONT-42).
const KNOWN: [(&str, &str); 6] = [
    ("GHOSTTY_RESOURCES_DIR", "ghostty"),
    ("KITTY_WINDOW_ID", "kitty"),
    ("ALACRITTY_WINDOW_ID", "alacritty"),
    ("WEZTERM_PANE", "wezterm"),
    ("GNOME_TERMINAL_SERVICE", "gnome-terminal"),
    ("VTE_VERSION", "vte"),
];

/// The name of the terminal: `TERM_PROGRAM`, else a variable of a known terminal, else `TERM`.
pub fn terminal(var: &dyn Fn(&str) -> Option<String>) -> String {
    let get = |k: &str| var(k).filter(|v| !v.is_empty());
    let known = || KNOWN.iter().find(|(k, _)| get(k).is_some()).map(|(_, n)| n.to_string());
    get("TERM_PROGRAM").map(|p| p.to_lowercase()).or_else(known).or_else(|| get("TERM")).unwrap_or_else(|| "unknown".into())
}

/// The terminal of this process.
pub fn current_terminal() -> String {
    terminal(&|k| std::env::var(k).ok())
}

/// The program is on the `PATH`.
pub fn on_path(program: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|p| std::env::split_paths(&p).any(|d| d.join(program).is_file()))
}

/// The commands of a recipe on this machine: no shell, no input (FONT-30).
pub struct System;

impl Run for System {
    fn run(&self, argv: &[String]) -> Result<String, String> {
        let (program, args) = argv.split_first().ok_or("an empty command")?;
        let out = std::process::Command::new(program).args(args).stdin(std::process::Stdio::null()).output().map_err(|e| format!("`{program}`: {e}"))?;
        match out.status.success() {
            true => Ok(String::from_utf8_lossy(&out.stdout).into_owned()),
            false => Err(format!("`{program}`: {}", String::from_utf8_lossy(&out.stderr).trim())),
        }
    }
}

/// `~/.zendo/terminal-fonts.toml`.
pub fn path() -> PathBuf {
    crate::setup::config::config_dir().join("terminal-fonts.toml")
}

#[cfg(test)]
#[path = "recipe_tests.rs"]
mod tests;
