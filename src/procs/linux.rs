//! Linux: the process tree of zendo from `/proc` (ADR 0016). zendo reads only the root and the
//! processes under it, through the `children` file of each thread; it starts no program and does
//! not list the other processes of the computer.
#![cfg_attr(not(target_os = "linux"), allow(dead_code))]
use super::Proc;
use std::fs;
use std::path::Path;

/// The clock ticks per second of `/proc/<pid>/stat` (`USER_HZ`), 100 on Linux.
const TICKS: f64 = 100.0;

/// The process `root` and the processes under it, read from the `/proc` at `base`. `None` when the
/// kernel has no `children` files: then zendo finds no background process, and runs no `ps`.
pub fn tree(base: &Path, root: u32) -> Option<Vec<Proc>> {
    fs::metadata(base.join(format!("{root}/task/{root}/children"))).ok()?;
    let uptime = uptime(base)?;
    let mut out = Vec::new();
    let mut todo = vec![root];
    while let Some(pid) = todo.pop() {
        let Some(p) = process(base, pid, uptime) else { continue };
        todo.extend(children(base, pid));
        out.push(p);
    }
    Some(out)
}

/// True while the process `pid` runs: it is in `/proc`, and not a zombie (BG-10, ADR 0023).
pub fn leader_runs(base: &Path, pid: u32) -> bool {
    base.join(pid.to_string()).is_dir() && !zombie(base, pid)
}

/// True when the process `pid` is a zombie: it ended, and its parent did not wait for it yet.
fn zombie(base: &Path, pid: u32) -> bool {
    let stat = fs::read_to_string(base.join(format!("{pid}/stat"))).unwrap_or_default();
    stat.rsplit_once(')').is_some_and(|(_, rest)| rest.trim_start().starts_with('Z'))
}

/// The seconds since boot.
fn uptime(base: &Path) -> Option<f64> {
    fs::read_to_string(base.join("uptime")).ok()?.split_whitespace().next()?.parse().ok()
}

/// The children of each thread of `pid`; none when it ended.
fn children(base: &Path, pid: u32) -> Vec<u32> {
    let Ok(tasks) = fs::read_dir(base.join(format!("{pid}/task"))) else { return Vec::new() };
    tasks
        .flatten()
        .filter_map(|t| fs::read_to_string(t.path().join("children")).ok())
        .flat_map(|s| s.split_whitespace().filter_map(|c| c.parse().ok()).collect::<Vec<u32>>())
        .collect()
}

/// One process from its `stat` and `cmdline`. A zombie, a process that ended, and a process without
/// a command line (a kernel thread) do not count, as in `ps`.
fn process(base: &Path, pid: u32, uptime: f64) -> Option<Proc> {
    let stat = fs::read_to_string(base.join(format!("{pid}/stat"))).ok()?;
    // The name in the brackets can hold spaces and brackets: the fields start after the last `)`.
    let fields: Vec<&str> = stat.rsplit_once(')')?.1.split_whitespace().collect();
    if fields.first()?.starts_with('Z') {
        return None;
    }
    let (ppid, pgid) = (fields.get(1)?.parse().ok()?, fields.get(2)?.parse().ok()?);
    let start = fields.get(19)?.parse::<u64>().ok()? as f64 / TICKS;
    let cmdline = fs::read(base.join(format!("{pid}/cmdline"))).ok()?;
    let args = String::from_utf8_lossy(&cmdline).split('\0').filter(|a| !a.is_empty()).collect::<Vec<_>>().join(" ");
    // As `etime` of `ps`: the whole seconds of the exact age.
    let secs = (uptime - start).max(0.0) as u64;
    (!args.is_empty()).then_some(Proc { pid, ppid, pgid, secs, args })
}

#[cfg(test)]
#[path = "linux_tests.rs"]
mod tests;
