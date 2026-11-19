//! AGENTS.md, Tests: the links between the specifications in `docs/zendo` and the tests. The
//! number of gaps stays at or below `tests/traceability-baseline.toml`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The kinds of acceptance test that do not run in `cargo test`.
const MANUAL: [&str; 3] = ["terminal check", "browser check", "real Claude"];

/// What a specification gives to the check.
#[derive(Default)]
struct Spec {
    path: String,
    status: String,
    /// The architecturally significant requirements.
    asr: Vec<String>,
    /// Every requirement of the tables; apart, the IDs of the `Retired` and `Not used` lines.
    ids: Vec<String>,
    retired: BTreeSet<String>,
    /// The acceptance tests that run in `cargo test`.
    tests: Vec<String>,
}

/// The gaps of rules 1 to 3 and the list of rule 4.
#[derive(Default, Debug, PartialEq)]
struct Report {
    asr_without_test: Vec<String>,
    unknown_ids: Vec<String>,
    acceptance_tests_missing: Vec<String>,
    waits_for_review: Vec<String>,
}

#[derive(serde::Deserialize)]
struct Baseline {
    asr_without_test: usize,
    acceptance_tests_missing: usize,
}

/// The topic and the number of a requirement ID such as `TERM-14`.
fn id(word: &str) -> Option<(&str, u32)> {
    let (topic, n) = word.split_once('-')?;
    let capitals = topic.len() >= 2 && topic.bytes().all(|b| b.is_ascii_uppercase());
    n.parse().ok().filter(|_| capitals && n.bytes().all(|b| b.is_ascii_digit())).map(|n| (topic, n))
}

/// The requirement IDs in a text, such as `TERM-14`; `TERM-14 to TERM-16` gives all three.
fn ids(text: &str) -> BTreeSet<String> {
    let words: Vec<&str> = text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_')).filter(|w| !w.is_empty()).collect();
    let mut out = BTreeSet::new();
    for (i, word) in words.iter().enumerate() {
        let Some((topic, n)) = id(word) else { continue };
        out.insert(word.to_string());
        let end = words.get(i + 1).filter(|w| **w == "to").and_then(|_| words.get(i + 2)).and_then(|w| id(w));
        if let Some((_, m)) = end.filter(|(t, m)| *t == topic && *m > n && m - n < 500) {
            out.extend((n..=m).map(|k| format!("{topic}-{k}")));
        }
    }
    out
}

/// The ID and the pattern of a row of a requirement table.
fn row(line: &str) -> Option<(&str, &str)> {
    let mut cells = line.strip_prefix('|')?.split('|').map(str::trim);
    let (first, pattern) = (cells.next()?, cells.next()?);
    id(first).map(|_| (first, pattern))
}

/// The name of an acceptance test that runs in `cargo test`, from its line in a specification:
/// the name in bold code, then the kind of the test and the IDs that it verifies. A name with
/// spaces is the function with `_` in their place.
fn acceptance(line: &str) -> Option<String> {
    let (name, kind) = line.strip_prefix("**`")?.split_once("`**")?;
    (!MANUAL.iter().any(|m| kind.contains(m))).then(|| name.replace(' ', "_"))
}

fn spec(path: &str, text: &str) -> Spec {
    let mut s = Spec { path: path.into(), ..Default::default() };
    let mut in_asr = false;
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("status:").filter(|_| s.status.is_empty()) {
            s.status = v.trim().trim_matches('"').into();
        }
        if let Some(heading) = line.strip_prefix("## ") {
            in_asr = heading.starts_with("Architecturally significant");
        }
        match row(line) {
            Some((id, "Retired")) => drop(s.retired.insert(id.into())),
            Some((id, _)) => {
                s.asr.extend(in_asr.then(|| id.to_string()));
                s.ids.push(id.into());
            }
            None if line.contains("etired") || line.starts_with("Not used") => s.retired.extend(ids(line)),
            None => {}
        }
        s.tests.extend(acceptance(line));
    }
    s
}

fn report(specs: &[Spec], tests: &str, code: &str) -> Report {
    let named = ids(tests);
    let fns: BTreeSet<&str> = tests.split("fn ").skip(1).filter_map(|s| s.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).next()).collect();
    let known: BTreeSet<&String> = specs.iter().flat_map(|s| s.ids.iter().chain(&s.retired)).collect();
    let topic = |id: &String| id.split_once('-').map_or(String::new(), |(t, _)| t.to_string());
    let topics: BTreeSet<String> = specs.iter().flat_map(|s| &s.ids).map(topic).collect();
    let mut r = Report::default();
    for s in specs.iter().filter(|s| s.status == "implemented" || s.status == "completed") {
        r.asr_without_test.extend(s.asr.iter().filter(|i| !named.contains(*i)).map(|i| format!("{}: {i}", s.path)));
        r.acceptance_tests_missing.extend(s.tests.iter().filter(|t| !fns.contains(t.as_str())).map(|t| format!("{}: {t}", s.path)));
    }
    for s in specs.iter().filter(|s| s.status == "draft") {
        let n = s.ids.iter().filter(|i| named.contains(*i)).count();
        if n > 0 {
            r.waits_for_review.push(format!("{}: tests name {n} of {} requirements", s.path, s.ids.len()));
        }
    }
    r.unknown_ids = named.union(&ids(code)).filter(|i| topics.contains(&topic(i)) && !known.contains(i)).cloned().collect();
    r
}

