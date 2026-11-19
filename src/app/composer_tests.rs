//! The composer without an app (ADR 0008).
use super::*;
use crate::agent::Command;
use crate::setup::history::{SavedChip, Sent};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

// ADR 0008, CLAUDE-17: a chip gets a free label, and the arrows and delete take it as one unit.
#[test]
fn a_chip_has_a_free_label_and_is_one_unit() {
    let mut c = Composer::default();
    assert_eq!(c.add_chip("note".into(), Some("text one".into()), None, false), "⟦note⟧");
    assert_eq!(c.add_chip("note".into(), None, None, false), "⟦note 2⟧");
    assert_eq!(c.input().text(), "⟦note⟧ ⟦note 2⟧ ");
    c.input_mut().left();
    assert!(c.chip_key(KeyCode::Backspace), "the chip before the cursor");
    assert_eq!(c.input().text(), "⟦note⟧  ");
    assert_eq!(c.chips().len(), 1);
    c.clear();
    assert!(c.input().is_empty() && c.chips().is_empty());
}

// ADR 0008, CLAUDE-59, CLAUDE-60, CLAUDE-62: each chip goes at its place, a text as its text in
// marks, a file as its label with the file attached; the transcript keeps `@` and the path of a
// file; a chip that the user deleted from the text gives nothing.
#[test]
fn the_question_takes_the_context_and_the_files_of_its_chips() {
    // A file that is there: the file of a red chip does not go (CLAUDE-57).
    let dir = std::env::temp_dir().join(format!("zendo-composer-question-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let png = dir.join("a.png");
    std::fs::write(&png, b"png").unwrap();
    let mut c = Composer::default();
    c.add_chip("sel".into(), Some("selected text".into()), None, false);
    c.add_chip("img".into(), None, Some(Attachment::Image(png)), false);
    c.add_chip("gone".into(), Some("not sent".into()), None, false);
    let kept = c.input().text().replace("⟦gone⟧ ", "");
    c.input_mut().take();
    c.input_mut().type_str(&format!("{kept}why?"));
    let q = c.take_question();
    assert_eq!(q.text, "⟦selected text⟧ ⟦img⟧ why?");
    assert_eq!(q.shown, format!("⟦sel\u{2063}selected text⟧ ⟦img\u{2063}@{}⟧ why?", dir.join("a.png").display()));
    assert!(c.sends_image());
    assert_eq!(c.take_attach().len(), 1);
    assert_eq!(c.take_context(), None, "the text of a chip is in the question");
    assert!(c.input().is_empty() && c.chips().is_empty() && !c.has_context());
    let _ = std::fs::remove_dir_all(&dir);
}

// ADR 0008: with a context and no words of the user, the default question goes.
#[test]
fn only_chips_send_the_default_question() {
    let mut c = Composer::default();
    c.set_context("the delta".into());
    c.set_default("What changed?".into(), "the changes");
    assert_eq!(c.about(), "the changes");
    assert_eq!(c.take_question().text, "What changed?");
    c.add_chip("sel".into(), Some("text".into()), None, false);
    assert_eq!(c.take_question().text, format!("⟦text⟧ {}", crate::app::DELTA_QUESTION), "no default left: the question of the delta");
    assert_eq!(c.take_context().as_deref(), Some("the delta"));
    c.input_mut().type_str("plain");
    assert_eq!(c.take_question().text, "plain", "no context: the text as it is");
}

// ADR 0008, CLAUDE-28: `/` and a part of a name open the commands; tab takes the selected one.
#[test]
fn slash_suggests_the_commands_and_tab_takes_one() {
    let mut c = Composer::default();
    let cmd = |name: &str| Command { name: name.into(), description: String::new(), hint: None };
    c.set_commands(vec![cmd("review"), cmd("compact")]);
    c.input_mut().type_str("/co");
    c.update_slash();
    assert!(c.slash().open);
    assert_eq!(c.slash().hits.len(), 1);
    assert!(c.key_slash(key(KeyCode::Tab)));
    assert_eq!(c.input().text(), "/compact ");
    assert!(!c.slash().open && !c.key_slash(key(KeyCode::Char('x'))));
}

// ADR 0008: `@` and a part of a name suggest files; enter takes one and removes `@word`.
#[test]
fn at_suggests_files_and_enter_takes_one() {
    // The files of the repository, as the worker reads them (IO-1).
    let files = || Some(vec![PathBuf::from("docs/other.md"), PathBuf::from("docs/plan.md")]);
    let mut c = Composer::default();
    c.input_mut().type_str("see @pla");
    c.update_at(None, &[]);
    assert_eq!(c.at().map(|p| p.hits.len()), Some(0), "the suggestions wait for the files");
    c.update_at(files(), &[]);
    assert_eq!(c.at().map(|p| p.hits.clone()), Some(vec![PathBuf::from("docs/plan.md")]));
    assert_eq!(c.key_at(key(KeyCode::Char('x'))), AtKey::Unused);
    assert_eq!(c.key_at(key(KeyCode::Enter)), AtKey::Taken("docs/plan.md".into(), false));
    assert_eq!(c.input().text(), "see ");
    assert!(c.at().is_none());
    c.pick_template();
    assert_eq!(c.input().text(), "see @");
    c.update_at(files(), &["docs/other.md".into()]);
    assert_eq!(c.at().map(|p| (p.templates, p.hits.clone())), Some((true, vec![PathBuf::from("docs/other.md")])));
    c.close_picks();
    assert!(c.at().is_none());
}

// ADR 0008: `\` before the cursor makes `enter` a new line.
#[test]
fn a_backslash_before_the_cursor_is_seen() {
    let mut c = Composer::default();
    assert!(!c.ends_with_backslash());
    c.input_mut().type_str("one\\");
    assert!(c.ends_with_backslash());
}

// ADR 0008, TERM-28: `alt-o` again puts the input back and takes the next terminal; another key
// starts with the first terminal again.
#[test]
fn a_second_pick_puts_the_input_back_and_takes_the_next_terminal() {
    let mut c = Composer::default();
    c.input_mut().type_str("see ");
    assert_eq!(c.next_pick(3), 0);
    c.input_mut().type_str("output of 1");
    assert_eq!(c.next_pick(3), 1);
    assert_eq!(c.input().text(), "see ", "the output of the first terminal is gone");
    assert_eq!((c.next_pick(3), c.next_pick(3)), (2, 0));
    c.drop_pick();
    c.input_mut().type_str("more");
    assert_eq!(c.next_pick(3), 0);
    assert_eq!(c.input().text(), "see more");
}

fn sent(text: &str, chips: Vec<SavedChip>) -> Sent {
    Sent { text: text.into(), at: chrono::Local::now(), chips, merged: None }
}

fn history() -> Vec<Sent> {
    vec![sent("first", vec![]), sent("two\nlines", vec![]), sent("last", vec![])]
}

// HISTORY-5, HISTORY-6: `up` goes back through the history, `down` comes forward, and after the
// newest message the draft of the user comes back.
#[test]
fn up_and_down_go_through_the_history_and_bring_the_draft_back() {
    let mut c = Composer::default();
    c.input_mut().type_str("my draft");
    let h = history();
    assert!(c.recall_back(&h));
    assert_eq!(c.input().text(), "last");
    assert!(c.recall_back(&h));
    assert_eq!(c.input().text(), "two\nlines");
    assert_eq!((c.input().row, c.input().col), (0, 0), "the cursor is on the first row: the next up goes back");
    assert!(c.recall_back(&h));
    assert_eq!(c.input().text(), "first");
    assert!(!c.recall_back(&h), "nothing before the oldest");
    assert_eq!(c.input().text(), "first");
    assert!(c.recall_forward(&h));
    assert_eq!(c.input().text(), "two\nlines");
    assert_eq!(c.input().row, 1, "the cursor is on the last row: the next down goes forward");
    assert!(c.recall_forward(&h));
    assert!(c.recall_forward(&h));
    assert_eq!(c.input().text(), "my draft", "the draft comes back");
    assert!(!c.recall_forward(&h), "down without a recall is a plain down");
}

// HISTORY-5: an empty history changes nothing, and a sent message ends the recall.
#[test]
fn a_send_ends_the_recall() {
    let mut c = Composer::default();
    assert!(!c.recall_back(&[]));
    let h = history();
    c.recall_back(&h);
    c.take_message();
    assert!(!c.recall_forward(&h), "after a send, down does not go through the history");
    assert!(c.recall_back(&h));
    assert_eq!(c.input().text(), "last", "a new recall starts at the newest message");
}

// HISTORY-7, HISTORY-5, HISTORY-6, CLAUDE-59: a message from the history brings its text chips
// back as chips: one unit for the keys, and their text goes at their places in the question. The
// draft gets its own chips back.
#[test]
fn a_recalled_message_brings_its_text_chips_back() {
    let mut c = Composer::default();
    c.paste_chip("draft text\nof two lines");
    let label = "⟦fn main · 3 lines⟧";
    let h = vec![sent(&format!("why {label}"), vec![SavedChip { label: label.into(), text: "fn main() {}".into() }])];
    assert!(c.recall_back(&h));
    assert_eq!(c.chips().len(), 1);
    assert_eq!(c.chips()[0].label, label);
    c.input_mut().move_to_end();
    assert!(c.chip_key(KeyCode::Left), "the recalled chip is one unit");
    assert!(c.recall_forward(&h));
    assert_eq!(c.chips().len(), 1);
    assert!(c.chips()[0].label.starts_with("⟦draft text"), "the draft has its own chip again");
    assert!(c.recall_back(&h));
    assert_eq!(c.take_question().text, "why ⟦fn main() {}⟧");
    assert_eq!(c.take_context(), None);
}

// HISTORY-7: the history keeps the text chips of a message and their labels; the labels of file
// and image chips leave the text.
#[test]
fn the_history_entry_keeps_text_chips_and_drops_file_chips() {
    let mut c = Composer::default();
    c.input_mut().type_str("see ");
    let text = c.add_chip("sel".into(), Some("selected".into()), None, false);
    let file = c.add_chip("file".into(), None, Some(Attachment::File("/tmp/a.md".into())), false);
    c.input_mut().type_str("now");
    let (t, chips) = c.history_entry();
    assert_eq!(t, format!("see {text}  now"));
    assert!(!t.contains(&file));
    assert_eq!(chips, [SavedChip { label: text, text: "selected".into() }]);
}
