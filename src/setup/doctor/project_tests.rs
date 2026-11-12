use super::*;

fn repo(name: &str, dirs: &[&str], files: &[&str]) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-doctor-project-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join(".git")).unwrap();
    for x in dirs {
        std::fs::create_dir_all(d.join(x)).unwrap();
    }
    for f in files {
        std::fs::write(d.join(f), "x\n").unwrap();
    }
    d
}

// STRUCT-16: the template keeps only the paths that are in the repository, and it is valid.
#[test]
fn the_template_keeps_only_the_paths_that_are_here() {
    let root = repo("template", &["docs/adr", "docs/design", "src"], &["README.md"]);
    let text = structure_from_template(&root);
    let r = crate::docs::structure::check(&text, &root);
    let st = r.parsed.as_ref().unwrap();
    assert!(r.issues.is_empty(), "{:?}", r.issues);
    let paths: Vec<String> = st.entries.iter().map(|e| e.paths_label()).collect();
    assert_eq!(paths, vec!["README.md", "docs", "docs/design", "docs/adr", "src"]);
    assert!(text.contains(&format!("name = \"{}\"", root.file_name().unwrap().to_string_lossy())));
    assert!(!text.contains("{{"), "no placeholder is left");
    let _ = std::fs::remove_dir_all(&root);
}

// STRUCT-17: the zendo section goes in once, and doctor replaces it; the text around it stays.
#[test]
fn the_zendo_section_is_replaced_and_the_rest_stays() {
    let once = with_section("# Rules\n\nMine.\n", "new", SECTION_MARKS);
    assert_eq!(once, "# Rules\n\nMine.\n\n<!-- zendo -->\nnew\n<!-- /zendo -->\n");
    let again = with_section(&format!("{once}After.\n"), "newer", SECTION_MARKS);
    assert_eq!(again, "# Rules\n\nMine.\n\n<!-- zendo -->\nnewer\n<!-- /zendo -->\nAfter.\n");
    assert_eq!(with_section("", "x", SECTION_MARKS), "<!-- zendo -->\nx\n<!-- /zendo -->\n");
}

// STRUCT-16, STRUCT-17, STRUCT-18: doctor makes the file, writes the section for the agent, and
// ignores .zendo/ in git; a second run changes nothing.
#[test]
fn doctor_sets_up_the_repository_once() {
    let root = repo("setup", &["docs/notes"], &["README.md"]);
    std::fs::write(root.join(".gitignore"), "/target\n").unwrap();
    let paths = Paths { root: root.clone(), notes_dir: root.join("docs/notes"), repo_notes: root.join("docs/notes") };
    let mut o = Out::new();
    setup_project(&paths, "npx @agentclientprotocol/claude-agent-acp", &mut o);
    assert!(root.join(FILE).exists());
    let claude = std::fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    assert!(claude.contains("<!-- zendo -->") && claude.contains("project-structure.toml"), "{claude}");
    assert!(!root.join("AGENTS.md").exists());
    let ignore = std::fs::read_to_string(root.join(".gitignore")).unwrap();
    assert!(ignore.starts_with("/target\n") && ignore.contains("\n.zendo/\n"), "{ignore}");
    let before: Vec<String> = [FILE, "CLAUDE.md", ".gitignore"].iter().map(|f| std::fs::read_to_string(root.join(f)).unwrap()).collect();
    setup_project(&paths, "npx @agentclientprotocol/claude-agent-acp", &mut o);
    let after: Vec<String> = [FILE, "CLAUDE.md", ".gitignore"].iter().map(|f| std::fs::read_to_string(root.join(f)).unwrap()).collect();
    assert_eq!(before, after, "a second run changes nothing");
    let _ = std::fs::remove_dir_all(&root);
}
