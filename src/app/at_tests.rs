use super::*;

#[test]
fn distance_counts_edits() {
    assert_eq!(distance("", "abc"), 3);
    assert_eq!(distance("kitten", "sitting"), 3);
    assert_eq!(distance("spec", "spec"), 0);
    assert_eq!(distance("spce", "spec"), 2);
}

// CLAUDE-22: `@` suggests files by the start of the name, a part, or with a typo.
#[test]
fn files_match_by_start_part_and_typo() {
    let files: Vec<PathBuf> = ["docs/zendo/zendo.md", "docs/guides/templates/specification.md", "src/app/chat.rs", "README.md"].iter().map(PathBuf::from).collect();
    assert_eq!(rank(&files, "spec")[0], PathBuf::from("docs/guides/templates/specification.md"));
    assert_eq!(rank(&files, "chat")[0], PathBuf::from("src/app/chat.rs"));
    assert_eq!(rank(&files, "guides")[0], PathBuf::from("docs/guides/templates/specification.md"));
    assert_eq!(rank(&files, "zendp")[0], PathBuf::from("docs/zendo/zendo.md"), "one typo");
    assert!(rank(&files, "qqqqq").is_empty());
    assert_eq!(rank(&files, "").len(), 4);
}

// CLAUDE-25: templates are the files that project-structure.toml names, and those of `templates` folders.
#[test]
fn templates_are_named_ones_and_those_of_templates_folders() {
    let files: Vec<PathBuf> = ["docs/templates/adr.md", "docs/guides/templates/spec.md", "docs/specs/a.md", "templates.md"].iter().map(PathBuf::from).collect();
    let named = vec![PathBuf::from("docs/forms/spec.md"), PathBuf::from("docs/templates/adr.md")];
    let t = template_files(&named, &files);
    assert_eq!(t, vec![PathBuf::from("docs/forms/spec.md"), PathBuf::from("docs/guides/templates/spec.md"), PathBuf::from("docs/templates/adr.md")]);
}
