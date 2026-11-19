//! The branch of the context (docs/zendo/files/files-branch.md): the top line names the branch of
//! the checkout of the active agent session, of the open document while the note pane has the focus,
//! and of the view while the file tree has the focus; the float names the branch of its note.
use super::version::Other;
use super::*;
use crate::docs::tree::worktree;

impl App {
    /// The branch in the top line (FILES-59, FILES-60, FILES-61).
    pub fn top_branch(&self) -> Option<String> {
        match self.focus {
            Focus::Tree if self.state.tree => self.view_branch(),
            Focus::Note if matches!(self.mode, Mode::Read | Mode::Write) => self.doc_branch(),
            _ => self.session_branch(),
        }
    }

    /// The branch of the checkout of the active agent session: its worktree, or the main checkout
    /// while it has none (FILES-59).
    pub fn session_branch(&self) -> Option<String> {
        match self.worktree_of(self.state.session) {
            Some(wt) => self.facts.branch(&self.cfg.root.join(wt)),
            None => self.main_branch(),
        }
    }

    /// The branch of the checkout of the document on the screen: the version that `m` or `u` shows
    /// (FILES-63), the note in write mode, else the open document. A new note goes to the checkout of
    /// the session (FILES-49, FILES-62).
    pub fn doc_branch(&self) -> Option<String> {
        let path = match (self.files.peek().on, self.mode) {
            (Some(Other::Main), _) => return Some("main".into()),
            (Some(Other::Disk), _) => return self.main_branch(),
            (None, Mode::Write) => self.writer.path().map(PathBuf::as_path),
            (None, _) => self.files.current_path(),
        };
        path.map_or_else(|| self.session_branch(), |p| self.branch_at(p))
    }

    /// The branch of the checkout that the view of the tree lists; none in a worktree view while
    /// the session has no worktree (FILES-61, FILES-64).
    fn view_branch(&self) -> Option<String> {
        let tree = self.files.tree();
        match tree.view.is_worktree() {
            true => tree.worktree.as_ref().and_then(|wt| self.facts.branch(&self.cfg.root.join(wt))),
            false => self.main_branch(),
        }
    }

    /// The branch of the checkout that holds a file: a worktree, the main checkout, or another repo.
    fn branch_at(&self, p: &Path) -> Option<String> {
        let rel = p.strip_prefix(&self.cfg.root).ok().map(|r| r.to_string_lossy().into_owned());
        match rel {
            Some(r) if worktree::split(&r).is_none() => self.main_branch(),
            _ => self.facts.branch(p),
        }
    }

    /// The branch of the main checkout, from the memo that a worker keeps fresh; the branch of the
    /// start where git does not say it (FILES-65, IO-1).
    fn main_branch(&self) -> Option<String> {
        self.facts.branch(&self.cfg.root).or_else(|| self.files.branch().filter(|b| !b.is_empty()).map(str::to_string))
    }
}
