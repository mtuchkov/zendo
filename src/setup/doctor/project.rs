//! What `zendo doctor` sets up in a repository: `project-structure.toml` from the zendo template,
//! the zendo section in the instructions for agents, and the zendo lines in `.gitignore`. Each
//! zendo part sits between its marks, so that doctor can replace it safely.
use super::*;
use crate::docs::structure::FILE;

/// The project structure of zendo, as a template for other repositories.
pub const TEMPLATE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/project-structure.template.toml"));
/// The section about the project structure in `CLAUDE.md` or `AGENTS.md`.
pub const AGENTS_SECTION: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/agents-section.md"));
pub const SECTION_MARKS: (&str, &str) = ("<!-- zendo -->", "<!-- /zendo -->");
pub const IGNORE_MARKS: (&str, &str) = ("# >>> zendo: kept by zendo doctor", "# <<< zendo");
const IGNORED: &str = ".zendo/";

/// The template for the repository at `root`: its name, today, and only the paths that are there.
pub fn structure_from_template(root: &Path) -> String {
    let name = root.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "project".into());
    let text = TEMPLATE.replace("{{NAME}}", &name).replace("{{DATE}}", &chrono::Local::now().format("%Y-%m-%d").to_string());
    let Ok(mut doc) = text.parse::<toml_edit::DocumentMut>() else { return text };
    if let Some(entries) = doc.get_mut("entry").and_then(|v| v.as_array_of_tables_mut()) {
        for t in entries.iter_mut() {
            let here: Vec<String> = paths_of(t).into_iter().filter(|p| root.join(p).exists()).collect();
            let v = match here.as_slice() {
                [one] => toml_edit::Value::from(one.as_str()),
                many => toml_edit::Value::Array(many.iter().map(String::as_str).collect()),
            };
            t.insert("path", toml_edit::Item::Value(v));
        }
        entries.retain(|t| !paths_of(t).is_empty());
    }
    doc.to_string()
}

fn paths_of(t: &toml_edit::Table) -> Vec<String> {
    match t.get("path").and_then(|i| i.as_value()) {
        Some(toml_edit::Value::String(s)) => vec![s.value().clone()],
        Some(toml_edit::Value::Array(a)) => a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect(),
        _ => Vec::new(),
    }
}

/// The text with the zendo part between `marks`: in place of the old part, or at the end.
pub fn with_section(text: &str, body: &str, (open, close): (&str, &str)) -> String {
    let block = format!("{open}\n{}\n{close}", body.trim_end());
    let at = text.find(open).and_then(|a| text[a..].find(close).map(|b| (a, a + b + close.len())));
    match at {
        Some((a, b)) => format!("{}{block}{}", &text[..a], &text[b..]),
        None if text.trim().is_empty() => format!("{block}\n"),
        None => format!("{}\n\n{block}\n", text.trim_end()),
    }
}

/// Write `text` to `path` when it changed. True when it changed.
fn put(path: &Path, text: &str) -> bool {
    std::fs::read_to_string(path).ok().as_deref() != Some(text) && std::fs::write(path, text).is_ok()
}

/// Set up the repository: the structure file, the section for agents, the lines in `.gitignore`.
pub(super) fn setup_project(paths: &Paths, agent: &str, o: &mut Out) {
    let root = &paths.root;
    if !root.join(".git").exists() {
        return;
    }
    if !root.join(FILE).exists() && !root.join("project-structure.md").exists() && put(&root.join(FILE), &structure_from_template(root)) {
        o.ok("structure", format!("made {FILE} from the zendo template"));
        o.hint(format!("{FILE} says what lives where. zendo shows its main entries as the file tree, and agents follow its rules. Change it as you need; `zendo doctor` checks it."));
    }
    if root.join(FILE).exists() {
        agents_section(root, agent, o);
    }
    gitignore(root, o);
}

/// The zendo section in each file of instructions for agents; a new one for the agent when there is none.
fn agents_section(root: &Path, agent: &str, o: &mut Out) {
    let mut files: Vec<&str> = ["CLAUDE.md", "AGENTS.md"].into_iter().filter(|f| root.join(f).exists()).collect();
    if files.is_empty() {
        files.push(if config::kind_of(agent) == config::Kind::Codex { "AGENTS.md" } else { "CLAUDE.md" });
    }
    for f in files {
        let old = std::fs::read_to_string(root.join(f)).unwrap_or_default();
        if put(&root.join(f), &with_section(&old, AGENTS_SECTION, SECTION_MARKS)) {
            o.ok("agents", format!("{f}: the zendo section tells agents to follow {FILE}"));
            o.hint(format!("zendo doctor keeps the text between {} and {}; write your own text outside it.", SECTION_MARKS.0, SECTION_MARKS.1));
        }
    }
}

/// `.zendo/` in `.gitignore`, unless a line of the user ignores it already.
fn gitignore(root: &Path, o: &mut Out) {
    let path = root.join(".gitignore");
    let old = std::fs::read_to_string(&path).unwrap_or_default();
    let outside = with_section(&old, "", IGNORE_MARKS).replace(&format!("{}\n\n{}", IGNORE_MARKS.0, IGNORE_MARKS.1), "");
    let theirs = outside.lines().any(|l| matches!(l.trim(), ".zendo" | ".zendo/" | "/.zendo" | "/.zendo/"));
    if !theirs && put(&path, &with_section(&old, &format!("# The local state of zendo.\n{IGNORED}"), IGNORE_MARKS)) {
        o.ok("gitignore", format!(".gitignore: git ignores {IGNORED}, the local state of zendo"));
    }
}

#[cfg(test)]
#[path = "project_tests.rs"]
mod tests;
