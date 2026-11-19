use super::*;
use std::io::Write;
use std::path::PathBuf;

/// A folder of its own for a test; an earlier run of the test leaves nothing.
fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-atomic-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// The names of the files in `d`.
fn names(d: &Path) -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(d).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
    v.sort();
    v
}

// FAIL-2: a write that stops after half of the new content leaves the file as it was, and no
// temporary file stays.
#[test]
fn a_failed_write_keeps_the_file() {
    let d = dir("half");
    let file = d.join("config.toml");
    std::fs::write(&file, "sound = true\nmusic = true\n").unwrap();
    let err = write_with(&file, |f| {
        f.write_all(b"sound = fa")?;
        Err(io::Error::other("the disk is full"))
    });
    assert!(err.is_err(), "the error comes back");
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "sound = true\nmusic = true\n");
    assert_eq!(names(&d), ["config.toml"], "no temporary file stays");
    let _ = std::fs::remove_dir_all(&d);
}

// FAIL-2: a write puts the whole new content in place, also for a new file, and leaves no
// temporary file.
#[test]
fn a_write_puts_the_whole_content_in_place() {
    let d = dir("whole");
    let file = d.join("note.md");
    write(&file, "# New\n").unwrap();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "# New\n");
    write(&file, "# Second\n\nmore\n").unwrap();
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "# Second\n\nmore\n");
    assert_eq!(names(&d), ["note.md"]);
    let _ = std::fs::remove_dir_all(&d);
}

// FAIL-2, FAIL-3: a config that is a link into a dotfiles repository stays a link; the file that
// the link names gets the content.
#[cfg(unix)]
#[test]
fn a_link_stays_a_link() {
    let d = dir("link");
    std::fs::create_dir_all(d.join("dotfiles")).unwrap();
    std::fs::write(d.join("dotfiles/config.toml"), "old\n").unwrap();
    std::os::unix::fs::symlink("dotfiles/config.toml", d.join("config.toml")).unwrap();
    write(&d.join("config.toml"), "new\n").unwrap();
    assert!(std::fs::symlink_metadata(d.join("config.toml")).unwrap().file_type().is_symlink(), "the link stays");
    assert_eq!(std::fs::read_to_string(d.join("dotfiles/config.toml")).unwrap(), "new\n");
    assert_eq!(names(&d.join("dotfiles")), ["config.toml"], "no temporary file beside the target");
    let _ = std::fs::remove_dir_all(&d);
}

// FAIL-3: the file keeps its permissions.
#[cfg(unix)]
#[test]
fn the_file_keeps_its_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let d = dir("mode");
    let file = d.join("secret.toml");
    std::fs::write(&file, "token = 1\n").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
    write(&file, "token = 2\n").unwrap();
    assert_eq!(std::fs::metadata(&file).unwrap().permissions().mode() & 0o777, 0o600);
    let _ = std::fs::remove_dir_all(&d);
}

// FAIL-2: a write to a folder that does not exist fails and makes nothing.
#[test]
fn a_write_to_a_missing_folder_fails() {
    let d = dir("missing");
    assert!(write(&d.join("no/such/file.md"), "x").is_err());
    assert_eq!(names(&d), Vec::<String>::new());
    let _ = std::fs::remove_dir_all(&d);
}
