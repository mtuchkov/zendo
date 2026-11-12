use super::*;

// WORK-13: the review is a summary and the confusing lines, with why.
#[test]
fn a_review_has_a_summary_and_lines() {
    let (summary, lines) = parse_review("SUMMARY: The log keeps events.\nIt does not say for how long.\nLINE 3: \"soon\" has no time.\nLINE 7-8: two owners.\nnoise\nEND\n");
    assert_eq!(summary, "The log keeps events. It does not say for how long.");
    assert_eq!(lines, vec![(2, "\"soon\" has no time.".to_string()), (6, "two owners.".to_string())]);
    assert_eq!(parse_review("").0, "");
}
