use super::*;
use chrono::Local;
use std::path::PathBuf;

fn note(body: &str) -> Note {
    Note {
        path: PathBuf::from("x.md"),
        title: "x".into(),
        created: None,
        modified: Local::now(),
        body: body.into(),
    }
}

fn lines_of(notes: &[Note], q: &str) -> Vec<Vec<usize>> {
    crate::docs::search::search(notes, q)
        .iter()
        .map(|g| g.snippets.iter().map(|s| s.line).collect())
        .collect()
}

// FIND-20: `⏎` matches a line break, and only a line break.
#[test]
fn a_new_line_mark_matches_a_line_break() {
    let notes = vec![
        note("# T\n\nend of one\nstart of two\n"),
        note("end of one start of two\n"),
    ];
    let g = crate::docs::search::search(&notes, "of one⏎start");
    assert_eq!(g.len(), 1, "the text in one line is no match");
    assert_eq!(
        (g[0].note, g[0].snippets[0].line),
        (0, 2),
        "the snippet is the first line of the match"
    );
}

// FIND-20: letter case, indentation, list markers and blank lines between the lines do not count.
#[test]
fn case_indentation_markers_and_blank_lines_do_not_count() {
    let notes = vec![note("# T\n\n- Item one\n  - item TWO\n\n## Next\n")];
    assert_eq!(lines_of(&notes, "• item one⏎• item two"), vec![vec![2]]);
    assert_eq!(lines_of(&notes, "item two⏎⏎next"), vec![vec![3]]);
    assert_eq!(lines_of(&notes, "item two⏎next"), vec![vec![3]]);
}

// FIND-20: a space of the query matches a line break where read mode joins the lines of a paragraph.
#[test]
fn a_space_matches_a_line_break_of_a_paragraph() {
    let notes = vec![note("# T\n\nfirst line of para\nsecond line\n\n- next\n")];
    assert_eq!(
        lines_of(&notes, "of para second line⏎• next"),
        vec![vec![2]]
    );
}

// FIND-20: a text shorter than the query is no match.
#[test]
fn a_short_text_is_no_match() {
    assert!(lines_of(&[note("a")], "a⏎b").is_empty());
}
