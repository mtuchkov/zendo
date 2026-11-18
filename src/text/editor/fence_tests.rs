use super::*;

fn at_end(text: &str) -> Editor {
    let mut e = Editor::new(text);
    e.move_to_end();
    e
}

// TEXT-7: Enter after an opening fence puts the cursor on a new line and closes the fence under it.
#[test]
fn enter_after_an_opening_fence_closes_it() {
    for (open, close) in [("```", "```"), ("```rust", "```"), ("  ```sh", "  ```"), ("~~~", "~~~"), ("````md", "````")] {
        let mut e = at_end(&format!("text\n{open}"));
        e.enter(true);
        assert_eq!(e.text(), format!("text\n{open}\n{}\n{close}", indent_of(open)), "{open:?}");
        assert_eq!((e.row, e.col), (2, indent_of(open).len()), "{open:?}");
    }
}

fn indent_of(s: &str) -> String {
    s.chars().take_while(|c| *c == ' ').collect()
}

// TEXT-7: a fence that closes a block, or that has a closing fence already, gets no new fence.
#[test]
fn a_closed_fence_gets_no_second_close() {
    let mut e = at_end("```\ncode\n```");
    e.enter(true);
    assert_eq!(e.text(), "```\ncode\n```\n");
    let mut e = Editor::new("```\n```");
    e.row = 0;
    e.end();
    e.enter(true);
    assert_eq!(e.text(), "```\n\n```");
}

// TEXT-7: inside a code block, Enter keeps the indent and does not go on with a list.
#[test]
fn inside_a_code_block_enter_keeps_the_indent() {
    let mut e = Editor::new("```\n    - x\n```");
    e.row = 1;
    e.end();
    assert!(e.in_fence());
    e.enter(true);
    assert_eq!(e.text(), "```\n    - x\n    \n```");
}

// TEXT-6: the code key on a selection of whole lines puts fences around them; again takes them away.
#[test]
fn code_on_lines_puts_fences_around_them() {
    let mut e = Editor::new("a\nb\nc");
    e.anchor = Some((0, 0));
    (e.row, e.col) = (1, 1);
    e.wrap_code();
    assert_eq!(e.text(), "```\na\nb\n```\nc");
    e.anchor = Some((0, 0));
    (e.row, e.col) = (3, 3);
    e.wrap_code();
    assert_eq!(e.text(), "a\nb\nc");
}

// TEXT-5: the code key in one line marks the selection or the word with backticks.
#[test]
fn code_in_a_line_marks_backticks() {
    let mut e = Editor::new("call run now");
    e.col = 6;
    e.wrap_code();
    assert_eq!(e.text(), "call `run` now");
    e.col = 7;
    e.wrap_code();
    assert_eq!(e.text(), "call run now");
}
