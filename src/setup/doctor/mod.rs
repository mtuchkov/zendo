//! `zendo setup` installs the Claude adapter once. `zendo doctor` checks every step
//! from Node to a real ACP handshake and says what to do when a step fails.

mod agent_checks;
mod checks;
mod fix;
mod fonts;
mod notes;
mod preflight;
mod project;
mod setup;
mod upgrade;

use agent_checks::*;
pub use checks::*;
pub use fix::*;
pub use fonts::setup_fonts;
pub use notes::*;
pub use preflight::*;
pub use project::*;
pub use setup::*;
pub use upgrade::*;

use crate::agent::{self, Cmd, Event};
use crate::setup::config::{self, SYSTEM_CA, Source, UserConfig};
use std::collections::BTreeMap;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::process::Command;

pub struct Paths {
    pub root: PathBuf,
    /// Where the notes of this run are: the folder in the repo, or the folder in the notes repo.
    pub notes_dir: PathBuf,
    /// The notes folder in the repo (`docs/notes`), also when the notes repo is on.
    pub repo_notes: PathBuf,
    /// The agent environment of this run, with its profile (ENV-1, PROFILE-7).
    pub env: config::AgentEnv,
}

/// The variable `name` of the agents: from the agent environment `env`, else from zendo (ENV-2).
pub(super) fn agent_var(env: &BTreeMap<String, String>, name: &str) -> Option<std::ffi::OsString> {
    env.get(name).map(Into::into).or_else(|| std::env::var_os(name)).filter(|v| !v.is_empty())
}

struct Out {
    color: bool,
    failed: bool,
    /// Labels of the failed checks, so that a fixed structure file no longer counts.
    failures: Vec<String>,
}

impl Out {
    fn new() -> Self {
        Self { color: std::io::stdout().is_terminal(), failed: false, failures: Vec::new() }
    }
    fn paint(&self, code: &str, s: &str) -> String {
        if self.color { format!("\x1b[{code}m{s}\x1b[0m") } else { s.to_string() }
    }
    fn line(&self, mark: &str, code: &str, label: &str, detail: &str) {
        println!("  {} {:<11}{}", self.paint(code, mark), label, detail);
    }
    fn ok(&self, label: &str, detail: impl AsRef<str>) {
        self.line("✓", "32", label, detail.as_ref());
    }
    fn warn(&self, label: &str, detail: impl AsRef<str>) {
        self.line("!", "33", label, detail.as_ref());
    }
    fn fail(&mut self, label: &str, detail: impl AsRef<str>) {
        self.failed = true;
        self.failures.push(label.to_string());
        self.line("✗", "31", label, detail.as_ref());
    }
    fn more(&self, s: impl AsRef<str>) {
        for l in s.as_ref().lines() {
            println!("  {:<13}{}", "", self.paint("2", l));
        }
    }
    fn hint(&self, s: impl AsRef<str>) {
        println!("  {:<13}{} {}", "", self.paint("36", "→"), s.as_ref());
    }
    fn head(&self, s: &str) {
        println!("\n{}", self.paint("1", s));
    }
}

/// Run a program; `None` when it is not installed.
async fn run(cmd: &str, args: &[&str], env: &BTreeMap<String, String>, secs: u64) -> Option<(bool, String)> {
    let mut c = Command::new(cmd);
    c.args(args).envs(env).stdin(std::process::Stdio::null()).kill_on_drop(true);
    match tokio::time::timeout(Duration::from_secs(secs), c.output()).await {
        Err(_) => Some((false, format!("timed out after {secs} s"))),
        Ok(Err(_)) => None,
        Ok(Ok(o)) => {
            let mut text = String::from_utf8_lossy(&o.stdout).into_owned();
            text.push_str(&String::from_utf8_lossy(&o.stderr));
            Some((o.status.success(), text.trim().to_string()))
        }
    }
}

/// The agent's exit reason, short: the stderr lines follow it anyway.
fn short(text: &str) -> String {
    let t = text.trim_start_matches("agent stopped: ");
    let t = t.split("\\n").next().unwrap_or(t).lines().next().unwrap_or(t);
    let t = if t.contains("Process exited") { "the process exited" } else { t };
    if t.chars().count() > 90 { format!("{}…", t.chars().take(89).collect::<String>()) } else { t.to_string() }
}

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or("").trim()
}

