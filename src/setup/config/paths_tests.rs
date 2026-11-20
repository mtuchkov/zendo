use super::*;
use std::ffi::OsString;

// CMD-21: an empty or relative XDG folder counts as not set: it would name a folder in the current
// one. The environment of the process stays as it is: the test passes the values.
#[test]
fn an_empty_or_relative_xdg_folder_counts_as_not_set() {
    let home = Path::new("/home/u");
    let dir = |v: Option<&str>| xdg_dir(v.map(OsString::from), home, ".local/share");
    assert_eq!(dir(None), home.join(".local/share"));
    assert_eq!(dir(Some("")), home.join(".local/share"), "empty");
    assert_eq!(dir(Some("data")), home.join(".local/share"), "relative");
    assert_eq!(dir(Some("/srv/data")), PathBuf::from("/srv/data"), "absolute");
}
