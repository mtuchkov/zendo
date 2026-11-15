//! What `zendo init` and `zendo doctor --fix` set up in a repository: `project-structure.toml` from the zendo template,
//! the zendo section in the instructions for agents, of the project and of the user, and the zendo
//! lines in `.gitignore`. Each zendo part sits between its marks, so that doctor can replace it safely.
use super::*;
use crate::docs::structure::FILE;

/// The project structure of zendo, as a template for other repositories.
pub const TEMPLATE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/project-structure.template.toml"));
/// The section about the project structure in `CLAUDE.md` or `AGENTS.md`.
pub const AGENTS_SECTION: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/agents-section.md"));
/// The section for zendo sessions in the instructions of the user (SESSIONS-22).
pub const USER_SECTION: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/user-agents-section.md"));
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

/// One file that zendo keeps up to date: its label in the report, its new text, and what the text is.
pub struct Change {
    pub label: &'static str,
    pub path: PathBuf,
    pub text: String,
    pub what: String,
}

/// The change of `path` to `text`, when the file does not hold it yet.
fn change(label: &'static str, path: PathBuf, text: String, what: String) -> Option<Change> {
    (std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str())).then_some(Change { label, path, text, what })
}

/// What the repository needs to be up to date (STRUCT-16 to STRUCT-18): the structure file, the
/// section for agents, the lines in `.gitignore`. Nothing outside a git repository.
pub fn project_changes(root: &Path, agent: &str) -> Vec<Change> {
    if !root.join(".git").exists() {
        return Vec::new();
    }
    let made = !root.join(FILE).exists() && !root.join("project-structure.md").exists();
    let structure = made.then(|| change("structure", root.join(FILE), structure_from_template(root), format!("{FILE}, from the zendo template"))).flatten();
    let agents = if made || root.join(FILE).exists() { agents_changes(root, agent) } else { Vec::new() };
    structure.into_iter().chain(agents).chain(gitignore_change(root)).collect()
}

/// The zendo section in each file of instructions for agents; a new file for the agent when there is none.
fn agents_changes(root: &Path, agent: &str) -> Vec<Change> {
    let mut files: Vec<&str> = ["CLAUDE.md", "AGENTS.md"].into_iter().filter(|f| root.join(f).exists()).collect();
    if files.is_empty() {
        files.push(if config::kind_of(agent) == config::Kind::Codex { "AGENTS.md" } else { "CLAUDE.md" });
    }
    let section = |f: &str| {
        let old = std::fs::read_to_string(root.join(f)).unwrap_or_default();
        change("agents", root.join(f), with_section(&old, AGENTS_SECTION, SECTION_MARKS), format!("{f}: the zendo section tells agents to follow {FILE}"))
    };
    files.into_iter().filter_map(section).collect()
}

/// `.zendo/` in `.gitignore`, unless a line of the user ignores it already.
fn gitignore_change(root: &Path) -> Option<Change> {
    let path = root.join(".gitignore");
    let old = std::fs::read_to_string(&path).unwrap_or_default();
    let outside = with_section(&old, "", IGNORE_MARKS).replace(&format!("{}\n\n{}", IGNORE_MARKS.0, IGNORE_MARKS.1), "");
    if outside.lines().any(|l| matches!(l.trim(), ".zendo" | ".zendo/" | "/.zendo" | "/.zendo/")) {
        return None;
    }
    let text = with_section(&old, &format!("# The local state of zendo.\n{IGNORED}"), IGNORE_MARKS);
    change("gitignore", path, text, format!(".gitignore: git ignores {IGNORED}, the local state of zendo"))
}

/// The files of instructions of the user: `CLAUDE.md` of Claude Code and `AGENTS.md` of Codex.
pub(super) fn user_files() -> [PathBuf; 2] {
    let home = PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    let dir = |var: &str, name: &str| std::env::var_os(var).filter(|v| !v.is_empty()).map(PathBuf::from).unwrap_or_else(|| home.join(name));
    [dir("CLAUDE_CONFIG_DIR", ".claude").join("CLAUDE.md"), dir("CODEX_HOME", ".codex").join("AGENTS.md")]
}

/// The zendo section in each of `files`, the instructions of the user (SESSIONS-22). A file whose
/// folder is not there stays away: the user does not have that agent.
pub fn user_changes(files: &[PathBuf]) -> Vec<Change> {
    let section = |f: &PathBuf| {
        let old = std::fs::read_to_string(f).unwrap_or_default();
        let what = format!("{}: the zendo section tells a session that zendo runs to follow the MCP server `zendo`", f.display());
        change("agents", f.clone(), with_section(&old, USER_SECTION, SECTION_MARKS), what)
    };
    files.iter().filter(|f| f.parent().is_some_and(Path::is_dir)).filter_map(section).collect()
}

/// Write each change and report it (CMD-15, CMD-16).
pub(super) fn apply(changes: Vec<Change>, o: &mut Out) {
    for c in changes {
        match std::fs::write(&c.path, &c.text) {
            Ok(()) => o.ok(c.label, format!("wrote {}", c.what)),
            Err(e) => o.fail(c.label, format!("cannot write {}: {e}", c.path.display())),
        }
    }
}

/// `zendo init`: bring the repository to the version of zendo, and tell each change (CMD-16).
pub fn init(paths: &Paths, agent_flag: Option<String>) -> bool {
    let mut o = Out::new();
    o.head("zendo init");
    if !paths.root.join(".git").exists() {
        o.fail("repo", "this folder is not a git repository");
        o.hint("Run `zendo init` in a repository: it makes the files of zendo there.");
        return false;
    }
    let (agent, _) = config::resolve_agent(agent_flag, &UserConfig::load());
    let changes = project_changes(&paths.root, &agent);
    if changes.is_empty() {
        o.ok("repo", "the repository is up to date");
    }
    apply(changes, &mut o);
    println!();
    !o.failed
}

#[cfg(test)]
#[path = "project_tests.rs"]
mod tests;
