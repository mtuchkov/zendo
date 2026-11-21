//! The book (`docs/book`): each file has a review in its front matter, and the review covers the
//! file as it is now. The review keeps the SHA-256 hash of the whole file, with an empty value in
//! the `hash` field. An edit after the review, also of the front matter, changes the hash and fails
//! this test (AGENTS.md, Book).

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const BOOK: &str = "docs/book";

/// The files of the book that hold a review: every Markdown file except the rules for agents.
fn book_files() -> Vec<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(BOOK);
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .filter(|p| p.file_name().is_some_and(|n| n != "AGENTS.md"))
        .collect();
    files.sort();
    files
}

/// The front matter and the text after it. The front matter is between the first line `---` and
/// the next line `---`; the text starts after the newline of that line.
fn split(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---\n")?;
    Some((&rest[..end], &rest[end + "\n---\n".len()..]))
}

/// The value of `  key: value` in the `review:` block, without quotes.
fn field<'a>(front: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("  {key}:");
    let review = front.split_once("review:\n")?.1;
    review
        .lines()
        .take_while(|l| l.starts_with("  "))
        .find_map(|l| l.strip_prefix(&prefix))
        .map(|v| v.trim().trim_matches('"'))
}

fn sha256(text: &str) -> String {
    let digest = Sha256::digest(text.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// The file with an empty value in the `hash` field: the text that the hash covers.
fn without_hash(text: &str) -> String {
    let mut out: String = text
        .lines()
        .map(|l| if l.starts_with("  hash:") { "  hash: \"\"" } else { l })
        .collect::<Vec<_>>()
        .join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// What is wrong with the review of one file, or nothing.
fn problem(text: &str) -> Option<String> {
    let Some((front, _)) = split(text) else { return Some("no front matter".into()) };
    for key in ["reviewer", "date"] {
        if field(front, key).is_none_or(str::is_empty) {
            return Some(format!("no `review.{key}`"));
        }
    }
    if field(front, "passes").and_then(|p| p.parse::<u32>().ok()).is_none_or(|p| p < 1) {
        return Some("`review.passes` is not 1 or more".into());
    }
    if field(front, "result") != Some("pass") {
        return Some("`review.result` is not `pass`".into());
    }
    let want = format!("sha256:{}", sha256(&without_hash(text)));
    match field(front, "hash") {
        Some(h) if h == want => None,
        Some(_) => Some("the file changed after the review: run the zendo-book-reviewer skill".into()),
        None => Some("no `review.hash`".into()),
    }
}

/// A reviewed file: the front matter with the right hash, then `body`.
fn reviewed(passes: &str, result: &str, body: &str) -> String {
    let file = |hash: &str| {
        format!(
            "---\nreview:\n  reviewer: \"agent\"\n  date: \"2026-11-20\"\n  passes: {passes}\n  result: \"{result}\"\n  hash: \"{hash}\"\n---\n{body}"
        )
    };
    file(&format!("sha256:{}", sha256(&file(""))))
}

#[test]
fn each_book_file_has_a_review_of_its_current_text() {
    let files = book_files();
    assert!(!files.is_empty(), "no files in {BOOK}");
    let problems: Vec<String> = files
        .iter()
        .filter_map(|p| {
            let text = std::fs::read_to_string(p).unwrap();
            problem(&text).map(|why| format!("{}: {why}", p.display()))
        })
        .collect();
    assert!(problems.is_empty(), "book files without a valid review:\n{}", problems.join("\n"));
}

#[test]
fn an_edit_after_the_review_fails_the_check() {
    let file = reviewed("1", "pass", "# Chapter 1: A title\n\nText.\n");
    assert_eq!(problem(&file), None);
    let edited = file.replace("Text.", "Text, changed.");
    assert!(problem(&edited).is_some_and(|p| p.contains("changed after the review")));
}

#[test]
fn an_edit_of_the_review_data_fails_the_check() {
    let file = reviewed("1", "pass", "Text.\n");
    let edited = file.replace("2026-11-20", "2026-11-21");
    assert!(problem(&edited).is_some_and(|p| p.contains("changed after the review")));
}

#[test]
fn a_review_needs_one_pass_and_a_pass_result() {
    assert!(problem(&reviewed("0", "pass", "Text.\n")).is_some());
    assert!(problem(&reviewed("1", "fail", "Text.\n")).is_some());
    assert!(problem("no front matter\nText.\n").is_some());
}
