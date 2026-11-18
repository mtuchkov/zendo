//! The composer without an app (ADR 0008).
use super::*;
use crate::agent::Command;
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

// ADR 0008: the chips in the input give their text to the context and their files to the
// attachments; a chip that the user deleted from the text gives nothing.
#[test]
fn the_question_takes_the_context_and_the_files_of_its_chips() {
    let mut c = Composer::default();
    c.add_chip("sel".into(), Some("selected text".into()), None, false);
    c.add_chip("img".into(), None, Some(Attachment::Image("/tmp/a.png".into())), false);
    c.add_chip("gone".into(), Some("not sent".into()), None, false);
    let kept = c.input().text().replace("⟦gone⟧ ", "");
    c.input_mut().take();
    c.input_mut().type_str(&format!("{kept}why?"));
    assert_eq!(c.take_question(), "⟦sel⟧ ⟦img⟧ why?");
    assert!(c.sends_image());
    assert_eq!(c.take_attach().len(), 1);
    assert_eq!(c.take_context().as_deref(), Some("selected text"));
    assert!(c.input().is_empty() && c.chips().is_empty() && !c.has_context());
}

// ADR 0008: with a context and no words of the user, the default question goes.
#[test]
fn only_chips_send_the_default_question() {
    let mut c = Composer::default();
    c.set_context("the delta".into());
    c.set_default("What changed?".into(), "the changes");
    assert_eq!(c.about(), "the changes");
    assert_eq!(c.take_question(), "What changed?");
    c.add_chip("sel".into(), Some("text".into()), None, false);
    assert_eq!(c.take_question(), format!("⟦sel⟧ {}", crate::app::DELTA_QUESTION), "no default left: the question of the delta");
    assert_eq!(c.take_context().as_deref(), Some("the delta\ntext"));
    c.input_mut().type_str("plain");
    assert_eq!(c.take_question(), "plain", "no context: the text as it is");
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
    let root = std::env::temp_dir().join(format!("zendo-composer-at-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("docs")).unwrap();
    std::fs::write(root.join("docs/plan.md"), "# plan\n").unwrap();
    std::fs::write(root.join("docs/other.md"), "# other\n").unwrap();
    let mut c = Composer::default();
    c.input_mut().type_str("see @pla");
    c.update_at(&root, &[]);
    assert_eq!(c.at().map(|p| p.hits.clone()), Some(vec![PathBuf::from("docs/plan.md")]));
    assert_eq!(c.key_at(key(KeyCode::Char('x'))), AtKey::Unused);
    assert_eq!(c.key_at(key(KeyCode::Enter)), AtKey::Taken("docs/plan.md".into(), false));
    assert_eq!(c.input().text(), "see ");
    assert!(c.at().is_none());
    c.pick_template();
    assert_eq!(c.input().text(), "see @");
    c.update_at(&root, &["docs/other.md".into()]);
    assert_eq!(c.at().map(|p| (p.templates, p.hits.clone())), Some((true, vec![PathBuf::from("docs/other.md")])));
    c.close_picks();
    assert!(c.at().is_none());
    let _ = std::fs::remove_dir_all(&root);
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

fn history() -> Vec<String> {
    vec!["first".into(), "two\nlines".into(), "last".into()]
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
