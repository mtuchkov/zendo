//! Code highlighting for fenced code blocks and front matter.
//!
//! Every language is a plugin: a Sublime syntax definition. zendo ships the definitions of syntect
//! and turns on `sh`, `rust`, `java`, `python`, `go`, `yaml` and `json`. `[highlight] languages` in
//! `~/.zendo/config.toml` turns on others of them by name, and a `.sublime-syntax` file in
//! `~/.zendo/syntaxes/` adds a language that is on at once. The colors come from the zendo theme.

use crate::ui::theme::Theme;
use ratatui::style::{Modifier, Style};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet};

pub const DEFAULT_LANGUAGES: [&str; 7] = ["sh", "rust", "java", "python", "go", "yaml", "json"];

/// `[highlight]` in the config.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// The languages that are on, by name or file extension. The default is DEFAULT_LANGUAGES.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub languages: Option<Vec<String>>,
}

struct Engine {
    set: SyntaxSet,
    /// The names of the syntaxes that are on.
    on: BTreeSet<String>,
    /// Plugin files that did not load, with the reason.
    problems: Vec<String>,
}

static ENGINE: OnceLock<Engine> = OnceLock::new();
type Cache = HashMap<(String, u64), Vec<Vec<(Kind, String)>>>;
static CACHE: OnceLock<Mutex<Cache>> = OnceLock::new();

pub fn plugin_dir() -> PathBuf {
    crate::setup::config::config_dir().join("syntaxes")
}

fn build(cfg: Option<&Config>, plugins: &Path) -> Engine {
    let mut problems = Vec::new();
    let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
    let mut plugin_names = Vec::new();
    if let Ok(dir) = std::fs::read_dir(plugins) {
        for e in dir.flatten() {
            let p = e.path();
            if p.extension().is_some_and(|x| x == "sublime-syntax") {
                match std::fs::read_to_string(&p).map_err(|e| e.to_string()).and_then(|t| syntect::parsing::SyntaxDefinition::load_from_str(&t, true, None).map_err(|e| e.to_string())) {
                    Ok(def) => {
                        plugin_names.push(def.name.clone());
                        builder.add(def);
                    }
                    Err(e) => problems.push(format!("{}: {e}", p.display())),
                }
            }
        }
    }
    let set = builder.build();
    let wanted: Vec<String> = cfg.and_then(|c| c.languages.clone()).unwrap_or_else(|| DEFAULT_LANGUAGES.iter().map(|s| s.to_string()).collect());
    let mut on: BTreeSet<String> = plugin_names.into_iter().collect();
    for w in wanted {
        match find(&set, &w) {
            Some(s) => {
                on.insert(s.name.clone());
            }
            None => problems.push(format!("[highlight] languages: no syntax for \"{w}\"")),
        }
    }
    Engine { set, on, problems }
}

fn find<'a>(set: &'a SyntaxSet, lang: &str) -> Option<&'a SyntaxReference> {
    let l = lang.trim().to_lowercase();
    let l = match l.as_str() {
        "shell" | "bash" | "zsh" | "sh" | "console" => "sh",
        "yml" => "yaml",
        "py" => "python",
        "rs" => "rust",
        "golang" => "go",
        other => other,
    };
    set.find_syntax_by_token(l).or_else(|| set.syntaxes().iter().find(|s| s.name.to_lowercase() == l))
}

/// Set up the languages from the config and the plugin folder. Without a call, the defaults apply.
pub fn init(cfg: Option<&Config>) {
    let _ = ENGINE.set(build(cfg, &plugin_dir()));
}

fn engine() -> &'static Engine {
    ENGINE.get_or_init(|| build(None, &plugin_dir()))
}

/// What is wrong with the plugins and the language list, for `zendo doctor`.
pub fn problems(cfg: Option<&Config>) -> (Vec<String>, Vec<String>) {
    let e = build(cfg, &plugin_dir());
    (e.on.into_iter().collect(), e.problems)
}

/// The kind of a piece of code; the theme gives it a color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Plain,
    Comment,
    String,
    Constant,
    Keyword,
    Name,
}

impl Kind {
    pub fn style(self, base: Style, t: &Theme) -> Style {
        match self {
            Kind::Plain => base,
            Kind::Comment => base.fg(t.dim).add_modifier(Modifier::ITALIC),
            Kind::String => base.fg(t.add),
            Kind::Constant => base.fg(t.del),
            Kind::Keyword => base.fg(t.acc),
            Kind::Name => base.fg(t.acc2),
        }
    }
}

fn kind_of(stack: &ScopeStack) -> Kind {
    for s in stack.as_slice().iter().rev() {
        let name = s.build_string();
        let k = classify(&name);
        if k != Kind::Plain {
            return k;
        }
    }
    Kind::Plain
}

fn classify(scope: &str) -> Kind {
    let starts = |p: &str| scope == p || scope.starts_with(&format!("{p}."));
    if starts("comment") {
        Kind::Comment
    } else if starts("string") || starts("markup.raw") {
        Kind::String
    } else if starts("constant.numeric") || starts("constant.language") || starts("constant.character") || starts("constant.other") {
        Kind::Constant
    } else if starts("keyword") || starts("storage") {
        Kind::Keyword
    } else if starts("entity.name") || starts("support.function") || starts("support.type") || starts("support.class") || starts("entity.other.attribute-name") {
        Kind::Name
    } else {
        Kind::Plain
    }
}

/// The lines of a code block, cut into pieces of a kind. None when the language is off or unknown.
pub fn pieces(lang: &str, lines: &[String]) -> Option<Vec<Vec<(Kind, String)>>> {
    let e = engine();
    let syntax = find(&e.set, lang).filter(|s| e.on.contains(&s.name))?;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(lines, &mut h);
    let key = (syntax.name.clone(), std::hash::Hasher::finish(&h));
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(v) = cache.lock().ok().and_then(|c| c.get(&key).cloned()) {
        return Some(v);
    }
    let mut state = ParseState::new(syntax);
    let mut stack = ScopeStack::new();
    let mut out = Vec::with_capacity(lines.len());
    for line in lines {
        let text = format!("{line}\n");
        let ops = state.parse_line(&text, &e.set).ok()?;
        let mut row: Vec<(Kind, String)> = Vec::new();
        let mut pos = 0;
        let push = |row: &mut Vec<(Kind, String)>, k: Kind, s: &str| {
            let s = s.trim_end_matches('\n');
            if s.is_empty() {
                return;
            }
            match row.last_mut() {
                Some((lk, ls)) if *lk == k => ls.push_str(s),
                _ => row.push((k, s.to_string())),
            }
        };
        for (at, op) in ops {
            if at > pos {
                push(&mut row, kind_of(&stack), &text[pos..at]);
                pos = at;
            }
            let _ = stack.apply(&op);
        }
        if pos < text.len() {
            push(&mut row, kind_of(&stack), &text[pos..]);
        }
        out.push(row);
    }
    if let Ok(mut c) = cache.lock() {
        if c.len() > 200 {
            c.clear();
        }
        c.insert(key, out.clone());
    }
    Some(out)
}

#[cfg(test)]
#[path = "highlight_tests.rs"]
mod tests;
