//! Find modes (docs/zendo/find-modes.md): Notes, Docs, Code and All; `tab` and `shift-tab`; the mode
//! row; the global `/`.

mod common;
use common::*;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::Path;
use zendo::app::{App, FindMode, Focus, Mode};

const FILES: [(&str, &str); 4] = [
    ("docs/find.md", "# Find\n\nThe find panel.\n"),
    ("src/find.rs", "// find\nfn main() {}\n"),
    ("docs/notes/2026-11-17-find-idea.md", "# Find idea\n\nAn idea.\n"),
    ("docs/other.md", "# Other\n\nWe find things here.\n"),
];

fn alt(a: &mut App, c: char) {
    a.on_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::ALT));
}

fn listed(a: &App, root: &Path) -> Vec<String> {
    a.files.find().items.iter().map(|i| i.path.strip_prefix(root).unwrap().display().to_string()).collect()
}

fn query(a: &mut App, q: &str) {
    typ(a, q);
    a.settle_find();
}

// FIND-21: `tab` goes round Notes, Docs, Code, All with the same query; each mode lists its own
// kind of files.
#[tokio::test]
async fn tab_switches_the_mode_and_keeps_the_query() {
    let root = repo("find-modes-tab", &FILES);
    let mut a = app(&root);
    alt(&mut a, '/');
    assert_eq!(a.files.find().mode, FindMode::Docs, "FIND-27");
    query(&mut a, "find");
    assert_eq!(listed(&a, &root), vec!["docs/find.md", "docs/other.md"], "Docs: by path, then by text; no note, no code");
    key(&mut a, KeyCode::Tab);
    assert_eq!((a.files.find().mode, listed(&a, &root)), (FindMode::Code, vec!["src/find.rs".to_string()]), "Code: by path only");
    key(&mut a, KeyCode::Tab);
    let all = listed(&a, &root);
    assert_eq!(a.files.find().mode, FindMode::All);
    assert_eq!(all.len(), 4, "{all:?}");
    assert_eq!(all[3], "docs/other.md", "All: the matches by path, then by text");
    key(&mut a, KeyCode::Tab);
    assert_eq!((a.files.find().mode, listed(&a, &root)), (FindMode::Notes, vec!["docs/notes/2026-11-17-find-idea.md".to_string()]));
    assert_eq!(a.files.find().query.text(), "find", "the query stays");
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-24: `shift-tab` goes to the previous mode.
#[tokio::test]
async fn shift_tab_goes_to_the_previous_mode() {
    let root = repo("find-modes-back", &FILES);
    let mut a = app(&root);
    alt(&mut a, '/');
    for want in [FindMode::Notes, FindMode::All, FindMode::Code, FindMode::Docs] {
        a.on_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT));
        assert_eq!(a.files.find().mode, want);
    }
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-25, FIND-26: the modes stand in a row above the query, after `tab`; the active one has the
// accent color, the others the second accent color.
#[tokio::test]
async fn the_modes_stand_above_the_query() {
    let root = repo("find-modes-row", &FILES);
    let mut a = app(&root);
    alt(&mut a, '/');
    let mut term = Terminal::new(TestBackend::new(140, 40)).unwrap();
    term.draw(|f| a.draw(f)).unwrap();
    let buf = term.backend().buffer().clone();
    let rows: Vec<String> = (0..40).map(|y| (0..140).map(|x| buf[(x, y)].symbol().to_string()).collect()).collect();
    let modes = rows.iter().position(|r| r.contains("Notes") && r.contains("Docs") && r.contains("Code") && r.contains("All")).expect("a mode row");
    let query = rows.iter().position(|r| r.contains(" / ")).expect("the query row");
    assert!(modes < query, "{}", rows.join("\n"));
    let row = &rows[modes];
    assert!(row.find("tab").unwrap() < row.find("Notes").unwrap(), "{row}");
    assert!(row.find("Notes").unwrap() < row.find("Docs").unwrap() && row.find("Code").unwrap() < row.find("All").unwrap());
    let t = a.theme();
    let bg = |word: &str| buf[(row[..row.find(word).unwrap()].chars().count() as u16, modes as u16)].bg;
    assert_eq!(bg("Docs"), t.acc, "the active mode");
    assert_eq!((bg("Notes"), bg("Code"), bg("All")), (t.acc2, t.acc2, t.acc2));
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-30, FIND-41: `?` asks the agent only in Notes mode.
#[tokio::test]
async fn notes_mode_keeps_questions() {
    let root = repo("find-modes-question", &FILES);
    let mut a = app(&root);
    alt(&mut a, '/');
    typ(&mut a, "?what is find");
    key(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Find, "Docs mode does not ask");
    a.on_key(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT));
    key(&mut a, KeyCode::Enter);
    assert_eq!(a.mode, Mode::Read, "Notes mode asks the agent");
    a.cli.stop_all();
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-22, FIND-23: `/` opens the panel where the focus takes no text; where it takes text, `/`
// types.
#[tokio::test]
async fn slash_opens_find_where_no_text_is_typed() {
    let root = repo("find-modes-slash", &FILES);
    type Setup = (&'static str, bool, fn(&mut App));
    let setups: [Setup; 7] = [
        ("read", true, |_| {}),
        ("tree", true, |a| alt(a, 'b')),
        ("notes list", true, |a| key(a, KeyCode::Char('o'))),
        ("transcript", true, |a| {
            a.open_panel();
            a.focus = Focus::Claude;
            a.threads.transcript_mut().focus(true);
        }),
        ("write", false, |a| key(a, KeyCode::Char('i'))),
        ("chat input", false, |a| {
            a.open_panel();
            a.focus = Focus::Claude;
        }),
        ("rename field", false, |a| {
            alt(a, 'b');
            let _ = a.files.ask_file(Some("docs/find.md".into()), 'r');
            assert!(a.files.prompt().is_some(), "the rename field");
        }),
    ];
    for (name, opens, setup) in setups {
        let mut a = app(&root);
        a.open_file(root.join("docs/find.md"));
        draw(&mut a, 140, 40);
        setup(&mut a);
        draw(&mut a, 140, 40);
        key(&mut a, KeyCode::Char('/'));
        assert_eq!(a.mode == Mode::Find, opens, "{name}");
        a.cli.stop_all();
    }
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-27: the panel always opens in Docs mode.
#[tokio::test]
async fn find_opens_in_docs_mode() {
    let root = repo("find-modes-docs", &FILES);
    let mut a = app(&root);
    alt(&mut a, '/');
    key(&mut a, KeyCode::Tab);
    key(&mut a, KeyCode::Esc);
    key(&mut a, KeyCode::Char('/'));
    assert_eq!(a.files.find().mode, FindMode::Docs);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-28: Docs mode lists markdown files that are not notes, by path and by text.
#[tokio::test]
async fn docs_mode_has_no_notes_and_no_code() {
    let root = repo("find-modes-docs-only", &FILES);
    let mut a = app(&root);
    alt(&mut a, '/');
    query(&mut a, "find");
    assert_eq!(listed(&a, &root), vec!["docs/find.md", "docs/other.md"]);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-29: Code mode lists text files that are not markdown, by path only; with an empty query the
// recent ones.
#[tokio::test]
async fn code_mode_finds_by_path_only() {
    let root = repo("find-modes-code", &[("src/main.rs", "// find\n"), ("src/find.rs", "\n"), ("docs/a.md", "# A\n")]);
    let mut a = app(&root);
    a.open_file(root.join("src/main.rs"));
    a.tick();
    a.open_file(root.join("docs/a.md"));
    a.tick();
    alt(&mut a, '/');
    key(&mut a, KeyCode::Tab);
    assert_eq!(listed(&a, &root), vec!["src/main.rs"], "the recent code");
    query(&mut a, "find");
    assert_eq!(listed(&a, &root), vec!["src/find.rs"], "not the text of main.rs");
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-31: All mode lists every kind; with an empty query every recent file.
#[tokio::test]
async fn all_mode_finds_every_kind() {
    let root = repo("find-modes-all", &FILES);
    let mut a = app(&root);
    for f in ["src/find.rs", "docs/notes/2026-11-17-find-idea.md", "docs/find.md", "docs/other.md"] {
        a.open_file(root.join(f));
        a.tick();
    }
    alt(&mut a, '/');
    key(&mut a, KeyCode::Tab);
    key(&mut a, KeyCode::Tab);
    assert_eq!(listed(&a, &root), vec!["docs/find.md", "docs/notes/2026-11-17-find-idea.md", "src/find.rs"]);
    let _ = std::fs::remove_dir_all(&root);
}
