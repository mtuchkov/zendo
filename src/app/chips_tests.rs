use super::*;

// CLAUDE-20, CLAUDE-41: a text chip says the first 15 chars of the text and, always, its lines.
#[test]
fn a_text_chip_says_its_first_chars_and_its_lines() {
    assert_eq!(text_label("IF a legal hold covers an event"), "IF a legal hold… · 1 line");
    assert_eq!(text_label("short"), "short · 1 line");
    assert_eq!(text_label("IF a legal\nhold covers\nan event"), "IF a legal hold… · 3 lines");
    // Blank lines at the ends do not count.
    assert_eq!(text_label("\n  fn main() {}\n\n"), "fn main() {} · 1 line");
}

// CLAUDE-42: a paste of more than one line, or of a long line, is a chip; a short line is text.
#[test]
fn a_long_paste_is_a_chip_and_a_short_one_is_text() {
    assert!(paste_is_chip("a\nb"));
    assert!(paste_is_chip(&"x".repeat(PASTE_CHIP_CHARS + 1)));
    assert!(!paste_is_chip(&"x".repeat(PASTE_CHIP_CHARS)), "200 chars stay text");
    assert!(!paste_is_chip("hello world"));
    assert!(!paste_is_chip("hello\n"), "one line and a new line at the end");
}

// CLAUDE-21: a file chip says the last 4 chars of its folder and the first 10 chars of its name.
#[test]
fn a_file_chip_says_its_folder_tail_and_name_head() {
    assert_eq!(file_label(Path::new("/r/docs/guides/templates/specification.md")), "ates/specificat");
    assert_eq!(file_label(Path::new("/r/a/x.png")), "a/x.png");
}

#[test]
fn chips_are_found_by_their_marks() {
    assert_eq!(chip_ranges("⟦ab⟧ x ⟦c⟧"), vec![(0, 4), (7, 10)]);
    assert!(chip_ranges("[selection] plain").is_empty());
}

// CLAUDE-61, CLAUDE-64: an escaped text reads back as it was; a run of `\\` before a mark or at
// the end doubles; the chips of a question with escapes are found.
#[test]
fn escapes_go_and_come_back() {
    for t in ["plain", "a ⟦b⟧ c", "\\⟦", "end \\", "\\\\⟧x", "C:\\dir\\file"] {
        assert_eq!(unescape(&escape(t)), t, "{t:?} -> {:?}", escape(t));
    }
    assert_eq!(escape("a ⟦b⟧"), "a \\⟦b\\⟧");
    assert_eq!(escape("end \\"), "end \\\\", "a run at the end doubles");
    assert_eq!(escape("C:\\dir"), "C:\\dir", "other runs stay");
    let q = format!("why {OPEN}{}{CLOSE} now", escape("x ⟦y⟧ \\"));
    let r = chip_ranges(&q);
    assert_eq!(r.len(), 1, "{q}");
    let inner: String = q.chars().skip(r[0].0 + 1).take(r[0].1 - r[0].0 - 2).collect();
    assert_eq!(unescape(&inner), "x ⟦y⟧ \\");
}

// CLAUDE-64: the label of a chip from its text: the quoted lines, `↩` for text from the chat, a
// file for `@path`, the first chars of other text.
#[test]
fn a_chip_of_a_question_gets_a_label_from_its_text() {
    assert_eq!(label_of("Text from this chat:\n> what is it"), format!("{} what is it · 1 line", crate::app::CHAT_MARK));
    assert_eq!(label_of("Selected text from docs/a.md (about lines 1–2):\n> one\n> two"), "one two · 2 lines");
    assert_eq!(label_of("@/r/docs/notes/b.md"), "otes/b.md");
    assert_eq!(label_of("Template: docs/t.md. Write it"), "Template: docs/…");
}

// SPEC-10: a text with a chip goes before the draft; the chip keeps its text, with new lines and
// marks, and the text goes with the question only while the chip is in the input.
#[test]
fn a_text_with_a_chip_goes_before_the_draft() {
    let mut c = Composer::default();
    c.input_mut().type_str("draft");
    let rules = format!("Rule one.\n@not a file\nA mark {OPEN}in{CLOSE} the text.");
    c.put_first("/new The task.", "note to spec", &rules);
    let text = c.input().text();
    assert!(text.starts_with("/new The task. ") && text.ends_with("\ndraft"), "{text:?}");
    assert_eq!(c.chips().len(), 1);
    assert!(text.contains(&c.chips()[0].label) && c.chips()[0].attach.is_none(), "a text chip in the input: {:?}", c.chips());
    let q = c.take_question();
    assert!(q.text.starts_with("/new The task. ") && q.text.contains("Rule one.\n@not a file\nA mark"), "the text of the chip goes: {:?}", q.text);
    c.put_first("/new The task.", "note to spec", &rules);
    c.input_mut().clear_text();
    c.input_mut().type_str("another question");
    assert_eq!(c.take_question().text, "another question", "a chip that is not in the input does not go");
}
