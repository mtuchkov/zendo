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
        "docs/.drafts/hidden.md",
        ".obsidian/docs/hidden.md",
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
    let mut t = Tree { view: View::Text, ..Default::default() };
    t.scan_wait(&root);
    t.find = Some("DES".into());
    let rows: Vec<String> = t.rows().iter().map(|r| r.name.clone()).collect();
    assert_eq!(rows, vec!["svc/orders/docs/design.md"]);
    t.find = Some("notes".into());
    let rows: Vec<String> = t.rows().iter().map(|r| r.name.clone()).collect();
    assert_eq!(rows, vec!["docs/notes/a.md", "docs/notes/b.md"], "a folder name matches too");
    t.find = Some("docs a".into());
    let rows: Vec<String> = t.rows().iter().map(|r| r.name.clone()).collect();
    assert_eq!(rows[0], "docs/notes/a.md", "a match in the file name comes first: {rows:?}");
    t.find = Some(Default::default());
    assert!(t.rows().iter().any(|r| r.kind != Kind::File), "an empty filter shows the tree");
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-1: the tree lists the markdown files of the repository, without the files that git
// ignores and the files in hidden folders.
#[test]
fn docs_filter_and_gitignore() {
    let root = repo("filter");
    let mut t = Tree::default();
    t.scan_wait(&root);
    let files: Vec<String> = t.files.iter().map(|p| p.display().to_string()).collect();
    assert_eq!(files, vec!["docs/adr/0001-x.md", "docs/notes/a.md", "docs/notes/b.md", "svc/orders/docs/design.md"]);
    t.view = View::Text;
    t.scan_wait(&root);
    assert!(t.files.iter().any(|p| p == Path::new("README.md")));
    assert!(!t.files.iter().any(|p| p.starts_with("build") || p.starts_with("node_modules")));
    assert!(!t.files.iter().any(|p| p.components().any(|c| c.as_os_str().to_string_lossy().starts_with('.'))), "no file of a hidden folder: {:?}", t.files);
}

// FILES-2: the structure view shows the main materials in the order of the file.
#[test]
fn structure_view_lists_main_materials_in_file_order() {
    let root = repo("structure");
    std::fs::write(
        root.join("project-structure.toml"),
        "[[entry]]\npath = \"svc/orders/docs\"\nlabel = \"Orders\"\nmain = true\n\n[[entry]]\npath = \"README.md\"\nmain = true\n\n[[entry]]\npath = \"docs/adr\"\nlabel = \"Decisions\"\nmain = true\n\n[[entry]]\npath = \"docs/notes\"\nabout = \"not main\"\n",
    )
    .unwrap();
    let mut t = Tree { structure: Some(Structure::load(&root).unwrap().unwrap()), view: View::Structure, ..Tree::default() };
    t.scan_wait(&root);
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
    assert_eq!(t.view, View::Text);
    t.next_view();
    assert_eq!(t.view, View::Notes, "FILES-4: notes after text");
    t.next_view();
    assert_eq!(t.view, View::Structure);
    for gone in ["all", "folders"] {
        assert_eq!(View::from_name(gone), None, "the {gone} view is gone (FILES-4)");
    }
}

