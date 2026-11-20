//! The writer without an app (ADR 0008).
use super::*;

// ADR 0008, FLOAT-17: a new note starts with an empty title line, the cursor on the first line of
// the body, and has no file; the save gives it one.
#[test]
fn a_new_note_has_no_file_until_the_save() {
    let mut w = Writer::default();
    assert!(w.editor().is_empty() && w.path().is_none());
    w.start_new();
    assert_eq!((w.editor().text().as_str(), w.editor().row, w.editor().col), ("# \n\n", 2, 0));
    w.editor_mut().type_str("Plan");
    assert!(w.editor().dirty && w.saved_at().is_none());
    w.saved("/r/docs/plan.md".into());
    assert!(!w.editor().dirty && w.saved_at().is_some());
    assert_eq!(w.file(), Some(Path::new("/r/docs/plan.md")));
    w.not_saved();
    assert!(w.saved_at().is_none() && w.path().is_some());
}

// ADR 0008: write mode on a file takes its text and its path.
#[test]
fn write_mode_on_a_file_takes_its_text() {
    let mut w = Writer::default();
    w.start_on("# A\n\ntext", "/r/a.md".into());
    assert_eq!(w.editor().lines.len(), 3);
    assert_eq!(w.path(), Some(&PathBuf::from("/r/a.md")));
    w.start_new();
    assert!(w.path().is_none(), "a new note leaves the file");
}

// ADR 0008: the pause after the last edit is the time for the save.
#[test]
fn the_time_since_the_last_edit_counts_from_the_touch() {
    let mut w = Writer::default();
    w.edited_at(Instant::now() - Duration::from_secs(5));
    assert!(w.idle() >= Duration::from_secs(5));
    w.touch();
    assert!(w.idle() < Duration::from_secs(1));
}

// ADR 0008, FLOAT-1, FLOAT-17: the float and the screen behind it swap their drafts, and swap back.
#[test]
fn two_drafts_swap() {
    let mut w = Writer::default();
    w.start_on("behind", "/r/a.md".into());
    let mut other = Draft::new_note();
    w.swap_draft(&mut other);
    assert_eq!((w.editor().text().as_str(), w.path()), (NEW_NOTE, None));
    w.editor_mut().type_str("New");
    w.swap_draft(&mut other);
    assert_eq!((w.editor().text().as_str(), w.file()), ("behind", Some(Path::new("/r/a.md"))));
    assert!(w.float().is_none() && w.take_float().is_none());
}

// ADR 0008, NOTE-25: the mark of "say it here" stays until its end.
#[test]
fn the_mark_stays_until_its_end() {
    let mut w = Writer::default();
    w.start_say(Say { path: "/r/a.md".into(), line: 3, col: 1, key: "(chat)".into(), rewrite: false });
    assert_eq!(w.say().map(|s| (s.line, s.col)), Some((3, 1)));
    w.end_say();
    assert!(w.say().is_none());
}

// ADR 0008: a file waits once for its title or front matter.
#[test]
fn a_file_waits_once_for_its_meta() {
    let mut w = Writer::default();
    assert!(w.ask_meta("a.md".into()) && !w.ask_meta("a.md".into()));
    assert!(w.meta_waits("a.md") && !w.meta_waits("b.md"));
    assert_eq!((w.next_meta().as_deref(), w.next_meta()), (Some("a.md"), None));
}

// SPEC-7, FLOAT-20: the note of a closed float that went to spec keeps the name of its file; the
// mark goes by the file, so it holds for each way to spec.
#[test]
fn a_note_that_went_to_spec_keeps_its_name() {
    let mut w = Writer::default();
    let (first, second) = (Local::now(), Local::now() + chrono::Duration::seconds(1));
    w.float_closed(first, "/r/docs/notes/a.md".into());
    w.float_closed(second, "/r/docs/notes/b.md".into());
    assert!(!w.keeps_name(first) && !w.keeps_name(second));
    w.keep_name_of(Path::new("/r/docs/notes/a.md"));
    assert!(w.keeps_name(first) && !w.keeps_name(second), "only the note that went to spec");
    w.keep_name_of(Path::new("/r/docs/notes/none.md"));
    assert!(!w.keeps_name(second), "a file of no closed float marks nothing");
}
