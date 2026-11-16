//! The worktree views of the file tree in the app: a popup before a file action while the agent of the
//! session answers (FILES-37), and the recap of the worktree diff (FILES-28, FILES-29, FILES-36).
use super::*;
use crate::docs::tree::worktree;

/// zendo looks this often whether the changes of the worktree differ from its last recap.
pub const RECAP_EVERY: Duration = Duration::from_secs(10 * 60);
/// The diff goes to the agent up to this many chars.
const DIFF_MAX: usize = 60_000;
const PROMPT: &str = "Recap the changes of the worktree below, for me to read at a glance.\n\
    Rules: at most 5 short markdown bullets, no preamble, no heading. Do not use tools and do not edit files.\n\
    Say what the changes do as a whole and why they matter. No open questions.\n\n";

#[derive(Default)]
pub struct WorktreeUi {
    /// A file action that waits for the popup, while the agent of the session answers.
    pub confirm: Option<TreeAct>,
    /// When zendo last looked whether the recap of the worktree is out of date.
    pub recap_check: Option<Instant>,
}

/// A file action from the tree: `d`, `r` or `m` on a file, or a new file or folder.
#[derive(Clone, Debug)]
pub enum TreeAct {
    File(Option<PathBuf>, char),
    New { folder: bool },
}

impl TreeAct {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            TreeAct::File(_, 'd') => "delete",
            TreeAct::File(_, 'r') => "rename",
            TreeAct::File(..) => "move",
            TreeAct::New { folder: true } => "new folder",
            TreeAct::New { folder: false } => "new file",
        }
    }
}

impl App {
    /// The agent of session `n` answers: in the chat, or in its Claude Code session.
    pub fn session_works(&self, n: usize) -> bool {
        let chat = self.threads.get(&session_key(n)).is_some_and(|t| t.busy.is_some());
        chat || self.cli.state(n) == Some(crate::session::State::Works)
    }

    /// A file action from the tree. In a worktree view, while its agent answers, a popup asks first.
    pub(crate) fn tree_act(&mut self, act: TreeAct) {
        let works = self.session_works(self.state.session);
        let asked = self.files.tree_act(act, works);
        self.apply(asked);
    }

    /// The popup is open: `y` goes on with the action; any other key drops it.
    pub(crate) fn key_confirm(&mut self, k: KeyEvent) {
        let asked = self.files.key_confirm(k);
        self.apply(asked);
    }

    /// The text of the popup.
    pub fn confirm_text(&self) -> Option<String> {
        self.files.confirm_text(self.state.session)
    }

    /// Called on every tick: while a worktree view shows, write the recap of the worktree again when
    /// its changes differ from those of the last recap, at most every 10 minutes. The agent does not
    /// need to be idle (FILES-29); the recap goes in a separate session without tools (FILES-36).
    pub fn worktree_recap_tick(&mut self) {
        let ready = self.state.tree && self.files.tree().view.is_worktree() && self.work.free() && self.link.ready() && !self.cfg.offline;
        let Some(wt) = self.files.tree().worktree.clone().filter(|_| ready) else { return };
        if self.files.worktree.recap_check.is_some_and(|t| t.elapsed() < RECAP_EVERY) {
            return;
        }
        self.files.worktree.recap_check = Some(Instant::now());
        let diff = worktree::diff(&self.cfg.root.join(&wt), DIFF_MAX);
        let rel = wt.display().to_string();
        let hash = state::hash(&diff);
        let fresh = self.state.files.get(&rel).and_then(|f| f.recap.as_ref()).is_some_and(|r| r.of_hash == hash);
        let Some(a) = self.link.handle().filter(|_| !fresh && !diff.trim().is_empty()) else { return };
        let text = format!("{PROMPT}Worktree: {rel}\n--- the changes against the merge base with main ---\n{diff}\n");
        a.send(Cmd::Prompt { key: format!("{RECAP_PREFIX}{rel}"), known: None, text, attach: vec![] });
        self.work.start(RecapJob { rel, hash, text: String::new(), buf: String::new(), auto: true, failed: None, meta: None, review: false, name: None });
    }
}
