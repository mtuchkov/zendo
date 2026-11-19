//! macOS: the process tree of zendo from libproc (ADR 0016, ADR 0023). zendo reads only the root and
//! the processes under it: the children with `proc_listchildpids`, the parent, the group, the start
//! and the state with `proc_pidinfo`, and the command line with `sysctl` `KERN_PROCARGS2`. It starts
//! no program and does not list the other processes of the computer.
//!
//! Built for macOS, but not run on a Mac yet.
use super::Proc;
use std::ffi::c_void;

/// The most processes that one list gives.
const MAX_PIDS: usize = 4096;
/// The longest command line that zendo reads.
const MAX_ARGS: usize = 256 * 1024;

/// The process `root` and the processes under it.
pub fn tree(root: u32) -> Vec<Proc> {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let mut out = Vec::new();
    let mut todo = vec![root];
    while let Some(pid) = todo.pop() {
        let Some(p) = process(pid, now) else { continue };
        todo.extend(pids(|buf, size| unsafe { libc::proc_listchildpids(pid as libc::pid_t, buf, size) }));
        out.push(p);
    }
    out
}

/// True while a process of the group `pgid` runs that is not a zombie (BG-10). No signal goes out.
pub fn group_exists(pgid: u32) -> bool {
    let Some(id) = libc::pid_t::try_from(pgid).ok().filter(|id| *id > 0) else { return false };
    let members = pids(|buf, size| unsafe { libc::proc_listpgrppids(id, buf, size) });
    members.into_iter().any(|p| info(p).is_some_and(|i| i.pbi_status != libc::SZOMB))
}

/// The pids that `list` writes into a buffer. The buffer starts as zeros, and only the pids that are
/// not 0 count, so the answer does not depend on whether `list` gives the number of pids or of bytes.
fn pids(list: impl Fn(*mut c_void, libc::c_int) -> libc::c_int) -> Vec<u32> {
    let mut buf = vec![0 as libc::pid_t; MAX_PIDS];
    let size = libc::c_int::try_from(buf.len() * std::mem::size_of::<libc::pid_t>()).unwrap_or(libc::c_int::MAX);
    let n = list(buf.as_mut_ptr().cast(), size);
    let n = usize::try_from(n).unwrap_or(0).min(buf.len());
    buf[..n].iter().filter_map(|p| u32::try_from(*p).ok()).filter(|p| *p > 0).collect()
}

/// The BSD info of `pid`: its parent, group, start and state.
fn info(pid: u32) -> Option<libc::proc_bsdinfo> {
    let mut info = std::mem::MaybeUninit::<libc::proc_bsdinfo>::zeroed();
    let size = libc::c_int::try_from(std::mem::size_of::<libc::proc_bsdinfo>()).ok()?;
    let id = libc::c_int::try_from(pid).ok()?;
    let got = unsafe { libc::proc_pidinfo(id, libc::PROC_PIDTBSDINFO, 0, info.as_mut_ptr().cast(), size) };
    // SAFETY: the kernel filled the whole struct when it gave its size.
    (got == size).then(|| unsafe { info.assume_init() })
}

/// One process. A zombie, a process that ended, and a process without a command line do not count.
fn process(pid: u32, now: u64) -> Option<Proc> {
    let i = info(pid)?;
    if i.pbi_status == libc::SZOMB {
        return None;
    }
    let args = args(pid)?;
    let secs = now.saturating_sub(i.pbi_start_tvsec);
    (!args.is_empty()).then_some(Proc { pid, ppid: i.pbi_ppid, pgid: i.pbi_pgid, secs, args })
}

/// The command line of `pid` from `KERN_PROCARGS2`: the number of the arguments, the path of the
/// program, then the arguments, each ending with a 0.
fn args(pid: u32) -> Option<String> {
    let mut mib = [libc::CTL_KERN, libc::KERN_PROCARGS2, libc::c_int::try_from(pid).ok()?];
    let mut buf = vec![0u8; MAX_ARGS];
    let mut len = buf.len();
    let ok = unsafe { libc::sysctl(mib.as_mut_ptr(), 3, buf.as_mut_ptr().cast(), &mut len, std::ptr::null_mut(), 0) };
    if ok != 0 || len < 4 {
        return None;
    }
    let argc = usize::try_from(i32::from_ne_bytes(buf[..4].try_into().ok()?)).ok()?;
    let rest = &buf[4..len];
    // The path of the program, then the zeros of its padding.
    let start = rest.iter().position(|b| *b == 0)?;
    let rest = &rest[start..];
    let words = rest.split(|b| *b == 0).filter(|w| !w.is_empty()).take(argc);
    Some(words.map(String::from_utf8_lossy).collect::<Vec<_>>().join(" "))
}
