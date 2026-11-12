//! Put the git commit, a "has uncommitted changes" flag, and the source folder into the binary.
//! `zendo --version`, the update check and `zendo upgrade` read them.

use std::path::Path;
use std::process::Command;

fn git(dir: &str, args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).current_dir(dir).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let Some(commit) = git(&dir, &["rev-parse", "HEAD"]) else { return };
    // Untracked files do not count: cargo's own checkout has an untracked `.cargo-ok`.
    let dirty = git(&dir, &["status", "--porcelain", "--untracked-files=no"]).is_some_and(|s| !s.is_empty());
    println!("cargo:rustc-env=ZENDO_COMMIT={commit}");
    println!("cargo:rustc-env=ZENDO_DIRTY={}", u8::from(dirty));
    println!("cargo:rustc-env=ZENDO_SRC={dir}");
    // Build again when HEAD moves, a commit or `git add` happens, or a source file changes.
    if let Some(gd) = git(&dir, &["rev-parse", "--absolute-git-dir"]) {
        for f in ["HEAD", "index", "packed-refs", "refs/heads"] {
            let p = Path::new(&gd).join(f);
            if p.exists() {
                println!("cargo:rerun-if-changed={}", p.display());
            }
        }
    }
    println!("cargo:rerun-if-changed=src");
}
