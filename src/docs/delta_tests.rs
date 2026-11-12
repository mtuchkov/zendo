use super::*;

#[test]
fn counts_and_numbers() {
    let d = compute("a\nb\nc\n", "a\nB x\nc\nd\n", 1);
    assert_eq!((d.added, d.removed), (2, 1));
    let del = d.rows.iter().find(|r| r.tag == Tag::Del).unwrap();
    assert_eq!(del.old, Some(2));
    let add = d.rows.iter().find(|r| r.tag == Tag::Add).unwrap();
    assert_eq!(add.new, Some(2));
}

#[test]
fn changed_words_are_marked() {
    let d = compute("the relay shall publish each row\n", "the relay shall publish each log row\n", 0);
    let add = d.rows.iter().find(|r| r.tag == Tag::Add).unwrap();
    let emph: String = add.parts.iter().filter(|(e, _)| *e).map(|(_, s)| s.as_str()).collect();
    assert!(emph.contains("log"), "{:?}", add.parts);
    assert!(!emph.contains("relay"));
}

#[test]
fn far_changes_get_a_gap() {
    let old: String = (1..=20).map(|i| format!("l{i}\n")).collect();
    let new = old.replace("l2\n", "x\n").replace("l18\n", "y\n");
    let d = compute(&old, &new, 1);
    assert!(d.rows.iter().any(|r| r.tag == Tag::Gap));
}

#[test]
fn rows_fill_the_width() {
    let d = compute("a\n", "b\n", 0);
    let lines = render(&d, 30, &crate::ui::theme::THEMES[0]);
    for l in lines {
        let w: usize = l.spans.iter().map(|s| s.content.width()).sum();
        assert_eq!(w, 30);
    }
}
