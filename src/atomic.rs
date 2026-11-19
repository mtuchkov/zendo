//! A write of user data that is complete or does not happen (FAIL-2): the new content goes to a
//! temporary file beside the file, and a rename puts it in place. A link stays a link, and the file
//! keeps its permissions.
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Write `contents` to `path`: the whole content, or nothing.
pub fn write(path: &Path, contents: impl AsRef<[u8]>) -> io::Result<()> {
    write_with(path, |f| f.write_all(contents.as_ref()))
}

/// Write to `path` what `fill` writes. When `fill` or a later step fails, `path` keeps the content
/// that it had, and the temporary file goes.
pub fn write_with(path: &Path, fill: impl FnOnce(&mut File) -> io::Result<()>) -> io::Result<()> {
    let target = target(path);
    let tmp = temp_beside(&target);
    let done = put(&target, &tmp, fill);
    if done.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    done
}

fn put(target: &Path, tmp: &Path, fill: impl FnOnce(&mut File) -> io::Result<()>) -> io::Result<()> {
    let mut f = OpenOptions::new().write(true).create_new(true).open(tmp)?;
    if let Ok(meta) = fs::metadata(target) {
        f.set_permissions(meta.permissions())?;
    }
    fill(&mut f)?;
    f.sync_all()?;
    drop(f);
    fs::rename(tmp, target)
}

/// The file that gets the content: for a link, the file that the link names, also when it does not
/// exist yet; else `path`.
fn target(path: &Path) -> PathBuf {
    let Ok(link) = fs::read_link(path) else { return path.to_path_buf() };
    let named = if link.is_absolute() { link } else { path.parent().unwrap_or(Path::new("")).join(link) };
    fs::canonicalize(&named).unwrap_or(named)
}

/// A new hidden name in the folder of `target`: the tree and the file list skip it.
fn temp_beside(target: &Path) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let name = target.file_name().map_or_else(Default::default, |n| n.to_string_lossy().into_owned());
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    target.with_file_name(format!(".{name}.zendo-{}-{n}.tmp", std::process::id()))
}

#[cfg(test)]
#[path = "atomic_tests.rs"]
mod tests;
