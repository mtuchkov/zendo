//! `tab`: the next pane. The chat input and its transcript are a pair (CLAUDE-36): the first `tab`
//! of a run goes to the other pane of the pair. More `tab`s go around the panes, and come back into
//! the chat at the pane where the run started. Another key, a click, or a pause starts a new run.
use super::*;
use crate::keys::Pane;

/// A pause between two `tab`s this long starts a new run.
pub const TAB_PAUSE: Duration = Duration::from_secs(2);

/// The run of `tab`s that goes on.
#[derive(Clone, Debug, Default)]
pub struct TabRun {
    /// The time of the last `tab`, while no other key or click came after it.
    pub at: Option<Instant>,
    /// The time of the `tab` before the key that comes now.
    last: Option<Instant>,
    /// The run started in the transcript.
    from_transcript: bool,
}

impl TabRun {
    /// A key comes: only a `tab` keeps the run.
    pub fn key(&mut self) {
        self.last = self.at.take();
    }

    /// A click or a paste ends the run.
    pub fn end(&mut self) {
        (self.at, self.last) = (None, None);
    }

    /// A `tab` in `pane`: continue the run, or start a new one there. True when the run started
    /// in the transcript.
    fn step(&mut self, pane: Pane) -> bool {
        if !self.last.is_some_and(|t| t.elapsed() < TAB_PAUSE) {
            self.from_transcript = pane == Pane::Transcript;
        }
        self.at = Some(Instant::now());
        self.from_transcript
    }
}

impl App {
    /// `tab` in `pane`.
    pub(crate) fn tab(&mut self, pane: Pane) {
        let from_transcript = self.tab_run.step(pane);
        let in_chat = matches!(pane, Pane::Chat | Pane::Transcript);
        // The pane of the pair where the run comes into the chat goes on to its sister.
        if in_chat && (pane == Pane::Transcript) == from_transcript {
            return self.show_transcript(pane == Pane::Chat);
        }
        self.transcript.on = false;
        self.next_pane();
        if self.focus != Focus::Claude {
            return;
        }
        if in_chat {
            // No other pane on screen.
            self.focus = Focus::Note;
        } else {
            self.show_transcript(from_transcript);
        }
    }

    fn show_transcript(&mut self, on: bool) {
        self.transcript.on = on;
        if on {
            self.transcript.to_end();
        }
    }
}
