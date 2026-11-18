use super::*;

fn enter_at_end(text: &str) -> Editor {
    let mut e = Editor::new(text);
    e.move_to_end();
    e.split_line(true);
    e
}

// TEXT-8: a bullet line starts the next bullet with the same indent and marker.
#[test]
fn bullets_go_on_with_their_marker() {
    for (line, next) in [("- a", "- "), ("* a", "* "), ("+ a", "+ "), ("  - a", "  - "), ("- [ ] a", "- [ ] "), ("- [x] a", "- [ ] ")] {
        let e = enter_at_end(line);
        assert_eq!(e.lines[1], next, "{line:?}");
        assert_eq!(e.col, next.chars().count(), "{line:?}");
    }
}

// TEXT-9: a numbered item starts the next number, from any number, with the same whitespace.
#[test]
fn numbered_items_count_on_from_any_number() {
    for (line, next) in [("1. a", "2. "), ("9. a", "10. "), ("41. a", "42. "), ("  7. a", "  8. "), ("3.  a", "4.  "), ("5.\ta", "6.\t"), ("1) a", "2) ")] {
        let e = enter_at_end(line);
        assert_eq!(e.lines[1], next, "{line:?}");
        assert_eq!(e.col, next.chars().count(), "{line:?}");
    }
}

// TEXT-9: a number without a period and whitespace after it is not a list item.
#[test]
fn a_number_alone_is_not_a_list() {
    for line in ["1.5 kg", "2026", "12.", "v1. a", "-a"] {
        let e = enter_at_end(line);
        assert_eq!(e.lines[1], "", "{line:?}");
    }
}

// TEXT-8, TEXT-9: Enter on an empty item ends the list.
#[test]
fn enter_on_an_empty_item_ends_the_list() {
    for line in ["- ", "12. ", "  - [ ] "] {
        let e = enter_at_end(&format!("x\n{line}"));
        assert_eq!(e.text(), "x\n", "{line:?}");
        assert_eq!((e.row, e.col), (1, 0));
    }
}

// TEXT-8: the text after the cursor goes into the new item.
#[test]
fn the_tail_goes_into_the_new_item() {
    let mut e = Editor::new("3. one two");
    e.col = 6;
    e.split_line(true);
    assert_eq!(e.text(), "3. one\n4.  two");
}
