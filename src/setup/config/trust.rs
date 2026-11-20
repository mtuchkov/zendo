//! The trusted folders (docs/zendo/trust.md): zendo reads the `.envrc` of a repository and starts
//! an agent in it only after the user trusts its folder.

use super::agent_env::{checkout, expand, with_profile};
use super::{AgentEnv, UserConfig, select_profile};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

/// The terminal where zendo asks.
pub struct Terminal<'a> {
    pub input: &'a mut dyn BufRead,
    pub out: &'a mut dyn Write,
}

/// How a command of zendo treats the trust.
#[derive(Clone, Copy, Debug)]
pub struct Command {
    /// It checks the machine: without a terminal it goes on without the repository (TRUST-6), and
    /// it goes on without the profile (PROFILE-3).
    pub checks: bool,
    /// It can write the config file. `zendo doctor` without `--fix` writes no file (CMD-14), so
    /// its answer counts for the run (TRUST-5).
    pub saves: bool,
}

/// The agent environment of a run of the command `cmd` that the user starts in the folder `dir`
/// (ENV-2), after the question about the trust (TRUST-2). `files`: the config file, and the home
/// folder when the user has one.
pub fn run_env(cfg: &UserConfig, files: (&Path, Option<&Path>), dir: &Path, flag: Option<&str>, cmd: Command, terminal: Option<Terminal>) -> Result<AgentEnv, String> {
    // The location: the main checkout of the repository, and a worktree outside it; outside a
    // repository, the folder where zendo starts (TRUST-7).
    let real = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let dir = real(dir);
    // The home folder as it is on the disk: it can be a link. Without one, `~` is the start folder.
    let home = files.1.map(real);
    let files = (files.0, home.as_deref());
    let home = home.clone().unwrap_or_else(|| dir.clone());
    let home = home.as_path();
    let (folders, at) = match checkout(&dir) {
        Some((root, main)) => (repo_folders(&root, &main), main),
        None => (vec![dir.clone()], dir),
    };
    // PROFILE-3: a command that checks the machine goes on without a profile that is not there.
    let (profile, unknown) = match select_profile(cfg, flag, &at, home) {
        Ok(p) => (p, None),
        Err(e) if cmd.checks => (None, Some(format!("{e}; zendo goes on without a profile"))),
        Err(e) => return Err(e),
    };
    let missing = untrusted(cfg, &folders, home);
    let refused = match terminal {
        _ if missing.is_empty() => None,
        Some(t) => settle(files, &missing, t, cmd.saves).map(|()| None)?,
        None if cmd.checks => Some(how_to_trust(&missing)),
        None => return Err(how_to_trust(&missing)),
    };
    let mut env = with_profile(cfg, profile, Some(at.as_path()).filter(|_| refused.is_none()), home, &|k| std::env::var(k).ok());
    env.problems.extend(unknown);
    // TRUST-3: an entry that is no full path trusts nothing.
    let odd = cfg.trusted_dirs.iter().filter(|d| !expand(d, home).is_absolute());
    env.problems.extend(odd.map(|d| format!("trusted_dirs: `{d}` is no full path, so it trusts no folder")));
    Ok(AgentEnv { untrusted: refused, ..env })
}

/// Ask about `missing` and save the answer `y`; an error for any other answer (TRUST-4). `files`:
/// the config file and the home folder. zendo does not save the answer of a command that does not
/// save (CMD-14), and not for the home folder or a folder above it, which hold the other folders
/// of the user (TRUST-5); then the answer counts for this run (TRUST-8).
fn settle(files: (&Path, Option<&Path>), missing: &[PathBuf], t: Terminal, saves: bool) -> Result<(), String> {
    if !ask(missing, t.input, t.out) {
        return Err(format!("{}, so zendo did not start", not_trusted(missing)));
    }
    let (wide, kept): (Vec<PathBuf>, Vec<PathBuf>) = missing.iter().cloned().partition(|f| files.1.is_some_and(|home| home.starts_with(f)));
    let saved = if saves { save(files.0, &kept) } else { Err("zendo doctor writes no file".to_string()) };
    let said = saved.err().map(|e| format!("{e}: the answer counts for this run only"));
    let wide = wide.iter().map(|f| format!("{} holds your home folder: zendo does not save its trust, and asks again at the next start", f.display()));
    for line in said.into_iter().chain(wide) {
        let _ = writeln!(t.out, "zendo: {line}");
    }
    Ok(())
}

/// The folders of a repository that need the trust: the root of the main checkout `main`, and the
/// root `root` of the worktree when it is outside the main checkout.
pub fn repo_folders(root: &Path, main: &Path) -> Vec<PathBuf> {
    let outside = (!root.starts_with(main)).then(|| root.to_path_buf());
    std::iter::once(main.to_path_buf()).chain(outside).collect()
}

/// The folders of `folders` that are not trusted: neither the folder nor a folder above it is in
/// `trusted_dirs` (TRUST-3).
pub fn untrusted(cfg: &UserConfig, folders: &[PathBuf], home: &Path) -> Vec<PathBuf> {
    let real = |d: &String| {
        let d = expand(d, home);
        std::fs::canonicalize(&d).unwrap_or(d)
    };
    // An entry that is no full path, as an empty text or `.`, trusts nothing.
    let trusted: Vec<PathBuf> = cfg.trusted_dirs.iter().filter(|d| expand(d, home).is_absolute()).map(real).collect();
    folders.iter().filter(|f| !trusted.iter().any(|t| f.starts_with(t))).cloned().collect()
}

fn names(folders: &[PathBuf]) -> String {
    folders.iter().map(|f| f.display().to_string()).collect::<Vec<_>>().join(" and ")
}

/// `A is not trusted`, `A and B are not trusted`.
fn not_trusted(folders: &[PathBuf]) -> String {
    format!("{} {} not trusted", names(folders), if folders.len() == 1 { "is" } else { "are" })
}

/// The question about `folders` (TRUST-2): true only for the answer `y`.
fn ask(folders: &[PathBuf], input: &mut dyn BufRead, out: &mut dyn Write) -> bool {
    let (this, it) = if folders.len() == 1 { ("this folder", "it") } else { ("these folders", "them") };
    let text = format!(
        "zendo: do you trust the files in {}?\n  zendo gives the variables of its .envrc to the agents and the terminals, and starts agents in\n  {this}. Both can run code of the folder.\n  Trust {this} and the folders in {it}? [y/N] ",
        names(folders)
    );
    let _ = out.write_all(text.as_bytes()).and_then(|()| out.flush());
    let mut answer = String::new();
    input.read_line(&mut answer).is_ok() && matches!(answer.trim().to_lowercase().as_str(), "y" | "yes")
}

/// Add `folders` to `trusted_dirs` in the config file `path`; all other content stays (TRUST-5).
fn save(path: &Path, folders: &[PathBuf]) -> Result<(), String> {
    if folders.is_empty() {
        return Ok(());
    }
    UserConfig::update(path, |c| {
        let new: Vec<String> = folders.iter().map(|f| f.display().to_string()).filter(|f| !c.trusted_dirs.contains(f)).collect();
        c.trusted_dirs.extend(new);
    })
}

/// What zendo says where it cannot ask: the folder, and the two ways to trust it (TRUST-6).
fn how_to_trust(folders: &[PathBuf]) -> String {
    format!(
        "{}: zendo does not read the .envrc and starts no agent there. To trust a folder, start zendo there in a terminal and answer the question, or add the folder to trusted_dirs of {}",
        not_trusted(folders),
        super::config_path().display()
    )
}

#[cfg(test)]
#[path = "trust_tests.rs"]
mod tests;