fn set(ids: &[&str]) -> BTreeSet<String> {
    ids.iter().map(|s| s.to_string()).collect()
}

const SAMPLE: &str = r#"---
status: "implemented"
---

## Architecturally significant requirements

| ID     | Pattern    | Requirement                  | Source |
| ------ | ---------- | ---------------------------- | ------ |
| TERM-1 | Ubiquitous | The system shall show a row. | Design |
| TERM-2 | Event      | When a key comes, it beeps.  | Design |

### Acceptance tests

**`a_row_shows`** (unit test; verifies TERM-1)

**`a key beeps`** (mock agent; verifies TERM-2)

**`the_screen_looks_right`** (terminal check; verifies TERM-1)

## Detailed requirements

| ID      | Pattern | Requirement              | Source |
| ------- | ------- | ------------------------ | ------ |
| TERM-3  | State   | While it waits, it dims. | Design |
| TERM-14 | Retired | Retired on 2026-11-14.   | User   |

Retired in revision 2: TERM-5 (a busy mark), because the user removed it.

Not used: TERM-6 (a proposal).
"#;

#[test]
fn an_id_is_read_alone_and_from_a_range() {
    assert_eq!(ids("`TERM-1`, (TERM-14) and FILES-26 to FILES-28."), set(&["TERM-1", "TERM-14", "FILES-26", "FILES-27", "FILES-28"]));
    assert_eq!(ids("ctrl-A, term-3, T-1, TERM-, TERM-x1 and X1-2"), set(&[]), "these are not IDs");
}

#[test]
fn a_specification_gives_its_requirements_and_tests() {
    let s = spec("terminal.md", SAMPLE);
    assert_eq!(s.status, "implemented");
    assert_eq!(s.asr, ["TERM-1", "TERM-2"]);
    assert_eq!(s.ids, ["TERM-1", "TERM-2", "TERM-3"]);
    assert_eq!(s.retired, set(&["TERM-14", "TERM-5", "TERM-6"]));
    assert_eq!(s.tests, ["a_row_shows", "a_key_beeps"], "a terminal check is not a test function");
}

#[test]
fn the_report_finds_each_kind_of_gap() {
    let draft = spec("draft.md", &SAMPLE.replace("implemented", "draft").replace("TERM-", "FIND-"));
    let specs = [spec("terminal.md", SAMPLE), draft];
    let tests = "// TERM-1, TERM-5, TERM-9, FIND-3, UTF-8\nfn a_row_shows() {}\n";
    let r = report(&specs, tests, "// TERM-3 and FIND-77");
    assert_eq!(r.asr_without_test, ["terminal.md: TERM-2"], "a draft has no gaps");
    assert_eq!(r.unknown_ids, ["FIND-77", "TERM-9"], "a retired ID and a word of another topic are not gaps");
    assert_eq!(r.acceptance_tests_missing, ["terminal.md: a_key_beeps"]);
    assert_eq!(r.waits_for_review, ["draft.md: tests name 1 of 3 requirements"]);
}

fn files(dir: &Path, keep: &dyn Fn(&str) -> bool, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.is_dir() {
            files(&p, keep, out);
        } else if keep(&p.to_string_lossy()) {
            out.push(p);
        }
    }
    out.sort();
}

/// The text of every file under `dir` that `keep` takes.
fn text(dir: &str, keep: &dyn Fn(&str) -> bool) -> String {
    let mut all = Vec::new();
    files(&Path::new(env!("CARGO_MANIFEST_DIR")).join(dir), keep, &mut all);
    all.iter().map(|p| std::fs::read_to_string(p).unwrap()).collect::<Vec<_>>().join("\n")
}

#[test]
fn the_specifications_and_the_tests_agree() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut docs = Vec::new();
    files(&root.join("docs/zendo"), &|p| p.ends_with(".md"), &mut docs);
    let specs: Vec<Spec> = docs.iter().map(|p| spec(&p.strip_prefix(root).unwrap().to_string_lossy(), &std::fs::read_to_string(p).unwrap())).collect();
    assert!(specs.iter().any(|s| !s.asr.is_empty()), "the check found no requirement in docs/zendo");
    // This file is not read: its sample texts hold IDs that no specification has.
    let tests = text("tests", &|p| p.ends_with(".rs") && !p.ends_with("traceability.rs")) + &text("src", &|p| p.ends_with("_tests.rs"));
    let code = text("src", &|p| p.ends_with(".rs") && !p.ends_with("_tests.rs"));
    let r = report(&specs, &tests, &code);
    println!("Specifications that wait for the review of the user:\n{}", r.waits_for_review.join("\n"));
    let base: Baseline = toml::from_str(&std::fs::read_to_string(root.join("tests/traceability-baseline.toml")).unwrap()).unwrap();
    assert!(r.unknown_ids.is_empty(), "IDs in the tests or the code that no specification has:\n{}", r.unknown_ids.join("\n"));
    assert!(
        r.asr_without_test.len() <= base.asr_without_test,
        "architecturally significant requirements of implemented specifications without a test ({} allowed):\n{}",
        base.asr_without_test,
        r.asr_without_test.join("\n")
    );
    assert!(
        r.acceptance_tests_missing.len() <= base.acceptance_tests_missing,
        "acceptance tests of implemented specifications that are not test functions ({} allowed):\n{}",
        base.acceptance_tests_missing,
        r.acceptance_tests_missing.join("\n")
    );
}
