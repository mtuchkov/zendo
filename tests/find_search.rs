//! The search of the find panel: `#` for tags and requirement IDs (find-ids.md), the search off the
//! UI thread (find-background.md), and the regex button (find-regex.md).

mod common;
use common::*;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use std::path::Path;
use std::time::Instant;
use zendo::app::{App, FindMode};

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

fn to_mode(a: &mut App, m: FindMode) {
    while a.files.find().mode != m {
        key(a, KeyCode::Tab);
    }
}

fn clear(a: &mut App) {
    a.on_key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
}

// FIND-32: `#` and an ID finds the files with an ID that starts with it, as a whole ID, the whole ID
// first; in Code mode the text of code is searched for it.
#[tokio::test]
async fn hash_finds_a_requirement_id_in_code_and_docs() {
    let root = repo("find-ids", &[("src/find.rs", "// (FIND-9, FIND-19)\n"), ("src/other.rs", "// FIND-19 only\n"), ("docs/find.md", "| FIND-9 | a row |\n")]);
    let mut a = app(&root);
    alt(&mut a, '/');
    to_mode(&mut a, FindMode::Code);
    query(&mut a, "#FIND-9");
    assert_eq!(listed(&a, &root), vec!["src/find.rs"]);
    assert!(a.files.find().items[0].snippets[0].text.contains("FIND-9"), "the line with the ID");
    clear(&mut a);
    query(&mut a, "#FIND-1");
    assert_eq!(listed(&a, &root), vec!["src/find.rs", "src/other.rs"]);
    clear(&mut a);
    to_mode(&mut a, FindMode::Docs);
    query(&mut a, "#FIND-9");
    assert_eq!(listed(&a, &root), vec!["docs/find.md"]);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-33: `#` and a word that is no ID finds a tag in the docs and in the notes; code has no tags.
#[tokio::test]
async fn hash_finds_a_tag_in_docs() {
    let root = repo("find-tags", &[("docs/a.md", "# A\n\nWork #todo here.\n"), ("docs/notes/2026-11-17-n.md", "# N\n\n#todo too\n"), ("src/a.rs", "// #todo\n")]);
    let mut a = app(&root);
    alt(&mut a, '/');
    query(&mut a, "#to");
    assert_eq!(listed(&a, &root), vec!["docs/a.md"], "Docs");
    to_mode(&mut a, FindMode::Notes);
    assert_eq!(listed(&a, &root), vec!["docs/notes/2026-11-17-n.md"], "Notes");
    to_mode(&mut a, FindMode::Code);
    assert!(listed(&a, &root).is_empty(), "Code");
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-34, FIND-35: `tag_patterns` sets the forms of an ID; a bad pattern does not count, and none
// left gives the default.
#[tokio::test]
async fn the_tag_patterns_are_configurable() {
    let structure = "[project]\ntag_patterns = [\"REQ_[0-9]+\", \"ADR-[0-9]{4}\"]\n";
    let root = repo("find-id-pattern", &[("project-structure.toml", structure), ("src/a.rs", "// REQ_7\n"), ("src/b.rs", "// FIND-1\n"), ("src/c.rs", "// ADR-0011\n")]);
    let mut a = app(&root);
    alt(&mut a, '/');
    to_mode(&mut a, FindMode::Code);
    query(&mut a, "#REQ_7");
    assert_eq!(listed(&a, &root), vec!["src/a.rs"]);
    clear(&mut a);
    query(&mut a, "#ADR-0011");
    assert_eq!(listed(&a, &root), vec!["src/c.rs"]);
    key(&mut a, KeyCode::Esc);
    std::fs::write(root.join("project-structure.toml"), "[project]\ntag_patterns = [\"REQ_[0-9\"]\n").unwrap();
    alt(&mut a, '/');
    to_mode(&mut a, FindMode::Code);
    query(&mut a, "#FIND-1");
    assert_eq!(listed(&a, &root), vec!["src/b.rs"], "the default form");
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-36: a key returns before the search in the text ends; the results come later.
#[tokio::test]
async fn typing_does_not_wait_for_the_search() {
    let files: Vec<(String, String)> = (0..2000).map(|i| (format!("docs/d{i}.md"), format!("# D{i}\n\n{}\nneedle {i}\n", "text ".repeat(200)))).collect();
    let refs: Vec<(&str, &str)> = files.iter().map(|(p, t)| (p.as_str(), t.as_str())).collect();
    let root = repo("find-no-wait", &refs);
    let mut a = app(&root);
    alt(&mut a, '/');
    a.settle_find();
    for c in "needle".chars() {
        let t = Instant::now();
        key(&mut a, KeyCode::Char(c));
        assert!(t.elapsed().as_millis() < 50, "the key {c} took {:?}", t.elapsed());
    }
    assert_eq!(a.files.find().query.text(), "needle");
    assert!(a.files.find().running && a.files.find().items.is_empty(), "the text results are not there yet");
    a.settle_find();
    assert_eq!(a.files.find().counts[1], 200, "the results came; Docs lists up to 200 by text");
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-37: one search in the text fills every mode.
#[tokio::test]
async fn one_search_fills_every_mode() {
    let root = repo("find-one-search", &[("docs/a.md", "# A\n\nfind\n"), ("docs/notes/2026-11-17-n.md", "# N\n\nfind\n"), ("src/a.rs", "// find\n")]);
    let mut a = app(&root);
    alt(&mut a, '/');
    query(&mut a, "find");
    assert_eq!(a.files.find().counts, [1, 1, 0, 2], "Notes, Docs, Code (no text of code), All");
    to_mode(&mut a, FindMode::All);
    assert!(!a.files.find().running, "no new search");
    assert_eq!(listed(&a, &root), vec!["docs/a.md", "docs/notes/2026-11-17-n.md"]);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-38: the text comes from the disk for each search.
#[tokio::test]
async fn a_changed_file_is_found_without_a_new_open() {
    let root = repo("find-fresh", &[("docs/a.md", "# A\n\nnothing\n")]);
    let mut a = app(&root);
    alt(&mut a, '/');
    query(&mut a, "alpha");
    assert!(listed(&a, &root).is_empty());
    std::fs::write(root.join("docs/a.md"), "# A\n\nalpha\n").unwrap();
    query(&mut a, " ");
    clear(&mut a);
    query(&mut a, "alpha");
    assert_eq!(listed(&a, &root), vec!["docs/a.md"]);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-39: the list of files comes from a walk when the panel opens.
#[tokio::test]
async fn a_new_file_is_found_after_the_panel_opens_again() {
    let root = repo("find-new-file", &[("docs/a.md", "# A\n\nalpha\n")]);
    let mut a = app(&root);
    alt(&mut a, '/');
    query(&mut a, "alpha");
    std::fs::write(root.join("docs/b.md"), "# B\n\nalpha\n").unwrap();
    clear(&mut a);
    query(&mut a, "alpha");
    assert_eq!(listed(&a, &root), vec!["docs/a.md"], "not in this open");
    key(&mut a, KeyCode::Esc);
    alt(&mut a, '/');
    query(&mut a, "alpha");
    assert_eq!(listed(&a, &root), vec!["docs/a.md", "docs/b.md"]);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-40: a result of an old query does not show.
#[tokio::test]
async fn a_late_result_of_an_old_query_does_not_show() {
    let root = repo("find-late", &[("docs/a.md", "# A\n\nalpha\n"), ("docs/b.md", "# B\n\nbeta\n")]);
    let mut a = app(&root);
    alt(&mut a, '/');
    a.settle_find();
    typ(&mut a, "alpha");
    clear(&mut a);
    query(&mut a, "beta");
    std::thread::sleep(std::time::Duration::from_millis(300));
    a.poll_find();
    assert_eq!(listed(&a, &root), vec!["docs/b.md"]);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-41: `?` is no search.
#[tokio::test]
async fn a_question_is_not_searched() {
    let root = repo("find-question", &[("docs/a.md", "# A\n\nalpha\n")]);
    let mut a = app(&root);
    alt(&mut a, '/');
    typ(&mut a, "?alpha");
    assert!(!a.files.find().running && a.files.find().items.is_empty());
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-42: All is the merge of Notes, Docs and Code, in one order: by path first, then by text.
#[tokio::test]
async fn all_is_the_merge_of_the_three_modes() {
    let root = repo("find-merge", &[("docs/x.md", "# X\n\nzeta zeta\n"), ("docs/notes/2026-11-17-zeta.md", "# Z\n"), ("src/zeta.rs", "\n"), ("docs/notes/2026-11-17-n.md", "# N\n\nzeta\n")]);
    let mut a = app(&root);
    alt(&mut a, '/');
    to_mode(&mut a, FindMode::All);
    query(&mut a, "zeta");
    assert_eq!(listed(&a, &root), vec!["docs/notes/2026-11-17-zeta.md", "src/zeta.rs", "docs/x.md", "docs/notes/2026-11-17-n.md"]);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-43: the mode row shows `…` while the search runs, then the number of results.
#[tokio::test]
async fn the_mode_row_shows_counts_and_running_searches() {
    let root = repo("find-counts", &[("docs/a.md", "# A\n\nalpha\n")]);
    let mut a = app(&root);
    alt(&mut a, '/');
    a.settle_find();
    typ(&mut a, "alpha");
    let row = |a: &mut App| draw(a, 140, 40).into_iter().find(|r| r.contains("Notes") && r.contains("All")).unwrap();
    assert!(row(&mut a).contains("Docs …"), "{}", row(&mut a));
    a.settle_find();
    assert!(row(&mut a).contains("Docs 1"), "{}", row(&mut a));
    let _ = std::fs::remove_dir_all(&root);
}

/// A click on `[.*]`.
fn click_regex(a: &mut App) {
    let r = a.frame.find_regex().expect("the button");
    a.on_mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column: r.x + 1, row: r.y, modifiers: KeyModifiers::NONE });
}

// FIND-44: `alt-r` and a click on `[.*]` turn the regular expression on and off.
#[tokio::test]
async fn the_regex_button_turns_on_and_off() {
    let root = repo("find-regex-button", &[("src/config.json", "{}\n"), ("docs/config-toml.md", "# C\n")]);
    let mut a = app(&root);
    alt(&mut a, '/');
    to_mode(&mut a, FindMode::Code);
    query(&mut a, "config.*json");
    assert!(listed(&a, &root).is_empty(), "words");
    alt(&mut a, 'r');
    a.settle_find();
    assert_eq!(listed(&a, &root), vec!["src/config.json"], "a regular expression for the path");
    draw(&mut a, 140, 40);
    click_regex(&mut a);
    a.settle_find();
    assert!(!a.files.find().regex && listed(&a, &root).is_empty(), "a click turns it off");
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-45: the expression matches the text too, and the preview marks the match.
#[tokio::test]
async fn a_regex_matches_path_and_text() {
    let root = repo("find-regex-text", &[("docs/a.md", "# A\n\nRetry after 30 s\n"), ("docs/b.md", "# B\n\nretry later\n")]);
    let mut a = app(&root);
    alt(&mut a, '/');
    alt(&mut a, 'r');
    query(&mut a, "retry.*[0-9]+ s");
    assert_eq!(listed(&a, &root), vec!["docs/a.md"]);
    assert_eq!(a.files.find().items[0].snippets[0].hits, vec![(0, 16)], "the match in the line");
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-46: an expression that does not compile keeps the last results and marks the button.
#[tokio::test]
async fn a_bad_regex_keeps_the_last_results() {
    let root = repo("find-regex-bad", &[("docs/a.md", "# A\n\nretry\n")]);
    let mut a = app(&root);
    alt(&mut a, '/');
    alt(&mut a, 'r');
    query(&mut a, "retr");
    typ(&mut a, "(");
    assert!(a.files.find().bad_regex);
    assert_eq!(listed(&a, &root), vec!["docs/a.md"]);
    let screen = draw(&mut a, 140, 40);
    assert!(screen.iter().any(|l| l.contains("[.*]")));
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-47: the state of `[.*]` stays from one open to the next.
#[tokio::test]
async fn the_regex_state_stays() {
    let root = repo("find-regex-stays", &[("docs/a.md", "# A\n")]);
    let mut a = app(&root);
    alt(&mut a, '/');
    alt(&mut a, 'r');
    key(&mut a, KeyCode::Esc);
    alt(&mut a, '/');
    assert!(a.files.find().regex);
    let _ = std::fs::remove_dir_all(&root);
}

// FILES-11, FILES-67: `` ` `` filters the tree by name; `[.*]` and `alt-r` make it a regular
// expression.
#[tokio::test]
async fn the_tree_filter_takes_a_regex() {
    let root = repo("find-tree-regex", &[("docs/a-1.md", "# A\n"), ("docs/b-2.md", "# B\n"), ("docs/c.md", "# C\n")]);
    let mut a = app(&root);
    alt(&mut a, 'b');
    draw(&mut a, 140, 40);
    key(&mut a, KeyCode::Char('`'));
    assert!(a.files.filtering());
    typ(&mut a, "-[0-9]");
    assert!(a.files.tree().rows().is_empty(), "words");
    alt(&mut a, 'r');
    let rows: Vec<String> = a.files.tree().rows().iter().map(|r| r.rel.display().to_string()).collect();
    assert_eq!(rows, vec!["docs/a-1.md", "docs/b-2.md"]);
    let screen = draw(&mut a, 140, 40);
    assert!(screen.iter().any(|l| l.contains("[.*]")), "the button");
    click_regex(&mut a);
    assert!(a.files.tree().rows().is_empty(), "a click turns it off");
    let _ = std::fs::remove_dir_all(&root);
}