// FILES-5, FILES-6, FILES-56: the folders start closed; a folder that holds only one folder is
// one row; reveal opens the folders above a file.
#[test]
fn rows_join_single_folders_and_collapse() {
    let root = repo("rows");
    let mut t = Tree::default();
    t.scan_wait(&root);
    let names = |t: &Tree| -> Vec<String> { t.rows().iter().map(|r| format!("{}{}", "  ".repeat(r.depth), r.name)).collect() };
    assert_eq!(names(&t), vec!["docs", "svc/orders/docs"], "every folder starts closed");
    t.reveal(Path::new("docs/notes/b.md"));
    assert_eq!(names(&t), vec!["docs", "  adr", "  notes", "    a.md", "    b.md", "svc/orders/docs"]);
    assert_eq!(t.rows()[t.sel].rel, Path::new("docs/notes/b.md"));
    t.toggle(Path::new("docs"));
    assert_eq!(names(&t), vec!["docs", "svc/orders/docs"]);
    t.toggle(Path::new("svc/orders/docs"));
    assert_eq!(names(&t), vec!["docs", "svc/orders/docs", "  design.md"]);
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-57: expand all opens every folder and main material; collapse all closes them.
#[test]
fn expand_all_and_collapse_all() {
    let root = repo("all-folders");
    std::fs::write(root.join("project-structure.toml"), "[[entry]]\npath = \"docs/adr\"\nlabel = \"Decisions\"\nmain = true\n").unwrap();
    let mut t = Tree::default();
    t.scan_wait(&root);
    t.expand_all();
    let names: Vec<String> = t.rows().iter().map(|r| format!("{}{}", "  ".repeat(r.depth), r.name)).collect();
    assert_eq!(names, vec!["docs", "  adr", "    0001-x.md", "  notes", "    a.md", "    b.md", "svc/orders/docs", "  design.md"]);
    t.sel = 5;
    t.collapse_all();
    assert_eq!(t.rows().len(), 2, "every folder closes");
    assert_eq!((t.sel, t.scroll), (0, 0), "the selection goes to the top");
    t.structure = Some(Structure::load(&root).unwrap().unwrap());
    t.view = View::Structure;
    t.scan_wait(&root);
    t.expand_all();
    let names: Vec<String> = t.rows().iter().map(|r| format!("{}{}", "  ".repeat(r.depth), r.name)).collect();
    assert_eq!(names, vec!["Decisions", "  0001-x.md"], "a main material opens too");
    t.collapse_all();
    assert_eq!(t.rows().len(), 1);
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-17: the text view shows every folder, also an empty one or one without a text file.
#[test]
fn the_text_view_shows_every_folder() {
    let root = repo("folders");
    std::fs::create_dir_all(root.join("docs/empty")).unwrap();
    let mut t = Tree { view: View::Text, ..Default::default() };
    t.scan_wait(&root);
    t.expand_all();
    let rows: Vec<String> = t.rows().iter().map(|r| r.rel.display().to_string()).collect();
    assert!(rows.iter().any(|r| r == "docs/empty"), "{rows:?}");
    t.view = View::Docs;
    t.scan_wait(&root);
    t.expand_all();
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
    t.scan_wait(&root);
    let files: Vec<String> = t.files.iter().map(|p| p.display().to_string()).collect();
    for f in ["README.md", "Makefile", "svc/orders/Cargo.toml", "svc/orders/src/main.rs"] {
        assert!(files.iter().any(|x| x == f), "{f}: {files:?}");
    }
    assert!(!files.iter().any(|x| x.ends_with("logo.png")), "no binary file: {files:?}");
    assert!(!files.iter().any(|x| x.starts_with("build")), "git ignores it: {files:?}");
    assert_eq!(t.filter_label(), "text files");
    let _ = std::fs::remove_dir_all(&root);
}

// STRUCT-15: one entry with more locations is one main material; the tree shows the files of all
// of them as one flat list.
#[test]
fn an_entry_with_more_paths_shows_its_files_as_one_flat_list() {
    let root = repo("group");
    std::fs::write(
        root.join(crate::docs::structure::FILE),
        "[[entry]]\npath = [\"docs/adr\", \"svc/*/docs\", \"README.md\"]\nlabel = \"Decisions\"\nmain = true\n",
    )
    .unwrap();
    let mut t = Tree { structure: Some(Structure::load(&root).unwrap().unwrap()), view: View::Structure, ..Tree::default() };
    t.scan_wait(&root);
    let names: Vec<String> = t.rows().iter().map(|r| r.name.clone()).collect();
    assert_eq!(names, vec!["Decisions"], "one row: {names:?}");
    t.reveal(Path::new("svc/orders/docs/design.md"));
    let names: Vec<String> = t.rows().iter().map(|r| format!("{}{}", "  ".repeat(r.depth), r.name)).collect();
    assert_eq!(names, vec!["Decisions", "  README.md", "  docs/adr/0001-x.md", "  svc/orders/docs/design.md"]);
    assert_eq!(t.rows()[t.sel].rel, Path::new("svc/orders/docs/design.md"));
    t.toggle(Path::new("docs/adr"));
    assert_eq!(t.rows().len(), 1, "the row closes the list");
    let _ = std::fs::remove_dir_all(&root);
}

/// A notes folder outside the repository, as in a notes repo, with one note.
fn notes_folder(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-tree-notes-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("2026-11-13-idea.md"), "# Idea\n").unwrap();
    std::fs::write(d.join(".zendo-location.toml"), "root = \"x\"\n").unwrap();
    d
}

// FILES-25: while the notes repo is on, the structure view, or the docs view without main
// materials, shows the notes of the notes repo as a `notes` folder at the top.
#[test]
fn the_tree_shows_the_notes_of_the_notes_repo() {
    let root = repo("notes-repo");
    let notes = notes_folder("structure");
    std::fs::write(root.join("project-structure.toml"), "[[entry]]\npath = \"docs/adr\"\nlabel = \"Decisions\"\nmain = true\n").unwrap();
    let mut t = Tree { structure: Some(Structure::load(&root).unwrap().unwrap()), view: View::Structure, notes: Some(notes.clone()), ..Tree::default() };
    t.scan_wait(&root);
    assert_eq!(t.rows().len(), 2, "the notes folder starts closed");
    t.toggle(&notes);
    let rows = t.rows();
    let names: Vec<String> = rows.iter().map(|r| format!("{}{}", "  ".repeat(r.depth), r.name)).collect();
    assert_eq!(names, vec!["notes", "  2026-11-13-idea.md", "Decisions"]);
    assert_eq!(rows[0].rel, notes);
    assert_eq!(root.join(&rows[1].rel), notes.join("2026-11-13-idea.md"), "opening the row opens the file in the notes repo");
    t.toggle(&notes);
    assert_eq!(t.rows().len(), 2, "the notes folder closes");
    t.reveal(&notes.join("2026-11-13-idea.md"));
    assert_eq!(t.rows()[t.sel].rel, notes.join("2026-11-13-idea.md"), "reveal opens the notes folder");
    t.next_view();
    t.scan_wait(&root);
    assert!(!t.rows().iter().any(|r| r.rel.starts_with(&notes)), "the docs view has main materials elsewhere");

    // Without main materials, the docs view shows the notes first.
    let _ = std::fs::remove_file(root.join("project-structure.toml"));
    let mut t = Tree { notes: Some(notes.clone()), ..Tree::default() };
    t.scan_wait(&root);
    t.toggle(&notes);
    let rows = t.rows();
    assert_eq!((rows[0].name.as_str(), rows[1].rel.clone()), ("notes", notes.join("2026-11-13-idea.md")));
    t.view = View::Text;
    assert!(!t.rows().iter().any(|r| r.rel.starts_with(&notes)), "only the structure or the docs view");
    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&notes);
}

