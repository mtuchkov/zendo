use super::*;

// FILES-13: F2 renames a file in its folder; a name without an extension keeps the old one.
#[test]
fn a_new_name_stays_in_the_folder_and_keeps_the_extension() {
    let old = Path::new("/r/docs/a.md");
    assert_eq!(rename_target(old, "b"), Ok(PathBuf::from("/r/docs/b.md")));
    assert_eq!(rename_target(old, " b.txt "), Ok(PathBuf::from("/r/docs/b.txt")));
    assert!(rename_target(old, "").is_err());
    assert!(rename_target(old, "x/b.md").is_err());
    assert!(rename_target(old, "..").is_err());
}
