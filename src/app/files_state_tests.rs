//! The files without an app (ADR 0008).
use super::*;
use crate::keys::Pane;
use crate::docs::structure;
use crate::docs::tree::View;
use crate::app::{FilterKey, TreeAct};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// The action of a key in `pane` of the default keymap.
fn action_of(pane: Pane, k: KeyEvent) -> Option<&'static str> {
    crate::keys::Keymap::defaults().local(pane, &k)
}

/// A repository with two notes in `docs/`; an earlier run of the test leaves nothing.
fn repo(name: &str) -> (PathBuf, Files) {
    let root = std::env::temp_dir().join(format!("zendo-files-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("docs")).unwrap();
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::write(root.join("docs/a.md"), "# A\n\nfirst\n").unwrap();
    std::fs::write(root.join("docs/b.md"), "# B\n\nsecond\n").unwrap();
    std::fs::write(root.join("code.rs"), "fn main() {}\n").unwrap();
    let files = Files::start(Store::new(root.clone(), root.join("docs"))).unwrap();
    (root, files)
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn typ(f: &mut Files, root: &Path, text: &str) -> Option<Effect> {
    text.chars().for_each(|c| assert_eq!(f.key_prompt(key(KeyCode::Char(c)), action_of(Pane::Question, key(KeyCode::Char(c))), root), None));
    answer(f, root, KeyCode::Enter)
}

/// A key of the question, and its effect: an `Io` job runs here, as the app runs it on a thread,
/// and the files follow its answer as the app makes them follow (IO-1).
fn answer(f: &mut Files, root: &Path, code: KeyCode) -> Option<Effect> {
    let e = f.key_prompt(key(code), action_of(Pane::Question, key(code)), root);
    let e = match e {
        Some(Effect::Io(job)) => job.run(),
        e => e,
    };
    match &e {
        Some(Effect::Deleted(k)) => f.gone(&root.join(k)),
        Some(Effect::Moved { old, target, .. }) => f.moved(old, target),
        Some(Effect::MadeFile(_)) => f.reload(),
        _ => {}
    }
    e
}

fn select(f: &mut Files, root: &Path, rel: &str) {
    f.set_view(View::Text);
    f.scan_wait(root, None, None);
    f.reveal(Path::new(rel));
    assert_eq!(f.tree_file().as_deref(), Some(Path::new(rel)), "the row of {rel}");
}

/// Wait at most 5 s for the worker that reads the notes, and take its answer (IO-6).
fn settled(f: &mut Files) {
    let end = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while f.reloading() && std::time::Instant::now() < end {
        std::thread::sleep(std::time::Duration::from_millis(2));
        f.poll_reload();
    }
}

// ADR 0008: the files start with the notes of the store and an open document; a file outside the
// notes is read when it opens, and goes when it is gone from the disk.
#[test]
fn a_file_outside_the_notes_is_read_when_it_opens() {
    let (root, mut f) = repo("open");
    assert_eq!(f.notes().len(), 2);
    assert!(f.current().is_some() && f.note().is_some());
    f.open(root.join("code.rs"));
    settled(&mut f);
    assert_eq!((f.notes().len(), f.rel(f.current().unwrap())), (3, "code.rs".to_string()));
    assert_eq!(f.note().map(|n| n.body.as_str()), Some("fn main() {}\n"));
    std::fs::remove_file(root.join("code.rs")).unwrap();
    f.open(root.join("docs/a.md"));
    f.reload();
    settled(&mut f);
    assert_eq!(f.notes().len(), 2);
    f.show_note(1);
    assert_eq!(f.note_index(), Some(1));
    let _ = std::fs::remove_dir_all(&root);
}

// ADR 0008, FILES-13: a rename moves the file on disk; the open document follows it, and the app
// gets the old and the new path for the thread and the session.
#[test]
fn a_rename_moves_the_file_and_returns_the_paths() {
    let (root, mut f) = repo("rename");
    f.open(root.join("docs/a.md"));
    assert_eq!(f.ask_file(None, 'r'), Effect::flash("select a file first"));
    assert_eq!(f.ask_file(Some("docs/a.md".into()), 'r'), None);
    assert_eq!(f.prompt(), Some(&FilePrompt::Rename("docs/a.md".into(), "a.md".into())));
    (0..4).for_each(|_| drop(f.key_prompt(key(KeyCode::Backspace), action_of(Pane::Question, key(KeyCode::Backspace)), &root)));
    let moved = typ(&mut f, &root, "c");
    assert_eq!(moved, Some(Effect::Moved { old: root.join("docs/a.md"), target: root.join("docs/c.md"), done: "renamed" }));
    assert!(root.join("docs/c.md").exists() && !root.join("docs/a.md").exists() && f.prompt().is_none());
    assert_eq!(f.current_path(), Some(root.join("docs/c.md").as_path()));
    f.ask(FilePrompt::Rename("docs/c.md".into(), "b.md".into()));
    assert_eq!(answer(&mut f, &root, KeyCode::Enter), Effect::flash("a file with that name exists"));
    f.ask(FilePrompt::Move("docs/c.md".into(), "../out".into()));
    assert_eq!(f.key_prompt(key(KeyCode::Enter), action_of(Pane::Question, key(KeyCode::Enter)), &root), Effect::flash("give a folder inside the repository"));
    let _ = std::fs::remove_dir_all(&root);
}

// ADR 0008: `y` deletes the file, and the app gets its key; another key drops the question.
#[test]
fn a_delete_removes_the_file_and_returns_its_key() {
    let (root, mut f) = repo("delete");
    f.open(root.join("docs/a.md"));
    f.ask(FilePrompt::Delete("docs/a.md".into()));
    assert_eq!(f.key_prompt(key(KeyCode::Char('n')), action_of(Pane::Question, key(KeyCode::Char('n'))), &root), None);
    assert!(f.prompt().is_none() && root.join("docs/a.md").exists());
    f.ask(FilePrompt::Delete("docs/a.md".into()));
    assert_eq!(answer(&mut f, &root, KeyCode::Char('y')), Some(Effect::Deleted("docs/a.md".into())));
    assert!(!root.join("docs/a.md").exists());
    settled(&mut f);
    assert_eq!((f.notes().len(), f.current()), (1, None), "no document is open");
    let _ = std::fs::remove_dir_all(&root);
}

// ADR 0008: a new file gets the title of its name; a new folder shows in the folders view.
#[test]
fn a_new_file_and_a_new_folder_come_back_as_effects() {
    let (root, mut f) = repo("new");
    select(&mut f, &root, "docs/a.md");
    f.ask_new(false);
    assert_eq!(f.prompt(), Some(&FilePrompt::NewFile("docs/".into())), "the folder of the selection");
    assert_eq!(typ(&mut f, &root, "audit-log"), Some(Effect::MadeFile(root.join("docs/audit-log.md"))));
    assert_eq!(std::fs::read_to_string(root.join("docs/audit-log.md")).unwrap(), "# Audit log\n\n");
    settled(&mut f);
    assert_eq!(f.notes().len(), 3);
    f.ask(FilePrompt::NewFile("../x".into()));
    assert_eq!(f.key_prompt(key(KeyCode::Enter), action_of(Pane::Question, key(KeyCode::Enter)), &root), Effect::flash("give a path inside the repository"));
    f.ask(FilePrompt::NewFolder("docs/de".into()));
    assert_eq!(answer(&mut f, &root, KeyCode::Enter), Some(Effect::MadeFolder(root.join("docs/de"))));
    assert!(root.join("docs/de").is_dir() && f.tree().view == View::Text, "the text view shows a new folder (FILES-17)");
    f.ask(FilePrompt::NewFile("docs/d".into()));
    // The worker reads the folders; the tab completes when it knows them (IO-1).
    let end = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while f.complete_folder(&root, "docs/d").is_none() && std::time::Instant::now() < end {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(f.key_prompt(key(KeyCode::Tab), action_of(Pane::Question, key(KeyCode::Tab)), &root), None);
    assert_eq!(f.prompt(), Some(&FilePrompt::NewFile("docs/de/".into())), "tab completes the folder");
    let _ = std::fs::remove_dir_all(&root);
}

// ADR 0008: the keys that only move in the tree are done in the files; the others are not.
#[test]
fn the_tree_moves_and_filters_without_the_app() {
    let (root, mut f) = repo("tree");
    select(&mut f, &root, "docs/a.md");
    let at = f.tree().sel;
    assert!(f.key_tree("down") && f.tree().sel == at + 1);
    assert!(f.key_tree("up") && f.key_tree("top") && f.tree().sel == 0);
    assert!(!f.key_tree("delete") && !f.key_tree("open"), "the app does these");
    assert!(f.key_tree("search") && f.tree().find.as_deref() == Some(""));
    "b.md".chars().for_each(|c| assert!(matches!(f.key_filter(key(KeyCode::Char(c)), action_of(Pane::List, key(KeyCode::Char(c)))), FilterKey::Done)));
    assert_eq!(f.tree().find.as_deref(), Some("b.md"));
    assert!(matches!(f.key_filter(key(KeyCode::Enter), action_of(Pane::List, key(KeyCode::Enter))), FilterKey::Open(rel) if rel == Path::new("docs/b.md")));
    assert!(f.tree().find.is_none());
    select(&mut f, &root, "docs/b.md");
    assert_eq!(f.activate(), tree_keys::Picked::File("docs/b.md".into()));
    let _ = std::fs::remove_dir_all(&root);
}

// ADR 0008, FILES-37: in a worktree view, while the agent works, a popup asks before the action.
#[test]
fn a_file_action_waits_for_the_popup_while_the_agent_works() {
    let (root, mut f) = repo("confirm");
    let act = || TreeAct::File(Some("docs/a.md".into()), 'd');
    assert_eq!(f.tree_act(act(), true), None);
    assert!(f.prompt().is_some() && f.confirm_text(0).is_none(), "not a worktree view: no popup");
    f.key_prompt(key(KeyCode::Esc), action_of(Pane::Question, key(KeyCode::Esc)), &root);
    f.set_view(View::Worktree);
    assert_eq!(f.tree_act(act(), true), None);
    assert!(f.prompt().is_none() && f.confirm_text(1).is_some_and(|t| t.contains("session 2") && t.contains("delete")));
    assert_eq!(f.key_confirm(key(KeyCode::Char('n'))), None);
    assert!(f.prompt().is_none() && f.confirm_text(1).is_none());
    f.tree_act(act(), true);
    f.key_confirm(key(KeyCode::Char('y')));
    assert_eq!(f.prompt(), Some(&FilePrompt::Delete("docs/a.md".into())));
    let _ = std::fs::remove_dir_all(&root);
}

/// The structure that a look read again, and whether the notes changed.
type Seen = (Option<super::reload::StructureRead>, bool);

/// Look at the disk now, on the worker, and wait for the answer (IO-1).
fn look(f: &mut Files, root: &Path) -> Seen {
    f.look_again();
    f.look_at_disk(root);
    let end = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Some(seen) = f.poll_look() {
            return seen;
        }
        assert!(std::time::Instant::now() < end, "no answer of the look");
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}

// ADR 0008, IO-1: the files look at the disk at most once a second, on a worker, and tell a change
// after the first look.
#[test]
fn a_change_on_disk_shows_after_the_first_look() {
    let (root, mut f) = repo("disk");
    f.look_at_disk(&root);
    assert!(!f.disk_busy(), "the first second has no look");
    assert!(!look(&mut f, &root).1, "the first look only takes the state of the folder");
    assert!(!look(&mut f, &root).1);
    std::fs::write(root.join("docs/new.md"), "# New\n").unwrap();
    assert!(look(&mut f, &root).1);
    let _ = std::fs::remove_dir_all(&root);
}

// ADR 0008: a structure file that does not parse leaves its reason, and a message for the user.
// FAIL-1, FAIL-6: a structure that does not parse leaves zendo working and names the fix.
#[test]
fn a_structure_that_does_not_parse_leaves_its_reason() {
    let (root, mut f) = repo("structure");
    assert_eq!(f.load_structure(&root, "", false), None);
    assert!(f.structure().is_none() && f.structure_error().is_none());
    assert!(f.tree().view == View::Docs);
    std::fs::write(root.join(structure::FILE), "[[entry\n").unwrap();
    let (loaded, issues) = look(&mut f, &root).0.expect("the file changed");
    assert!(matches!(f.apply_structure(loaded, issues, "", false), Some(Effect::Flash(t)) if t.contains("does not parse")));
    assert!(f.structure_error().is_some());
    assert!(look(&mut f, &root).0.is_none(), "no change since");
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-2: the tree shows the structure view first when the repository has main materials, and
// the docs view first when it does not; a saved structure view without main materials gives docs.
#[test]
fn the_first_view_is_the_structure_when_there_are_main_materials() {
    let (root, mut f) = repo("first-view");
    std::fs::write(root.join(structure::FILE), "[[entry]]\npath = \"docs\"\nlabel = \"Docs\"\n").unwrap();
    f.load_structure(&root, "", false);
    assert!(f.tree().view == View::Docs, "no main materials: docs first");
    std::fs::write(root.join(structure::FILE), "[[entry]]\npath = \"docs\"\nlabel = \"Docs\"\nmain = true\n").unwrap();
    f.load_structure(&root, "", false);
    assert!(f.tree().view == View::Structure, "main materials: the structure first");
    std::fs::remove_file(root.join(structure::FILE)).unwrap();
    f.load_structure(&root, "structure", false);
    assert!(f.tree().view == View::Docs, "a saved structure view without main materials gives docs");
    let _ = std::fs::remove_dir_all(&root);
}

// ADR 0008: the app changes the tree only through the methods of the files.
#[test]
fn the_tree_changes_through_the_files() {
    let (root, mut f) = repo("tree-methods");
    f.set_view(View::Text);
    assert!(f.tree_stale(), "a tree that was never filled");
    f.scan_wait(&root, None, None);
    assert!(!f.tree_stale() && !f.tree().rows().is_empty());
    f.scan_again();
    assert!(f.tree_stale(), "the next tick fills it again");

    f.reveal(Path::new("docs/b.md"));
    assert_eq!(f.tree_file().as_deref(), Some(Path::new("docs/b.md")));
    f.select_row(0);
    f.scroll_tree(false, 50);
    let rows = f.tree().rows().len();
    f.fit_tree(rows, 3);
    assert!(f.tree().sel >= f.tree().scroll && f.tree().sel < f.tree().scroll + 3, "the draw keeps the selection in view");
    f.scroll_tree(true, 50);
    assert_eq!(f.tree().scroll, 0);

    f.mark_changed([PathBuf::from("docs/a.md")].into());
    assert!(f.tree().changed.contains(Path::new("docs/a.md")));

    f.select_row(1);
    let name = f.next_view();
    assert_eq!((name, f.tree().sel, f.tree().scroll), (f.tree().view.name(), 0, 0), "the next view starts from its first row");
    assert!(!f.filtering());
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-58: while the tree shows, it opens the folders above the open document and selects it:
// when the document changes, and when the app asks again (the pane opens, the session changes, a
// file opens). A document of the worktree shows at its place in the repository.
#[test]
fn the_tree_follows_the_open_document() {
    let (root, mut f) = repo("follow");
    f.set_view(View::Docs);
    f.open(root.join("docs/b.md"));
    f.follow_current();
    assert_eq!(f.tree().rows().len(), 0, "a tree that was never filled does nothing");
    f.scan_wait(&root, None, None);
    f.follow_current();
    assert_eq!(f.tree_file().as_deref(), Some(Path::new("docs/b.md")));
    f.collapse_all();
    f.follow_current();
    assert_eq!(f.tree().rows().len(), 1, "the same document: the tree stays as the user left it");
    f.follow_again();
    f.follow_current();
    assert_eq!(f.tree_file().as_deref(), Some(Path::new("docs/b.md")), "the app asks again");
    std::fs::create_dir_all(root.join(".worktrees/1/docs")).unwrap();
    std::fs::write(root.join(".worktrees/1/docs/a.md"), "# A\n").unwrap();
    f.open(root.join(".worktrees/1/docs/a.md"));
    f.follow_current();
    assert_eq!(f.tree_file().as_deref(), Some(Path::new("docs/a.md")), "the worktree file at its place in the repository");
    let _ = std::fs::remove_dir_all(&root);
}
