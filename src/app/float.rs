//! The new note in a float (FLOAT-1 to FLOAT-15). While the float is open, the app has the state
//! of write mode on the new note, so every key of write mode works as in the note pane; the state
//! of the screen behind waits in the float and comes back when the float closes.
use super::*;
use crate::keys::Pane;

/// The new note in a float, and the screen behind it.
pub struct Float {
    /// When the user opened the float: the date under the title (FLOAT-7).
    pub opened: chrono::DateTime<Local>,
    /// The area of the float in the last frame, for the mouse.
    pub rect: Rect,
    behind: Behind,
}

/// The fields of the app that the float takes for itself, and gives back.
struct Behind {
    editor: Editor,
    editor_path: Option<PathBuf>,
    last_edit: Instant,
    saved_at: Option<chrono::DateTime<Local>>,
    mode: Mode,
    back: Mode,
    focus: Focus,
    current: Option<PathBuf>,
    pinned: Option<String>,
    scroll: usize,
    read_cursor: usize,
    read_anchor: Option<(usize, usize)>,
    read_col: usize,
    read_want: usize,
    keep_y: Option<usize>,
    cursor_y: usize,
    write_rows: Vec<write_rows::RowSpan>,
}

macro_rules! swap_fields {
    ($app:expr, $b:expr, $($f:ident),*) => { $(std::mem::swap(&mut $app.$f, &mut $b.$f);)* };
}

impl Behind {
    /// Write mode on an empty note, with the cursor after `# `.
    fn new_note() -> Self {
        let mut editor = Editor::new("# ");
        editor.move_to_end();
        Self {
            editor,
            editor_path: None,
            last_edit: Instant::now(),
            saved_at: None,
            mode: Mode::Write,
            back: Mode::Write,
            focus: Focus::Note,
            current: None,
            pinned: None,
            scroll: 0,
            read_cursor: 0,
            read_anchor: None,
            read_col: 0,
            read_want: 0,
            keep_y: None,
            cursor_y: 0,
            write_rows: Vec::new(),
        }
    }
}

impl Float {
    /// Give the app the other state: the float's or the screen's behind it.
    pub fn swap(&mut self, app: &mut App) {
        let b = &mut self.behind;
        swap_fields!(app, b, editor, editor_path, last_edit, saved_at, mode, back, focus, current, pinned);
        swap_fields!(app, b, scroll, read_cursor, read_anchor, read_col, read_want, keep_y, cursor_y, write_rows);
    }
}

impl App {
    /// `alt-n`, `n`: a new note in a float over the screen; the open file in write mode is saved
    /// first (FLOAT-1, FLOAT-4, FLOAT-5). Nothing while a float is open (FLOAT-11).
    pub fn open_float(&mut self) {
        if self.float.is_some() {
            return;
        }
        if self.mode == Mode::Write {
            self.save();
        }
        let mut fl = Float { opened: Local::now(), rect: Rect::default(), behind: Behind::new_note() };
        fl.swap(self);
        self.float = Some(fl);
    }

    /// `esc`: save the note, ask for a missing title, and give back the screen as it was. A note
    /// without text makes no file (FLOAT-3, FLOAT-12, FLOAT-13).
    pub fn close_float(&mut self) {
        self.save();
        if self.editor_path.is_some() {
            self.ask_for_meta();
        }
        let made = self.editor_path.clone();
        let Some(mut fl) = self.float.take() else { return };
        fl.swap(self);
        // The new note is a recent file (FIND-6).
        if let Some(p) = made {
            self.add_recent(&p);
        }
    }

    /// A key in the float: the keys of write mode; the keys that open a pane or move the focus do
    /// nothing (FLOAT-10).
    pub(crate) fn key_float(&mut self, k: KeyEvent) {
        if self.mode == Mode::Help {
            self.mode = self.back;
            return;
        }
        let global = self.keys.local(Pane::Global, &k);
        if global == Some("help") {
            return self.open_help(Pane::Write);
        }
        if self.selection_key(Pane::Write, &k) {
            return;
        }
        let local = self.keys.local(Pane::Write, &k);
        match (local, global) {
            (Some("done"), _) => self.close_float(),
            (Some("chat" | "delta" | "toggle_tree" | "add_to_chat"), _) => {}
            (Some(_), _) | (None, None) => self.key_write(k, local),
            (None, Some("quit")) => {
                self.close_float();
                self.global_key("quit", Pane::Write);
            }
            (None, Some(a @ ("wider" | "narrower"))) => {
                self.global_key(a, Pane::Write);
            }
            (None, Some(_)) => {}
        }
    }
}
