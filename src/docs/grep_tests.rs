use super::*;
use std::time::Duration;

/// A folder of its own for each test, with the given files.
fn dir(name: &str, files: &[(&str, &str)]) -> PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-grep-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    for (p, text) in files {
        let p = d.join(p);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    d
}

fn paths(hits: &[Hit], root: &Path) -> Vec<String> {
    hits.iter().map(|h| h.path.strip_prefix(root).unwrap().display().to_string()).collect()
}

fn run(root: &Path, ask: &Ask) -> Vec<Hit> {
    let mut hits = search(&walk(&[root.to_path_buf()]), ask, &|| false).expect("not stopped");
    hits.sort_by(|a, b| a.path.cmp(&b.path));
    hits
}

// FIND-37: the text of markdown files, without regard to letter case, also outside ASCII; code is
// not searched for plain text.
#[test]
fn text_is_found_in_markdown_without_regard_to_case() {
    let root = dir("text", &[("docs/a.md", "# A\n\nThe Конфиг is here.\nконфиг again.\n"), ("src/a.rs", "// конфиг\n"), ("docs/b.md", "# B\n\nnothing\n")]);
    let hits = run(&root, &Ask::Text { query: "КОНФИГ".into(), regex: false });
    assert_eq!(paths(&hits, &root), vec!["docs/a.md"]);
    assert_eq!(hits[0].count, 2);
    assert_eq!(hits[0].snippets.iter().map(|s| s.line).collect::<Vec<_>>(), vec![2, 3], "0-based lines");
    assert_eq!(hits[0].snippets[0].text, "The Конфиг is here.");
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-45: a regular expression, without regard to letter case.
#[test]
fn a_regex_finds_text() {
    let root = dir("regex", &[("docs/a.md", "# A\n\nRetry after 30 s\n"), ("docs/b.md", "# B\n\nretry later\n")]);
    let hits = run(&root, &Ask::Text { query: "retry.*[0-9]+ s".into(), regex: true });
    assert_eq!(paths(&hits, &root), vec!["docs/a.md"]);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-32: an ID that starts with the word, as a whole ID, in markdown and in code; the whole ID
// marks the file as exact.
#[test]
fn an_id_is_found_by_its_start_in_docs_and_code() {
    let root = dir("id", &[("src/find.rs", "// (FIND-9, FIND-21)\n"), ("src/other.rs", "// FIND-21\n"), ("src/x.rs", "// XFIND-9 aFIND-9\n"), ("docs/find.md", "| FIND-9 | a row |\n")]);
    let id = |w: &str| Ask::Id { word: w.into(), patterns: vec![regex::Regex::new(DEFAULT_ID).unwrap()] };
    let hits = run(&root, &id("FIND-9"));
    assert_eq!(paths(&hits, &root), vec!["docs/find.md", "src/find.rs"], "not FIND-21 alone, not XFIND-9");
    assert!(hits.iter().all(|h| h.exact));
    let hits = run(&root, &id("find-1"));
    assert_eq!(paths(&hits, &root), vec!["src/find.rs", "src/other.rs"]);
    assert!(hits.iter().all(|h| !h.exact), "FIND-21 is not FIND-1");
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-34: an ID of any of the tag patterns.
#[test]
fn an_id_of_any_pattern_is_found() {
    let root = dir("id-patterns", &[("src/a.rs", "// REQ_7\n"), ("src/b.rs", "// ADR-0011\n")]);
    let patterns = ["REQ_[0-9]+", "ADR-[0-9]{4}"].map(|p| regex::Regex::new(p).unwrap()).to_vec();
    let hits = run(&root, &Ask::Id { word: "ADR-0011".into(), patterns: patterns.clone() });
    assert_eq!(paths(&hits, &root), vec!["src/b.rs"]);
    let hits = run(&root, &Ask::Id { word: "REQ_7".into(), patterns });
    assert_eq!(paths(&hits, &root), vec!["src/a.rs"]);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-33: a tag that starts with the word, outside headings; code has no tags.
#[test]
fn a_tag_is_found_by_its_start() {
    let root = dir("tag", &[("docs/a.md", "# A\n\nSome #todo here.\n"), ("docs/b.md", "# todo list\n\nnone\n"), ("src/a.rs", "// #todo\n")]);
    let hits = run(&root, &Ask::Tag("to".into()));
    assert_eq!(paths(&hits, &root), vec!["docs/a.md"]);
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-40: a search stops when it is old.
#[test]
fn a_stopped_search_gives_nothing() {
    let root = dir("stop", &[("docs/a.md", "# A\n\nx\n")]);
    assert!(search(&walk(std::slice::from_ref(&root)), &Ask::Text { query: "x".into(), regex: false }, &|| true).is_none());
    let _ = std::fs::remove_dir_all(&root);
}

// FIND-36, FIND-40: the worker answers off the caller's thread; a newer query drops the older one.
#[test]
fn the_worker_answers_the_last_query_only() {
    let root = dir("worker", &[("docs/a.md", "# A\n\nalpha\n"), ("docs/b.md", "# B\n\nbeta\n")]);
    let w = Worker::start(Default::default());
    w.walk(vec![root.clone()]);
    w.search(1, Ask::Text { query: "alpha".into(), regex: false });
    w.search(2, Ask::Text { query: "beta".into(), regex: false });
    let mut got = Vec::new();
    while let Some(out) = w.wait(Duration::from_secs(5)) {
        let done = matches!(out, Out::Hits { seq: 2, .. });
        got.push(out);
        if done {
            break;
        }
    }
    assert!(got.iter().any(|o| matches!(o, Out::Files(f) if f.len() == 2)), "the list of files");
    let hits: Vec<_> = got.iter().filter_map(|o| if let Out::Hits { seq, hits } = o { Some((*seq, hits.len())) } else { None }).collect();
    assert_eq!(hits, vec![(2, 1)], "only the last query");
    let _ = std::fs::remove_dir_all(&root);
}
