//! The processes of the computer, as `ps` lists them, and the background processes of the agent
//! among them (docs/zendo/background.md, ADR 0016). `ps` works the same on Linux and macOS.
use std::process::{Command, Stdio};

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

/// The processes of the computer now; none when `ps` does not run.
pub fn list() -> Vec<Proc> {
    let out = Command::new("ps").args(["-A", "-o", "pid=,ppid=,pgid=,stat=,etime=,args="]).stderr(Stdio::null()).output();
    out.map(|o| parse(&String::from_utf8_lossy(&o.stdout))).unwrap_or_default()
}

/// The lines of `ps -o pid=,ppid=,pgid=,stat=,etime=,args=`. A zombie runs no more: it does not
/// count.
pub fn parse(text: &str) -> Vec<Proc> {
    text.lines().filter_map(line).collect()
}

fn line(l: &str) -> Option<Proc> {
    let mut it = l.split_whitespace();
    let pid = it.next()?.parse().ok()?;
    let ppid = it.next()?.parse().ok()?;
    let pgid = it.next()?.parse().ok()?;
    if it.next()?.starts_with('Z') {
        return None;
    }
    let secs = etime(it.next()?)?;
    let args = it.collect::<Vec<_>>().join(" ");
    (!args.is_empty()).then_some(Proc { pid, ppid, pgid, secs, args })
}

/// The age of `ps`: `[[dd-]hh:]mm:ss`.
fn etime(s: &str) -> Option<u64> {
    let (days, rest) = s.split_once('-').map_or(Some((0, s)), |(d, r)| Some((d.parse::<u64>().ok()?, r)))?;
    let parts: Vec<u64> = rest.split(':').map(str::parse).collect::<Result<_, _>>().ok()?;
    let secs = parts.iter().fold(0, |a, p| a * 60 + p);
    (parts.len() <= 3).then_some(days * 86400 + secs)
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
/// direct child of `root`, and is not under another such process. The subtrees of `skip`, such as
/// the terminals of the user, do not count.
pub fn groups(procs: &[Proc], root: u32, skip: &[u32]) -> Vec<Group> {
    let leads: Vec<(u32, &Proc)> = under(procs, root, skip).into_iter().filter(|(top, p)| p.pid == p.pgid && p.pid != *top && !agent(&p.args)).collect();
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

/// Ask the group to end, or end it by force (BG-3, BG-9).
pub fn stop(pgid: u32, force: bool) {
    let signal = if force { "-KILL" } else { "-TERM" };
    let _ = Command::new("kill").args([signal, "--", &format!("-{pgid}")]).stdout(Stdio::null()).stderr(Stdio::null()).status();
}

#[cfg(test)]
#[path = "procs_tests.rs"]
mod tests;
