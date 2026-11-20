//! `zendo setup fonts` (docs/zendo/fonts.md), and what `zendo setup` says about the fonts at its
//! end (docs/zendo/fonts-tip.md).
use super::*;
use crate::setup::fonts::{self, Outcome, recipe};

/// Download and install the recommended fonts in the font folder of the user (FONT-7); then, in a
/// terminal without a working recipe, offer the session of the agent (FONT-22). False when a font
/// failed (FONT-10).
pub fn setup_fonts(env: &config::AgentEnv) -> bool {
    let mut o = Out::new();
    o.head("zendo setup fonts");
    checks::agent_env_warnings(env, &mut o);
    let dir = fonts::user_dir();
    println!("  {:<13}{}", "", o.paint("2", &format!("downloading into {}…", dir.display())));
    let mut installed = 0;
    let mut report = |name: &str, r: &Outcome| match r {
        Outcome::Installed => {
            installed += 1;
            o.ok("font", format!("{name}: installed"));
        }
        Outcome::Present => o.ok("font", format!("{name}: already on this machine")),
        Outcome::Failed(why) => o.fail("font", format!("{name}: {why}")),
    };
    fonts::install(fonts::FONTS, &mut fonts::Curl, &dir, &fonts::search_dirs(), &mut report);
    if installed > 0 && cfg!(target_os = "linux") {
        refresh_cache(&dir, &o);
    }
    use std::io::IsTerminal;
    with_session(&env.vars, |s| recipe_step(&o, &mut std::io::stdin().lock(), std::io::stdin().is_terminal(), s, &|name| find_program(&env.vars, name)));
    println!();
    println!("  {}", fonts::tip(theme_key().as_deref()));
    !o.failed
}

/// The font cache of the user knows the new files (FONT-12).
fn refresh_cache(dir: &Path, o: &Out) {
    let done = std::process::Command::new("fc-cache").arg("-f").arg(dir).stdin(std::process::Stdio::null()).output();
    match done {
        Ok(out) if out.status.success() => o.ok("font cache", "refreshed"),
        Ok(out) => o.warn("font cache", String::from_utf8_lossy(&out.stderr).trim()),
        Err(_) => o.warn("font cache", "fc-cache is not installed: the terminal may need a restart to see the fonts"),
    }
}

/// The key that opens the theme picker, from the keymap of the user (FONT-39).
fn theme_key() -> Option<String> {
    UserConfig::load().keymap().0.key(crate::keys::Pane::Read, "theme")
}

/// The end of `zendo setup`: the fonts, when their list changed since the last time (FONT-18,
/// FONT-19).
pub(super) fn fonts_note(o: &Out) {
    if let Some(text) = fonts::note(fonts::FONTS, &fonts::named_path(), theme_key().as_deref()) {
        let mut lines = text.lines();
        lines.next().into_iter().for_each(|l| o.hint(l));
        lines.for_each(|l| o.more(l));
    }
}

/// The interactive session of the agent that writes the recipe of the terminal font.
pub struct Session<'a> {
    pub program: &'a str,
    pub recipes: &'a Path,
    pub terminal: &'a str,
    pub cwd: &'a Path,
    /// The agent environment: the agent of the recipe uses the account of the profile (ENV-1).
    pub env: &'a BTreeMap<String, String>,
}

/// What the recipe step of `zendo setup fonts` did.
#[derive(Debug, PartialEq, Eq)]
pub enum Step {
    NotTerminal,
    Declined,
    NoAgent,
    Works,
    Broken(String),
}

/// The command of the agent for the session: `codex` for Codex, else `claude` (FONT-44).
pub fn agent_program(kind: config::Kind) -> &'static str {
    match kind {
        config::Kind::Codex => "codex",
        _ => "claude",
    }
}

