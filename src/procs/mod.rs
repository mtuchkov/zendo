//! The processes of zendo and the background processes of the agent among them
//! (docs/zendo/background.md, ADR 0016, ADR 0023). The tree of zendo comes from `/proc` on Linux and
//! from libproc on macOS. zendo starts no program for it, lists no other process of the computer,
//! and sends a signal only to stop a group.
use rustix::process::{Pid, Signal};

mod linux;
#[cfg(target_os = "macos")]
mod macos;

/// One process: its id, its parent, its group, its age in seconds, and its command line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proc {
    pub pid: u32,
    pub ppid: u32,
    pub pgid: u32,
    pub secs: u64,
    pub args: String,
}

/// A background process: a process group of its own under the root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub pgid: u32,
    /// The direct child of the root that the group runs under: the agent or a Claude Code session.
    pub under: u32,
    pub secs: u64,
    pub label: String,
}

/// The processes of zendo now: only its own tree, so that zendo starts no program each second and
/// does not list the other processes of the computer. A Linux kernel without the `children` files
/// gives none.
#[cfg(target_os = "linux")]
pub fn list() -> Vec<Proc> {
    linux::tree(std::path::Path::new("/proc"), std::process::id()).unwrap_or_default()
}

/// The processes of zendo now, from libproc (ADR 0023).
#[cfg(target_os = "macos")]
pub fn list() -> Vec<Proc> {
    macos::tree(std::process::id())
}

/// No process tree on other systems: no background processes.
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn list() -> Vec<Proc> {
    Vec::new()
}

/// The processes under `pid`, deep first, with the direct child of `pid` that each one is under.
fn under<'a>(procs: &'a [Proc], pid: u32, skip: &[u32]) -> Vec<(u32, &'a Proc)> {
    let mut out = Vec::new();
    let mut todo: Vec<(u32, u32)> = procs.iter().filter(|p| p.ppid == pid && !skip.contains(&p.pid)).map(|p| (p.pid, p.pid)).collect();
    while let Some((top, at)) = todo.pop() {
        let Some(p) = procs.iter().find(|p| p.pid == at) else { continue };
        out.push((top, p));
        todo.extend(procs.iter().filter(|c| c.ppid == at && !skip.contains(&c.pid)).map(|c| (top, c.pid)));
    }
    out
}

/// The background processes under `root`: each process that leads a group of its own, is not a
/// direct child of `root`, runs a command, and is not under another such process. The subtrees of
/// `skip`, such as the terminals of the user, do not count.
pub fn groups(procs: &[Proc], root: u32, skip: &[u32]) -> Vec<Group> {
    let leads: Vec<(u32, &Proc)> = under(procs, root, skip).into_iter().filter(|(top, p)| p.pid == p.pgid && p.pid != *top && !agent(&p.args) && command(procs, p)).collect();
    let pids: Vec<u32> = leads.iter().map(|(_, p)| p.pid).collect();
    let mut out: Vec<Group> = leads
        .iter()
        .filter(|(top, p)| !above(procs, p.ppid, *top).any(|a| pids.contains(&a)))
        .map(|(top, p)| Group { pgid: p.pgid, under: *top, secs: p.secs, label: label(p, procs) })
        .collect();
    out.sort_by_key(|g| g.pgid);
    out
}

/// The process `pid` and its parents, up to `top`.
fn above(procs: &[Proc], pid: u32, top: u32) -> impl Iterator<Item = u32> + '_ {
    std::iter::successors(Some(pid), move |at| procs.iter().find(|p| p.pid == *at).map(|p| p.ppid).filter(|_| *at != top)).take(64)
}

/// What a group runs: the command of the first child of a shell, the `eval` or `-c` of a shell
/// without a child, or the command line of the process.
pub fn label(leader: &Proc, procs: &[Proc]) -> String {
    let host = leader.args.contains("bg-pty-host");
    if !is_shell(&leader.args) && !host {
        return leader.args.clone();
    }
    if let Some(c) = procs.iter().filter(|c| c.ppid == leader.pid).min_by_key(|c| std::cmp::Reverse(c.secs)) {
        return c.args.clone();
    }
    let a = &leader.args;
    if let Some(e) = a.find("eval '").map(|i| &a[i + 6..]) {
        return e.split('\'').next().unwrap_or(e).to_string();
    }
    a.split_once(" -c ").map_or_else(|| a.clone(), |(_, c)| c.trim_start_matches("-l ").to_string())
}

/// True when the group of `leader` runs a command of the agent: a shell or the background terminal
/// of Claude Code runs in it. An MCP server, or a helper that it starts, runs no shell (BG-5).
fn command(procs: &[Proc], leader: &Proc) -> bool {
    procs.iter().filter(|p| p.pgid == leader.pgid).any(|p| is_shell(&p.args) || p.args.contains("bg-pty-host"))
}

/// A command line as the background line shows it: each path only by its file (BG-1).
pub fn short(args: &str) -> String {
    args.split(' ').map(|w| if w.starts_with('/') { w.rsplit('/').next().unwrap_or(w) } else { w }).collect::<Vec<_>>().join(" ")
}

/// The processes of an agent itself: the adapters and Claude Code in the mode of an SDK.
fn agent(args: &str) -> bool {
    ["stream-json", "claude-agent-acp", "codex-acp", "zendo-mock-agent"].iter().any(|a| args.contains(a))
}

fn is_shell(args: &str) -> bool {
    let bin = args.split_whitespace().next().unwrap_or("");
    matches!(bin.rsplit('/').next(), Some("sh" | "bash" | "zsh" | "dash" | "fish"))
}

/// True while a process of the group runs, in the tree of zendo or not (BG-10).
pub fn alive(procs: &[Proc], pgid: u32) -> bool {
    procs.iter().any(|p| p.pgid == pgid)
}

/// The group of zendo itself.
fn own_group() -> Option<u32> {
    u32::try_from(Pid::as_raw(Some(rustix::process::getpgrp()))).ok()
}

/// True while the group `pgid` runs, also after it left the tree of zendo (BG-10): its leader runs.
/// zendo reads it from `/proc/<pgid>` or libproc, and sends no signal (ADR 0023).
#[cfg(target_os = "linux")]
pub fn group_exists(pgid: u32) -> bool {
    pgid > 0 && linux::leader_runs(std::path::Path::new("/proc"), pgid)
}

#[cfg(target_os = "macos")]
pub fn group_exists(pgid: u32) -> bool {
    macos::group_exists(pgid)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn group_exists(_pgid: u32) -> bool {
    false
}

/// True when zendo may signal the group `pgid`: not 0 (the group of the caller), not 1 (every
/// process), and not the group of zendo itself, which must be known (BG-3).
pub fn may_stop(pgid: u32, own: Option<u32>) -> bool {
    pgid > 1 && own.is_some_and(|o| o != pgid)
}

/// Ask the group to end, or end it by force (BG-3, BG-9). False when `may_stop` refuses the group:
/// no signal goes out.
pub fn stop(pgid: u32, force: bool) -> bool {
    if !may_stop(pgid, own_group()) {
        return false;
    }
    let Some(pid) = i32::try_from(pgid).ok().and_then(Pid::from_raw) else { return false };
    let _ = rustix::process::kill_process_group(pid, if force { Signal::KILL } else { Signal::TERM });
    true
}

#[cfg(test)]
#[path = "procs_tests.rs"]
mod tests;
