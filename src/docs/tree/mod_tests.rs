use super::*;

fn repo(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-tree-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    for f in [
        "README.md",
        "docs/notes/a.md",
        "docs/notes/b.md",
        "docs/adr/0001-x.md",
        "svc/orders/docs/design.md",
        "svc/orders/src/main.rs",
        "build/docs/generated.md",
        "node_modules/pkg/docs/readme.md",
    ] {
        let p = d.join(f);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, "# x\n").unwrap();
    }
    std::fs::write(d.join(".gitignore"), "build/\nnode_modules/\n").unwrap();
    d
}

// FILES-11: `/` in the file tree filters the files by name.
#[test]
fn find_filters_files_by_name() {
    let root = repo("find");
    let mut t = Tree { view: View::All, ..Default::default() };
    t.scan(&root);
    t.find = Some("DES".into());
    let rows: Vec<String> = t.rows().iter().map(|r| r.name.clone()).collect();
    assert_eq!(rows, vec!["svc/orders/docs/design.md"]);
    t.find = Some("notes".into());
    let rows: Vec<String> = t.rows().iter().map(|r| r.name.clone()).collect();
    assert_eq!(rows, vec!["docs/notes/a.md", "docs/notes/b.md"], "a folder name matches too");
    t.find = Some("docs a".into());
    let rows: Vec<String> = t.rows().iter().map(|r| r.name.clone()).collect();
    assert_eq!(rows[0], "docs/notes/a.md", "a match in the file name comes first: {rows:?}");
    t.find = Some(String::new());
    assert!(t.rows().iter().any(|r| r.kind != Kind::File), "an empty filter shows the tree");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn docs_filter_and_gitignore() {
    let root = repo("filter");
    let mut t = Tree::default();
    t.scan(&root);
    let files: Vec<String> = t.files.iter().map(|p| p.display().to_string()).collect();
    assert_eq!(files, vec!["docs/adr/0001-x.md", "docs/notes/a.md", "docs/notes/b.md", "svc/orders/docs/design.md"]);
    t.view = View::All;
    t.scan(&root);
    assert!(t.files.iter().any(|p| p == Path::new("README.md")));
    assert!(!t.files.iter().any(|p| p.starts_with("build") || p.starts_with("node_modules")));
}

#[test]
fn structure_view_lists_main_materials_in_file_order() {
    let root = repo("structure");
    std::fs::write(
        root.join("project-structure.toml"),
        "[[entry]]\npath = \"svc/orders/docs\"\nlabel = \"Orders\"\nmain = true\n\n[[entry]]\npath = \"README.md\"\nmain = true\n\n[[entry]]\npath = \"docs/adr\"\nlabel = \"Decisions\"\nmain = true\n\n[[entry]]\npath = \"docs/notes\"\nabout = \"not main\"\n",
    )
    .unwrap();
    let mut t = Tree { structure: Some(Structure::load(&root).unwrap().unwrap()), view: View::Structure, ..Tree::default() };
    t.scan(&root);
    let names: Vec<String> = t.rows().iter().map(|r| format!("{}{}", "  ".repeat(r.depth), r.name)).collect();
    assert_eq!(names, vec!["Orders", "README.md", "Decisions"]);
    t.reveal(Path::new("docs/adr/0001-x.md"));
    let names: Vec<String> = t.rows().iter().map(|r| format!("{}{}", "  ".repeat(r.depth), r.name)).collect();
    assert_eq!(names, vec!["Orders", "README.md", "Decisions", "  0001-x.md"]);
    assert_eq!(t.rows()[t.sel].rel, Path::new("docs/adr/0001-x.md"));
    t.toggle(Path::new("docs/adr"));
    assert_eq!(t.rows().len(), 3);
    t.next_view();
    assert_eq!(t.view, View::Docs);
    t.next_view();
    assert_eq!(t.view, View::All);
    t.next_view();
    assert_eq!(t.view, View::Text);
    t.next_view();
    assert_eq!(t.view, View::Folders);
    t.next_view();
    assert_eq!(t.view, View::Structure);
}

#[test]
fn rows_join_single_folders_and_collapse() {
    let root = repo("rows");
    let mut t = Tree::default();
    t.scan(&root);
    let names: Vec<String> = t.rows().iter().map(|r| format!("{}{}", "  ".repeat(r.depth), r.name)).collect();
    assert_eq!(names, vec!["docs", "  adr", "    0001-x.md", "  notes", "    a.md", "    b.md", "svc/orders/docs", "  design.md"]);
    t.toggle(Path::new("docs"));
    let names: Vec<String> = t.rows().iter().map(|r| r.name.clone()).collect();
    assert_eq!(names, vec!["docs", "svc/orders/docs", "design.md"]);
    t.reveal(Path::new("docs/notes/b.md"));
    assert_eq!(t.rows()[t.sel].rel, Path::new("docs/notes/b.md"));
}

// FILES-17: the folders view shows every folder, also an empty one.
#[test]
fn the_folders_view_shows_empty_folders() {
    let root = repo("folders");
    std::fs::create_dir_all(root.join("docs/empty")).unwrap();
    let mut t = Tree { view: View::Folders, ..Default::default() };
    t.scan(&root);
    let rows: Vec<String> = t.rows().iter().map(|r| r.rel.display().to_string()).collect();
    assert!(rows.iter().any(|r| r == "docs/empty"), "{rows:?}");
    assert!(rows.iter().any(|r| r == "svc/orders/src"), "a folder without .md: {rows:?}");
    t.view = View::Docs;
    t.scan(&root);
    assert!(!t.rows().iter().any(|r| r.rel.ends_with("empty")));
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-20: the text view shows every text file: code and config too, and no binary file.
#[test]
fn the_text_view_shows_every_text_file() {
    let root = repo("text");
    std::fs::write(root.join("svc/orders/Cargo.toml"), "[package]\nname = \"orders\"\n").unwrap();
    std::fs::write(root.join("svc/orders/logo.png"), [0x89u8, b'P', b'N', b'G', 0, 0, 1]).unwrap();
    std::fs::write(root.join("Makefile"), "all:\n\techo hi\n").unwrap();
    let mut t = Tree { view: View::Text, ..Default::default() };
    t.scan(&root);
    let files: Vec<String> = t.files.iter().map(|p| p.display().to_string()).collect();
    for f in ["README.md", "Makefile", "svc/orders/Cargo.toml", "svc/orders/src/main.rs"] {
        assert!(files.iter().any(|x| x == f), "{f}: {files:?}");
    }
    assert!(!files.iter().any(|x| x.ends_with("logo.png")), "no binary file: {files:?}");
    assert!(!files.iter().any(|x| x.starts_with("build")), "git ignores it: {files:?}");
    assert_eq!(t.filter_label(), "text files");
    let _ = std::fs::remove_dir_all(&root);
}