/// The Claude adapter, by package name (a path may contain "claude" for other reasons).
const CODEX_LOGIN: &str = "Run `codex login` (the Codex CLI: npm install -g @openai/codex), or set OPENAI_API_KEY, then start zendo again.";

/// Whether Codex has a login: an API key, or the auth file that `codex login` writes.
pub fn codex_logged_in(env: &BTreeMap<String, String>) -> bool {
    if agent_var(env, "OPENAI_API_KEY").is_some() || agent_var(env, "CODEX_API_KEY").is_some() {
        return true;
    }
    let home = agent_var(env, "CODEX_HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".codex"));
    home.join("auth.json").exists()
}

pub fn is_claude_agent(cmd: &str) -> bool {
    config::kind_of(cmd) == config::Kind::Claude
}

pub fn is_tls_error(s: &str) -> bool {
    ["UNABLE_TO_VERIFY", "SELF_SIGNED_CERT", "DEPTH_ZERO_SELF_SIGNED", "unable to verify the first certificate", "unable to get local issuer", "CERT_HAS_EXPIRED", "ERR_TLS_CERT"]
        .iter()
        .any(|p| s.contains(p))
}

/// npm's own error lines (code, reason, log file), or the last lines when it printed none.
fn npm_errors(s: &str) -> String {
    let errs: Vec<&str> = s.lines().filter(|l| l.contains("npm error") || l.contains("npm ERR")).collect();
    let mut keep: Vec<&str> = errs.iter().filter(|l| !l.contains("A complete log")).take(4).copied().collect();
    keep.extend(errs.iter().filter(|l| l.contains("A complete log")).take(1));
    if keep.is_empty() {
        keep = s.lines().filter(|l| !l.trim().is_empty()).collect::<Vec<_>>().into_iter().rev().take(4).collect();
        keep.reverse();
    }
    if keep.is_empty() { "(npm printed nothing)".into() } else { keep.join("\n") }
}

/// The variables of the agents for a start now: the agent environment, with the system CA when the
/// config has it by now. `zendo setup` saves it in the run that finds it (ENV-5).
pub(super) fn agent_vars(cfg: &UserConfig, paths: &Paths) -> BTreeMap<String, String> {
    if cfg.system_ca == Some(true) { with_system_ca(&paths.env.vars) } else { paths.env.vars.clone() }
}

/// The API key of Anthropic is in the agent environment (ENV-1).
pub(super) fn has_api_key(env: &BTreeMap<String, String>) -> bool {
    agent_var(env, "ANTHROPIC_API_KEY").is_some()
}

/// `env` with the option of the system CA of Node.js (ENV-5).
fn with_system_ca(env: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut e = env.clone();
    config::add_system_ca(&mut e, std::env::var("NODE_OPTIONS").ok());
    e
}

fn tls_hints(o: &Out, system_ca: bool) {
    if !system_ca {
        o.hint("Your Node cannot use the system certificate store (needs Node 22.15+ or 23.8+). Update Node, or:");
    } else {
        o.hint("The system certificate store does not have the CA either. Point Node at the CA file:");
    }
    o.more("export NODE_EXTRA_CA_CERTS=/path/to/company-ca.pem   # add to ~/.zshrc\nthen run `zendo setup` again");
}

// ----- setup ------------------------------------------------------------------

// ----- notes ------------------------------------------------------------------

pub const MOVE_PROMPT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/move-notes.md"));
pub const CONSOLIDATE_PROMPT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/consolidate-notes.md"));

// ----- doctor -----------------------------------------------------------------

// ----- preflight --------------------------------------------------------------

// ----- fix project-structure.toml ---------------------------------------------

// ----- upgrade ----------------------------------------------------------------

pub const SSH_REPO: &str = "ssh://git@github.com/mtuchkov/zendo.git";
pub const HTTPS_REPO: &str = "https://github.com/mtuchkov/zendo.git";

/// The update check asks GitHub at most once in this time (CMD-20, ADR 0023).
const UPDATE_EVERY: Duration = Duration::from_secs(24 * 3600);

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
