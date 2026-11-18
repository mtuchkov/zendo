use super::*;
use chrono::NaiveDate;

fn day(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn iso() -> DateFormat {
    DateFormat::from_locale("C")
}

/// An editor with `text` typed and the date selector opened by `//` at its end.
fn picked(text: &str, fmt: DateFormat) -> Editor {
    let mut e = Editor::new(text);
    e.end();
    assert!(e.slashes_to_date(day(2026, 11, 18), fmt));
    e
}

// TEXT-15: the format of the date comes from the locale of the system.
#[test]
fn the_locale_gives_the_format() {
    let d = day(2026, 1, 5);
    for (locale, text) in [("C", "2026-01-05"), ("", "2026-01-05"), ("en_US.UTF-8", "01/05/2026"), ("en_GB.UTF-8", "05/01/2026"), ("ru_RU.UTF-8", "05.01.2026"), ("de_DE", "05.01.2026"), ("ja_JP.UTF-8", "2026/01/05"), ("fr_FR.UTF-8", "05/01/2026")] {
        assert_eq!(DateFormat::from_locale(locale).format(d), text, "{locale}");
    }
}

// TEXT-15: `//` at the start of a word puts the date of today, with its first part selected.
#[test]
fn slashes_put_the_date_of_today() {
    let e = picked("due //", iso());
    assert_eq!(e.text(), "due 2026-11-18");
    assert!(e.date.is_some());
    assert_eq!(e.selected_text().as_deref(), Some("2026"));
}

// TEXT-15: `//` inside a word, a link, a code block or inline code is text.
#[test]
fn slashes_in_a_word_or_code_are_text() {
    for text in ["https://", "a//", "```\n//", "`x //"] {
        let mut e = Editor::new(text);
        e.move_to_end();
        assert!(!e.slashes_to_date(day(2026, 11, 18), iso()), "{text:?}");
        assert_eq!(e.text(), text);
    }
}

// TEXT-16: up and down change the selected part; left and right go to the next part.
#[test]
fn the_keys_change_the_parts() {
    let mut e = picked("//", iso());
    assert!(e.date_key(DateKey::Up));
    assert_eq!(e.text(), "2027-11-18");
    assert!(e.date_key(DateKey::Right));
    assert_eq!(e.selected_text().as_deref(), Some("11"));
    e.date_key(DateKey::Up);
    e.date_key(DateKey::Up);
    assert_eq!(e.text(), "2028-01-18", "a month after December is January of the next year");
    e.date_key(DateKey::Right);
    e.date_key(DateKey::Down);
    assert_eq!(e.text(), "2028-01-17");
    let mut e = picked("//", DateFormat::from_locale("ru_RU"));
    assert_eq!(e.selected_text().as_deref(), Some("18"), "the first part of the format");
    (0..13).for_each(|_| assert!(e.date_key(DateKey::Up)));
    assert_eq!(e.text(), "01.12.2026", "a day after the end of the month is in the next month");
}

// TEXT-17: left on the first part, right on the last part and enter leave the date selector.
#[test]
fn the_selector_ends_at_its_border_and_with_enter() {
    let mut e = picked("a //", iso());
    assert!(e.date_key(DateKey::Left));
    assert!(e.date.is_none() && e.selection().is_none());
    assert_eq!(e.col, 2, "left leaves before the date");
    let mut e = picked("a //", iso());
    e.date_key(DateKey::Right);
    e.date_key(DateKey::Right);
    assert!(e.date.is_some(), "on the last part");
    e.date_key(DateKey::Right);
    assert!(e.date.is_none());
    assert_eq!(e.col, 12, "right leaves after the date");
    let mut e = picked("a //", iso());
    assert!(e.date_key(DateKey::Enter));
    assert_eq!((e.date.is_none(), e.col), (true, 12));
}

// TEXT-17: another key leaves the selector and does what it does in the editor.
#[test]
fn another_key_leaves_the_selector() {
    let mut e = picked("//", iso());
    assert!(!e.date_key(DateKey::Other));
    assert!(e.date.is_none() && e.selection().is_none());
    e.type_over(' ');
    assert_eq!(e.text(), "2026-11-18 ");
}
