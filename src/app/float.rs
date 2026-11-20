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
    /// The title of the note: asked, and the one of the agent (FLOAT-18, FLOAT-19).
    pub title: crate::agent::work::Titling,
    behind: Behind,
}

/// The state of the app that the float takes for itself, and gives back.
struct Behind {
    draft: Draft,
    mode: Mode,
    back: Mode,
    focus: Focus,
    current: Option<PathBuf>,
    /// The mark of that document: it shows as it is in each session (SPEC-11).
    as_is: Option<PathBuf>,
    pinned: Option<String>,
    view: Reader,
    write_rows: Vec<write_rows::RowSpan>,
}

macro_rules! swap_fields {
    ($app:expr, $b:expr, $($f:ident),*) => { $(std::mem::swap(&mut $app.$f, &mut $b.$f);)* };
}

impl Behind {
    /// Write mode on an empty note, without a title line (FLOAT-17).
    fn new_note() -> Self {
        Self {
            draft: Draft::new_note(),
            mode: Mode::Write,
            back: Mode::Write,
            focus: Focus::Note,
            current: None,
            as_is: None,
            pinned: None,
            view: Reader::default(),
            write_rows: Vec::new(),
        }
    }
}

impl Float {
    /// Give the app the other state: the float's or the screen's behind it.
    pub fn swap(&mut self, app: &mut App) {
        let b = &mut self.behind;
        swap_fields!(app, b, mode, back, focus);
        app.writer.swap_draft(&mut b.draft);
        app.files.swap_current(&mut b.current, &mut b.as_is);
        app.threads.swap_pinned(&mut b.pinned);
        app.reader.swap_view(&mut b.view);
    }

    /// The rows of write mode go with the state: the float and the screen behind keep their own.
    pub fn swap_rows(&mut self, frame: &mut crate::ui::frame::Frame) {
        frame.swap_write_rows(&mut self.behind.write_rows);
    }
}

impl App {
    /// `alt-n`, `n`: a new note in a float over the screen; the open file in write mode is saved
    /// first (FLOAT-1, FLOAT-4, FLOAT-5). Nothing while a float is open (FLOAT-11).
    pub fn open_float(&mut self) {
        if self.writer.float().is_some() {
            return;
        }
        if self.mode == Mode::Write {
            self.save();
        }
        let mut fl = Float { opened: Local::now(), rect: Rect::default(), title: Default::default(), behind: Behind::new_note() };
        fl.swap(self);
        fl.swap_rows(&mut self.frame);
        self.writer.set_float(fl);
    }

    /// `esc`: save the note, ask the name model for its title, and give back the screen as it was.
    /// A note without text makes no file (FLOAT-3, FLOAT-12, FLOAT-13, FLOAT-19).
    pub fn close_float(&mut self) {
        self.save();
        let made = self.writer.path().cloned();
        if let Some(p) = &made {
            self.title_on_close(p.clone());
        }
        let Some(mut fl) = self.writer.take_float() else { return };
        fl.swap(self);
        fl.swap_rows(&mut self.frame);
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
            self.did(Pane::Global, "help");
            return self.open_help(Pane::Write);
        }
        if self.date_key(Pane::Write, &k) || self.selection_key(Pane::Write, &k) {
            return;
        }
        let local = self.keys.local(Pane::Write, &k);
        // The keys that do nothing in the float run no action (FLOAT-10).
        self.did_some(Pane::Write, local.filter(|a| !matches!(*a, "chat" | "delta" | "toggle_tree" | "add_to_chat")));
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