impl Session<'_> {
    /// The recipe of the terminal: there and passing its check (FONT-31). `Ok(false)` without one.
    fn check(&self) -> Result<bool, String> {
        let recipes = recipe::Recipes::load(self.recipes)?;
        recipes.get(self.terminal).map(|r| r.check(&recipe::on_path).map(|()| true)).unwrap_or(Ok(false))
    }

    /// The first message: the terminal, the fonts, the recipe file and its format (FONT-27).
    fn first_message(&self) -> String {
        let list: String = fonts::FONTS.iter().map(|f| format!("- {} (family name `{}`)\n", f.name, f.family)).collect();
        include_str!("../../../resources/font-recipe-prompt.md").replace("{terminal}", self.terminal).replace("{recipes}", &self.recipes.display().to_string()).replace("{fonts}", list.trim_end())
    }

    /// The interactive session in this terminal, until the user ends it (FONT-22, FONT-25).
    fn start(&self, o: &Out, find: &dyn Fn(&str) -> Option<PathBuf>) -> Result<(), Step> {
        let Some(program) = find(self.program) else {
            o.warn(self.program, "is not on the PATH: install Claude Code or Codex, and run this command again");
            return Err(Step::NoAgent);
        };
        o.ok(self.program, "starts: end the session when the recipe is ready");
        let run = std::process::Command::new(program).envs(self.env).arg(self.first_message()).current_dir(self.cwd).status();
        run.map(drop).map_err(|e| Step::Broken(format!("{}: {e}", self.program)))
    }

    /// After the session: the recipe works, or why not (FONT-28).
    fn result(&self, o: &Out) -> Step {
        match self.check() {
            Ok(true) => {
                o.ok("recipe", format!("works for {}", self.terminal));
                Step::Works
            }
            Ok(false) => self.broken(o, format!("no recipe for {} in {}", self.terminal, self.recipes.display())),
            Err(e) => self.broken(o, e),
        }
    }

    fn broken(&self, o: &Out, why: String) -> Step {
        o.warn("recipe", &why);
        Step::Broken(why)
    }
}

/// The recipe step of `zendo setup fonts`: while the terminal has no working recipe, ask, and on
/// yes start the session of the agent (FONT-22); no question without a terminal (FONT-26).
pub(super) fn recipe_step(o: &Out, input: &mut dyn std::io::BufRead, tty: bool, s: &Session, find: &dyn Fn(&str) -> Option<PathBuf>) -> Step {
    if s.check() == Ok(true) {
        o.ok("recipe", format!("works for {}", s.terminal));
        return Step::Works;
    }
    if !tty {
        o.hint("To teach zendo the font of your terminal, run `zendo setup fonts` in the terminal.");
        return Step::NotTerminal;
    }
    print!("  May zendo start {} to find how {} sets its font? [y/N] ", s.program, s.terminal);
    let _ = std::io::Write::flush(&mut std::io::stdout());
    let mut answer = String::new();
    let _ = input.read_line(&mut answer);
    if !matches!(answer.trim(), "y" | "Y" | "yes") {
        return Step::Declined;
    }
    s.start(o, find).map_or_else(|step| step, |()| s.result(o))
}

/// `zendo doctor`: the recipe of the current terminal (FONT-24); a broken one names the repair, and
/// with `--fix --agent` the session repairs it (FONT-25, FONT-31). False while it is broken.
pub(super) fn recipe_doctor(o: &Out, s: &Session, repair: bool, find: &dyn Fn(&str) -> Option<PathBuf>) -> bool {
    match s.check() {
        Ok(true) => o.ok("recipe", format!("works for {}", s.terminal)),
        Ok(false) => {}
        Err(why) if !repair => {
            o.warn("recipe", &why);
            o.hint("To repair it with the agent: zendo doctor --fix --agent");
            return false;
        }
        Err(why) => {
            o.warn("recipe", &why);
            return s.start(o, find).is_ok() && s.result(o) == Step::Works;
        }
    }
    true
}

/// The session of this machine: the agent of the config, the recipe file, the current terminal.
pub(super) fn with_session<T>(env: &BTreeMap<String, String>, f: impl FnOnce(&Session) -> T) -> T {
    let kind = config::kind_of(&config::resolve_agent(None, &UserConfig::load()).0);
    let (recipes, terminal, cwd) = (recipe::path(), recipe::current_terminal(), config::config_dir());
    let _ = std::fs::create_dir_all(&cwd);
    f(&Session { program: agent_program(kind), recipes: &recipes, terminal: &terminal, cwd: &cwd, env })
}

/// A program on the `PATH` of the agent environment `env` (ENV-1).
pub(super) fn find_program(env: &BTreeMap<String, String>, name: &str) -> Option<PathBuf> {
    agent_var(env, "PATH").and_then(|p| std::env::split_paths(&p).map(|d| d.join(name)).find(|f| f.is_file()))
}

#[cfg(test)]
#[path = "fonts_tests.rs"]
mod tests;
