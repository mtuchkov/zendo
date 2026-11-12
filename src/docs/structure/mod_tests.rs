use super::*;

const EXAMPLE: &str = r#"
[meta]
date = "2026-01-01"

[project]
name = "x"
claude = "Write in English."

[[entry]]
path = "docs/design"
label = "Design"
about = "Design notes."
claude = "Write each decision with its reason."
main = true

[[entry]]
path = "docs"
about = "Documentation."
claude = "Keep the documents short."

[[entry]]
path = "generated/*"
about = "Generated files."
claude = "Do not edit generated files."
"#;

#[test]
fn parses_and_finds_rules_from_general_to_specific() {
    let s: Structure = toml::from_str(EXAMPLE).unwrap();
    assert_eq!(s.main().map(|e| e.label()).collect::<Vec<_>>(), vec!["Design"]);
    let rules = s.rules_for(Path::new("docs/design/a.md"));
    let docs = rules.find("- docs:").unwrap();
    let design = rules.find("- docs/design:").unwrap();
    assert!(rules.starts_with("Project rules:\nWrite in English."));
    assert!(docs < design, "{rules}");
    assert_eq!(s.about(Path::new("docs/design/a.md")), Some("Design notes."));
    assert_eq!(s.about(Path::new("docs/guide.md")), Some("Documentation."));
}

#[test]
fn globs_cover_paths_inside() {
    let s: Structure = toml::from_str(EXAMPLE).unwrap();
    assert!(s.rules_for(Path::new("generated/api/index.md")).contains("Do not edit generated files."));
    assert!(!s.rules_for(Path::new("src/api/index.md")).contains("generated files"));
}

#[test]
fn checks_find_what_zendo_does_not_understand() {
    let root = std::env::temp_dir().join(format!("zendo-structure-check-{}", std::process::id()));
    std::fs::create_dir_all(root.join("docs/specs")).unwrap();
    let text = "extra = 1\n[project]\nowner = \"x\"\n[[entry]]\npath = \"docs/specs\"\nmain = true\n[[entry]]\npath = \"./docs\"\ndescription = \"d\"\n[[entry]]\npath = \"docs/specs\"\n[[entry]]\npath = \"nope\"\nmain = true\n[[entry]]\npath = \"out/*\"\nmain = true\n";
    let r = check(text, &root);
    let all: Vec<String> = r.issues.iter().map(|i| format!("{}: {}", i.at, i.what)).collect();
    let has = |s: &str| all.iter().any(|x| x.contains(s));
    assert!(has("top level: unknown key `extra`"), "{all:#?}");
    assert!(has("[project]: unknown key `owner`"));
    assert!(has("unknown key `description`; did you mean `about`?"));
    assert!(has("without `/`, `./`"));
    assert!(has("the same path"));
    assert!(has("entry 4 (nope): a main entry names a path that does not exist"));
    assert!(has("cannot have `*`"));
    assert!(check("not toml = = 1", &root).parsed.is_err());
}

#[test]
fn quick_fix_renames_keys_and_paths_and_keeps_comments() {
    let text = "# my file\n[[entry]]\n# the specs\npath = \"./docs/specs/\"\ndescription = \"Specs.\" # why\ninstructions = \"Keep it.\"\nmain = \"yes\"\n";
    let (fixed, changed) = quick_fix(text).unwrap();
    assert!(fixed.contains("# my file") && fixed.contains("# the specs") && fixed.contains("# why"), "{fixed}");
    let st: Structure = toml::from_str(&fixed).unwrap();
    let e = &st.entries[0];
    assert_eq!(e.path, "docs/specs/");
    assert_eq!(e.about.as_deref(), Some("Specs."));
    assert_eq!(e.claude.as_deref(), Some("Keep it."));
    assert!(e.main);
    assert_eq!(changed.len(), 4, "{changed:?}");
    assert!(quick_fix(&fixed).is_none());
}

#[test]
fn the_embedded_format_example_is_valid() {
    let toml_block = FORMAT.split("```toml").nth(1).unwrap().split("```").next().unwrap();
    let st: Structure = toml::from_str(toml_block).unwrap();
    assert_eq!(st.main().count(), 2);
    assert!(FIX_PROMPT.contains("{{FILE}}") && FIX_PROMPT.contains("{{FORMAT}}"));
}

#[test]
fn label_defaults_to_the_last_part() {
    let e = Entry { path: "docs/architecture/decisions/".into(), label: None, about: None, claude: None, main: true, words: None, template: None };
    assert_eq!(e.label(), "decisions");
}

// STRUCT-13: an entry can name the template of the new documents in its folder.
#[test]
fn a_folder_names_its_template() {
    let text = "[project]\nname = \"x\"\n[[entry]]\npath = \"docs\"\n[[entry]]\npath = \"docs/specs\"\ntemplate = \"docs/templates/spec.md\"\n[[entry]]\npath = \"docs/adr\"\ntemplate = \"docs/templates/adr.md\"\n";
    let s: Structure = toml::from_str(text).unwrap();
    assert_eq!(s.template_for(Path::new("docs/specs/new.md")), Some("docs/templates/spec.md"));
    assert_eq!(s.template_for(Path::new("docs/specs/deep/new.md")), Some("docs/templates/spec.md"));
    assert_eq!(s.template_for(Path::new("docs/other.md")), None);
    let all: Vec<(String, String)> = s.templates().map(|(t, at)| (t.to_string(), at.to_string())).collect();
    assert_eq!(all, vec![("docs/templates/spec.md".into(), "docs/specs".into()), ("docs/templates/adr.md".into(), "docs/adr".into())]);
}

// STRUCT-14: rules name an entry by its label, so two entries cannot have the same label.
#[test]
fn a_label_in_two_entries_is_an_issue() {
    let root = std::env::temp_dir();
    let text = "[[entry]]\npath = \"a\"\nlabel = \"Code reviews\"\n[[entry]]\npath = \"b\"\nlabel = \"code reviews\"\n[[entry]]\npath = \"c/*\"\n[[entry]]\npath = \"d/*\"\n";
    let all: Vec<String> = check(text, &root).issues.iter().map(|i| format!("{}: {}", i.at, i.what)).collect();
    assert_eq!(all, vec!["entry 2 (b): the label \"code reviews\" is in another entry".to_string()]);
}

// STRUCT-13: a template that is not there is a problem that `zendo doctor` shows.
#[test]
fn a_missing_template_is_an_issue() {
    let root = std::env::temp_dir().join(format!("zendo-structure-tmpl-{}", std::process::id()));
    std::fs::create_dir_all(root.join("docs/specs")).unwrap();
    let r = check("[project]\nname = \"x\"\n[[entry]]\npath = \"docs/specs\"\ntemplate = \"docs/templates/none.md\"\n", &root);
    assert!(r.issues.iter().any(|i| i.what.contains("template")), "{:?}", r.issues.iter().map(|i| &i.what).collect::<Vec<_>>());
    let _ = std::fs::remove_dir_all(&root);
}