// FILES-4, FILES-31: the two worktree views come after text, and only while the session has a
// worktree; the notes view comes last.
#[test]
fn the_worktree_views_follow_text_only_with_a_worktree() {
    let mut t = Tree { view: View::Text, ..Default::default() };
    t.next_view();
    assert_eq!(t.view, View::Notes, "no worktree: v skips both worktree views");
    t.view = View::Text;
    t.worktree = Some(PathBuf::from(".worktrees/1"));
    t.next_view();
    assert_eq!(t.view, View::Worktree);
    t.next_view();
    assert_eq!(t.view, View::WorktreeChanged);
    t.next_view();
    assert_eq!(t.view, View::Notes);
    t.next_view();
    assert_eq!(t.view, View::Docs);
    for v in [View::Worktree, View::WorktreeChanged, View::Notes] {
        assert_eq!(View::from_name(v.name()), Some(v), "the view is kept by its name (FILES-10)");
    }
}

// FILES-34: the top line shows the path in the worktree; the head of the note names the worktree.
#[test]
fn a_worktree_path_names_its_worktree() {
    assert_eq!(worktree::in_worktree(".worktrees/3/src/main.rs"), "src/main.rs");
    assert_eq!(worktree::in_worktree("docs/zendo/files/files.md"), "docs/zendo/files/files.md");
    assert_eq!(worktree::split(".worktrees/12/a/b.md"), Some((".worktrees/12", "a/b.md")));
    assert_eq!(worktree::split(".worktrees/12"), None);
}

// FILES-79, ADR 0009: the text view walks the files on a thread: a scan returns at once, and
// `poll` takes the files when the walk ends.
#[test]
fn the_text_view_walks_off_the_ui_thread() {
    let root = std::env::temp_dir().join(format!("zendo-tree-text-walk-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/a.rs"), "fn a() {}\n").unwrap();
    std::fs::write(root.join("b.bin"), [0u8, 1, 2]).unwrap();
    let mut t = Tree { view: View::Text, ..Default::default() };
    t.scan(&root);
    assert!(t.walking(), "the walk runs on its thread");
    assert!(t.files.is_empty(), "the scan does not wait for the walk");
    let end = Instant::now() + Duration::from_secs(10);
    while !t.poll() {
        assert!(Instant::now() < end, "timeout");
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(t.files, vec![PathBuf::from("src/a.rs")], "text files only");
    assert!(!t.walking());
    let _ = std::fs::remove_dir_all(&root);
}

// SESSIONS-23: agent session 0 has no worktree; agent session 1 has `.worktrees/1`.
#[test]
fn session_0_has_no_worktree() {
    let root = std::env::temp_dir().join(format!("zendo-tree-session-1-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for n in [1, 2] {
        std::fs::create_dir_all(root.join(format!(".worktrees/{n}"))).unwrap();
        std::fs::write(root.join(format!(".worktrees/{n}/.git")), "gitdir: x\n").unwrap();
    }
    assert_eq!(worktree::of(&root, 0), None, "session 0: the main checkout");
    assert_eq!(worktree::of(&root, 1), Some(PathBuf::from(".worktrees/1")));
    assert_eq!(worktree::of(&root, 2), Some(PathBuf::from(".worktrees/2")));
    let _ = std::fs::remove_dir_all(&root);
}
