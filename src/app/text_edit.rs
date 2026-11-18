//! The edit actions that the rich editors share: the note in write mode, the float and the chat
//! input (TEXT-4, TEXT-5, TEXT-6). Paste stays with each of them: the chat takes images and files.
use super::Pointer;
use crate::text::editor::Editor;

/// An action of the keymap on a rich editor: `Some(true)` when the text changed, `Some(false)`
/// when only the cursor or the clipboard did, `None` when it is no shared action.
pub(crate) fn rich_action(e: &mut Editor, a: &str, pointer: &mut Pointer) -> Option<bool> {
    let changed = match a {
        "copy" => {
            pointer.copy(e.selected_text()?);
            false
        }
        "cut" => {
            pointer.copy(e.selected_text()?);
            e.delete_selection()
        }
        "bold" => mark(e, |e| e.wrap_marks("**", "**")),
        "italic" => mark(e, |e| e.wrap_marks("*", "*")),
        "underline" => mark(e, |e| e.wrap_marks("<u>", "</u>")),
        "delete_word" => mark(e, |e| e.delete_selection_or(Editor::delete_word_back)),
        _ => return e.line_move(a).then_some(false),
    };
    Some(changed)
}

fn mark(e: &mut Editor, f: impl FnOnce(&mut Editor)) -> bool {
    f(e);
    true
}
