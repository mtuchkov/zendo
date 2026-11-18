use super::*;
use crate::text::ot::Op;

fn dir(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("zendo-logs-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn loglet(a: &str, b: &str) -> Loglet {
    let mut log = Loglet::default();
    log.record(Op::diff(a, b));
    log
}

// UNDO-4: the loglet of a document and the text at its head go to `.zendo/db.log.json` and come
// back; git does not see the file.
#[test]
fn a_loglet_goes_to_the_file_and_back() {
    let root = dir("back");
    let mut logs = Logs::default();
    logs.put("docs/a.md", loglet("a", "ab"), "ab".into());
    logs.save(&root);
    let back = Logs::load(&root);
    let doc = back.get("docs/a.md").expect("the loglet");
    assert_eq!((doc.text.as_str(), doc.log.len()), ("ab", 1));
    let ignore = fs::read_to_string(root.join(".zendo/.gitignore")).unwrap();
    assert!(ignore.lines().any(|l| l == "db.log.json"), "{ignore}");
    let _ = fs::remove_dir_all(&root);
}

// UNDO-4: the file keeps a limited number of documents; the oldest go first.
#[test]
fn the_oldest_documents_go() {
    let mut logs = Logs::default();
    for i in 0..MAX_DOCS + 3 {
        logs.put(&format!("docs/{i}.md"), loglet("", "x"), "x".into());
    }
    assert_eq!(logs.len(), MAX_DOCS);
    assert!(logs.get("docs/0.md").is_none() && logs.get(&format!("docs/{}.md", MAX_DOCS + 2)).is_some());
}

// UNDO-11: a broken file gives no loglets, and zendo goes on.
#[test]
fn a_broken_file_gives_no_loglets() {
    let root = dir("broken");
    fs::create_dir_all(root.join(".zendo")).unwrap();
    fs::write(root.join(".zendo/db.log.json"), "{ not json").unwrap();
    assert_eq!(Logs::load(&root).len(), 0);
    let _ = fs::remove_dir_all(&root);
}

// UNDO-4: an empty loglet takes the document out of the file.
#[test]
fn an_empty_loglet_goes() {
    let mut logs = Logs::default();
    logs.put("docs/a.md", loglet("a", "ab"), "ab".into());
    logs.put("docs/a.md", Loglet::default(), "ab".into());
    assert_eq!(logs.len(), 0);
}
